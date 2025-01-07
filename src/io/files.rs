use std::path::{Path, PathBuf};
use tokio::fs;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use zip;
use csv;
use std::io::Write;

use crate::core::{System, Component, Relationship};
use crate::core::types::{ComponentType, ComponentState, RelationshipType};
use crate::error::{Error, Result};
use super::{ExportFormat, FileConfig, ExportMetadata, ImportFormat};

pub struct FileManager {
    base_path: PathBuf,
    temp_dir: PathBuf,
    backup_retention: std::time::Duration,
    max_backup_size: usize,
}

impl FileManager {
    pub fn new(config: FileConfig) -> Self {
        Self {
            base_path: config.base_path,
            temp_dir: config.temp_dir,
            backup_retention: config.backup_retention,
            max_backup_size: config.max_backup_size,
        }
    }

    pub async fn save_system(&self, system: &System, format: ExportFormat) -> Result<PathBuf> {
        // Create system directory if it doesn't exist
        let system_dir = self.base_path.join(system.id.to_string());
        fs::create_dir_all(&system_dir).await?;

        // Generate file path
        let extension = match format {
            ExportFormat::JSON => "json",
            ExportFormat::CSV => "zip",
            ExportFormat::GraphML => "graphml",
            ExportFormat::Custom(ref ext) => ext,
        };
        let filename = format!(
            "{}-{}.{}",
            system.name.to_lowercase().replace(' ', "-"),
            Utc::now().format("%Y%m%d-%H%M%S"),
            extension
        );
        let file_path = system_dir.join(filename);

        // Export system to file
        let data = self.export_system(system, format).await?;
        fs::write(&file_path, data).await?;

        Ok(file_path)
    }

    pub async fn load_system(&self, path: &Path) -> Result<System> {
        // Read file contents
        let data = fs::read(path).await?;

        // Determine format from extension
        let format = self.get_format_from_path(path)?;

        // Import system
        self.import_system(&data, format).await
    }

    pub async fn create_backup(&self, system: &System) -> Result<PathBuf> {
        // Create backups directory if it doesn't exist
        let backup_dir = self.base_path.join("backups").join(system.id.to_string());
        fs::create_dir_all(&backup_dir).await?;

        // Generate backup file path
        let filename = format!(
            "backup-{}-{}.json",
            system.name.to_lowercase().replace(' ', "-"),
            Utc::now().format("%Y%m%d-%H%M%S")
        );
        let backup_path = backup_dir.join(filename);

        // Export system to JSON format
        let data = self.export_system(system, ExportFormat::JSON).await?;
        fs::write(&backup_path, data).await?;

        Ok(backup_path)
    }

