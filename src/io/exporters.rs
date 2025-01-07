use std::io::Write;

use crate::core::System;
use crate::error::Result;
use super::ExportFormat;

pub trait SystemExporter: Send + Sync {
    fn export_system(&self, system: &System) -> Result<Vec<u8>>;
    fn get_format(&self) -> ExportFormat;
}

pub struct JSONExporter;

impl JSONExporter {
    pub fn new() -> Self {
        Self
    }
}

impl SystemExporter for JSONExporter {
    fn export_system(&self, system: &System) -> Result<Vec<u8>> {
        let json = serde_json::to_vec_pretty(system)?;
        Ok(json)
    }

    fn get_format(&self) -> ExportFormat {
        ExportFormat::JSON
    }
}

pub struct CSVExporter;

impl CSVExporter {
    pub fn new() -> Self {
        Self
    }

    fn export_components(&self, system: &System) -> Result<String> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(vec![]);

        // Write components
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

        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }

    fn export_relationships(&self, system: &System) -> Result<String> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(vec![]);

        // Write relationships
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

        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }
}

impl SystemExporter for CSVExporter {
    fn export_system(&self, system: &System) -> Result<Vec<u8>> {
        // Export components and relationships as separate CSV files
        let components_csv = self.export_components(system)?;
        let relationships_csv = self.export_relationships(system)?;

        // Create a ZIP archive containing both files
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        
        // Write components CSV
        zip.start_file("components.csv", zip::write::FileOptions::default())?;
        zip.write_all(components_csv.as_bytes())?;

        // Write relationships CSV
        zip.start_file("relationships.csv", zip::write::FileOptions::default())?;
        zip.write_all(relationships_csv.as_bytes())?;

        let cursor = zip.finish()?;
        Ok(cursor.into_inner())
    }

    fn get_format(&self) -> ExportFormat {
        ExportFormat::CSV
    }
}

pub struct GraphMLExporter;

impl GraphMLExporter {
    pub fn new() -> Self {
        Self
    }
}

impl SystemExporter for GraphMLExporter {
    fn export_system(&self, system: &System) -> Result<Vec<u8>> {
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

    fn get_format(&self) -> ExportFormat {
        ExportFormat::GraphML
    }
} 