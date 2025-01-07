use std::path::{Path, PathBuf};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};
use crate::core::System;

mod exporters;
mod importers;
mod files;

pub use exporters::{SystemExporter, JSONExporter, CSVExporter, GraphMLExporter};
pub use importers::{SystemImporter, JSONImporter};
pub use files::FileManager;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    JSON,
    CSV,
    GraphML,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportFormat {
    JSON,
    CSV,
    GraphML,
    Custom(String),
}

pub struct FileConfig {
    pub base_path: PathBuf,
    pub temp_dir: PathBuf,
    pub backup_retention: std::time::Duration,
    pub max_backup_size: usize,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("data"),
            temp_dir: std::env::temp_dir().join("csa"),
            backup_retention: std::time::Duration::from_secs(7 * 24 * 3600), // 7 days
            max_backup_size: 1024 * 1024 * 1024, // 1GB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub id: Uuid,
    pub format: ExportFormat,
    pub timestamp: DateTime<Utc>,
    pub system_id: Uuid,
    pub checksum: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    pub id: Uuid,
    pub format: ImportFormat,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub version: String,
}

#[async_trait]
pub trait IOManager: Send + Sync {
    async fn export_system(&self, system: &System, format: ExportFormat) -> Result<Vec<u8>>;
    async fn import_system(&self, data: &[u8], format: ImportFormat) -> Result<System>;
    async fn save_system(&self, system: &System) -> Result<PathBuf>;
    async fn load_system(&self, path: &Path) -> Result<System>;
    async fn create_backup(&self, system: &System) -> Result<PathBuf>;
    async fn restore_backup(&self, backup_path: &Path) -> Result<System>;
    async fn list_backups(&self) -> Result<Vec<PathBuf>>;
    async fn cleanup_old_backups(&self) -> Result<()>;
}

pub struct DefaultIOManager {
    exporters: Vec<Box<dyn SystemExporter>>,
    importers: Vec<Box<dyn SystemImporter>>,
    file_manager: FileManager,
}

impl DefaultIOManager {
    pub fn new(config: FileConfig) -> Self {
        let mut exporters: Vec<Box<dyn SystemExporter>> = Vec::new();
        exporters.push(Box::new(JSONExporter::new()));
        exporters.push(Box::new(CSVExporter::new()));
        exporters.push(Box::new(GraphMLExporter::new()));

        let mut importers: Vec<Box<dyn SystemImporter>> = Vec::new();
        importers.push(Box::new(JSONImporter::new()));

        Self {
            exporters,
            importers,
            file_manager: FileManager::new(config),
        }
    }

    fn get_exporter(&self, format: ExportFormat) -> Result<&dyn SystemExporter> {
        self.exporters
            .iter()
            .find(|e| e.get_format() == format)
            .map(|e| e.as_ref())
            .ok_or_else(|| Error::system(format!("Unsupported export format: {:?}", format)))
    }

    fn get_importer(&self, format: ImportFormat) -> Result<&dyn SystemImporter> {
        self.importers
            .iter()
            .find(|i| i.get_format() == format)
            .map(|i| i.as_ref())
            .ok_or_else(|| Error::system(format!("Unsupported import format: {:?}", format)))
    }
}

#[async_trait]
impl IOManager for DefaultIOManager {
    async fn export_system(&self, system: &System, format: ExportFormat) -> Result<Vec<u8>> {
        let exporter = self.get_exporter(format)?;
        exporter.export_system(system)
    }

    async fn import_system(&self, data: &[u8], format: ImportFormat) -> Result<System> {
        let importer = self.get_importer(format)?;
        
        // Validate import data
        importer.validate_import(data)?;
        
        // Perform import
        importer.import_system(data)
    }

    async fn save_system(&self, system: &System) -> Result<PathBuf> {
        self.file_manager.save_system(system, ExportFormat::JSON).await
    }

    async fn load_system(&self, path: &Path) -> Result<System> {
        self.file_manager.load_system(path).await
    }

    async fn create_backup(&self, system: &System) -> Result<PathBuf> {
        self.file_manager.create_backup(system).await
    }

    async fn restore_backup(&self, backup_path: &Path) -> Result<System> {
        self.file_manager.load_system(backup_path).await
    }

    async fn list_backups(&self) -> Result<Vec<PathBuf>> {
        self.file_manager.list_backups().await
    }

    async fn cleanup_old_backups(&self) -> Result<()> {
        self.file_manager.cleanup_old_backups().await
    }
} 