    pub async fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let backup_dir = self.base_path.join("backups");
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                backups.push(path);
            }
        }

        // Sort by modification time, newest first
        let mut backups_with_time = Vec::new();
        for path in backups {
            if let Ok(metadata) = fs::metadata(&path).await {
                if let Ok(modified) = metadata.modified() {
                    backups_with_time.push((path, modified));
                }
            }
        }
        backups_with_time.sort_by_key(|(_, time)| *time);
        backups_with_time.reverse();

        Ok(backups_with_time.into_iter().map(|(path, _)| path).collect())
    }

    pub async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups().await?;
        let now = std::time::SystemTime::now();

        for backup in backups {
            let metadata = fs::metadata(&backup).await?;
            
            // Check age
            if let Ok(age) = now.duration_since(metadata.modified()?) {
                if age > self.backup_retention {
                    fs::remove_file(&backup).await?;
                    continue;
                }
            }

            // Check size
            if metadata.len() as usize > self.max_backup_size {
                fs::remove_file(&backup).await?;
            }
        }

        Ok(())
    }

    async fn export_system(&self, system: &System, format: ExportFormat) -> Result<Vec<u8>> {
        match format {
            ExportFormat::JSON => {
                let json = serde_json::to_vec_pretty(system)?;
                Ok(json)
            }
            ExportFormat::CSV => {
                let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));

                // Export components
                let mut wtr = csv::WriterBuilder::new()
                    .has_headers(true)
                    .from_writer(vec![]);

                wtr.write_record(&["id", "name", "type", "created_at", "properties"])?;
                for component in system.components.values() {
                    wtr.write_record(&[
                        component.id.to_string(),
                        component.name.clone(),
                        format!("{:?}", component.component_type),
                        component.created_at.to_rfc3339(),
                        serde_json::to_string(&component.properties)?,
                    ])?;
                }
                let components_csv = wtr.into_inner()?;

                // Export relationships
                let mut wtr = csv::WriterBuilder::new()
                    .has_headers(true)
                    .from_writer(vec![]);

                wtr.write_record(&["id", "source_id", "target_id", "type", "properties"])?;
                for relationship in system.relationships.values() {
                    wtr.write_record(&[
                        relationship.id.to_string(),
                        relationship.source_id.to_string(),
                        relationship.target_id.to_string(),
                        format!("{:?}", relationship.relationship_type),
                        serde_json::to_string(&relationship.properties)?,
                    ])?;
                }
                let relationships_csv = wtr.into_inner()?;

                // Create ZIP archive
                zip.start_file("components.csv", zip::write::FileOptions::default())?;
                zip.write_all(&components_csv)?;
                zip.start_file("relationships.csv", zip::write::FileOptions::default())?;
                zip.write_all(&relationships_csv)?;

                let cursor = zip.finish()?;
                Ok(cursor.into_inner())
            }
            ExportFormat::GraphML => {
                let mut output = String::new();

                // Add GraphML header
                output.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns
         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">
"#);

                // Add attribute definitions
                output.push_str(r#"  <key id="name" for="node" attr.name="name" attr.type="string"/>
  <key id="type" for="node" attr.name="type" attr.type="string"/>
  <key id="properties" for="node" attr.name="properties" attr.type="string"/>
  <key id="weight" for="edge" attr.name="weight" attr.type="double"/>
  <key id="type" for="edge" attr.name="type" attr.type="string"/>
  <key id="properties" for="edge" attr.name="properties" attr.type="string"/>
"#);

                // Start graph
                output.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");

                // Add nodes
                for component in system.components.values() {
                    output.push_str(&format!(
                        r#"    <node id="{}">
      <data key="name">{}</data>
      <data key="type">{:?}</data>
      <data key="properties">{}</data>
    </node>
"#,
                        component.id,
                        component.name,
                        component.component_type,
                        serde_json::to_string(&component.properties)?
                    ));
                }

                // Add edges
                for relationship in system.relationships.values() {
                    output.push_str(&format!(
                        r#"    <edge id="{}" source="{}" target="{}">
      <data key="type">{:?}</data>
      <data key="properties">{}</data>
    </edge>
"#,
                        relationship.id,
                        relationship.source_id,
                        relationship.target_id,
                        relationship.relationship_type,
                        serde_json::to_string(&relationship.properties)?
                    ));
                }

                // Close graph and GraphML
                output.push_str("  </graph>\n</graphml>");

                Ok(output.into_bytes())
            }
            ExportFormat::Custom(_) => Err(Error::system("Custom export formats are not supported")),
        }
    }

    async fn import_system(&self, data: &[u8], format: ImportFormat) -> Result<System> {
        match format {
            ImportFormat::JSON => {
                let system: System = serde_json::from_slice(data)?;
                Ok(system)
            }
            ImportFormat::CSV => {
                let mut zip = zip::ZipArchive::new(std::io::Cursor::new(data))?;
                let mut components = HashMap::new();
                let mut relationships = HashMap::new();
                
                // Read components
                {
                    let components_file = zip.by_name("components.csv")?;
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .from_reader(components_file);

                    for result in rdr.records() {
                        let record = result?;
                        let id = Uuid::parse_str(&record[0])?;
                        let now = Utc::now();
                        let component = Component {
                            id,
                            name: record[1].to_string(),
                            component_type: serde_json::from_str(&record[2])?,
                            state: ComponentState::default(),
                            properties: serde_json::from_str(&record[4])?,
                            created_at: now,
                            updated_at: now,
                        };
                        components.insert(id, component);
                    }
                }

                // Read relationships
                {
                    let relationships_file = zip.by_name("relationships.csv")?;
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .from_reader(relationships_file);

                    for result in rdr.records() {
                        let record = result?;
                        let id = Uuid::parse_str(&record[0])?;
                        let now = Utc::now();
                        let relationship = Relationship {
                            id,
                            source_id: Uuid::parse_str(&record[1])?,
                            target_id: Uuid::parse_str(&record[2])?,
                            relationship_type: serde_json::from_str(&record[3])?,
                            properties: serde_json::from_str(&record[5])?,
                            created_at: now,
                            updated_at: now,
                        };
                        relationships.insert(id, relationship);
                    }
                }

                Ok(System {
                    id: Uuid::new_v4(),
                    name: "Imported System".to_string(),
                    description: "Imported from CSV".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    components,
                    relationships,
                    metadata: HashMap::new(),
                })
            }
            ImportFormat::GraphML => {
                Err(Error::system("GraphML import is not yet supported"))
            }
            ImportFormat::Custom(_) => Err(Error::system("Custom import formats are not supported")),
        }
    }

    fn get_extension(&self, format: &ExportFormat) -> &'static str {
        match format {
            ExportFormat::JSON => "json",
            ExportFormat::CSV => "zip",
            ExportFormat::GraphML => "graphml",
            ExportFormat::Custom(_) => "custom",
        }
    }

    fn get_format_from_path(&self, path: &Path) -> Result<ImportFormat> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Ok(ImportFormat::JSON),
            Some("zip") => Ok(ImportFormat::CSV),
            Some("graphml") => Ok(ImportFormat::GraphML),
            Some(ext) => Ok(ImportFormat::Custom(ext.to_string())),
            None => Err(Error::validation("File has no extension")),
        }
    }
} 