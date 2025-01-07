use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::core::{System, Component, Relationship};

mod rocks;
mod sqlite;
mod cache;

use rocks::RocksDB;
use sqlite::{SQLiteDB, SystemMetadata, ComponentMetadata};
use cache::{Cache, CacheStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub rocks_db_path: PathBuf,
    pub sqlite_path: PathBuf,
    pub cache_size: usize,
    pub backup_interval: Duration,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            rocks_db_path: PathBuf::from("data/rocks"),
            sqlite_path: PathBuf::from("data/sqlite/metadata.db"),
            cache_size: 1024 * 1024 * 100, // 100MB
            backup_interval: Duration::from_secs(3600), // 1 hour
        }
    }
}

pub struct StorageManager {
    rocks_db: Arc<RocksDB>,
    sqlite: Arc<SQLiteDB>,
    cache: Arc<Cache>,
    config: StorageConfig,
}

impl StorageManager {
    pub fn new(config: StorageConfig) -> Result<Self> {
        // Create directories if they don't exist
        std::fs::create_dir_all(&config.rocks_db_path)
            .map_err(|e| Error::Storage(format!("Failed to create RocksDB directory: {}", e)))?;
        std::fs::create_dir_all(config.sqlite_path.parent().unwrap())
            .map_err(|e| Error::Storage(format!("Failed to create SQLite directory: {}", e)))?;

        // Initialize storage engines
        let rocks_db = Arc::new(RocksDB::new(&config.rocks_db_path)?);
        let sqlite = Arc::new(SQLiteDB::new(&config.sqlite_path)?);
        let cache = Arc::new(Cache::new(None));

        Ok(Self {
            rocks_db,
            sqlite,
            cache,
            config,
        })
    }

    pub async fn init_storage(&self) -> Result<()> {
        // Any additional initialization can go here
        Ok(())
    }

    pub async fn store_system(&self, system: &System) -> Result<()> {
        // Store in RocksDB
        let metadata = SystemMetadata {
            id: system.id,
            name: system.name.clone(),
            description: system.description.clone(),
            created_at: system.created_at,
            modified_at: system.updated_at,
            version: 1,
            properties: serde_json::to_value(&system.metadata)
                .map_err(|e| Error::Storage(format!("Failed to convert metadata: {}", e)))?,
        };

        // Store metadata in SQLite
        self.sqlite.store_metadata(&system.id, &metadata)?;

        // Update cache
        self.cache.store_system(system.clone());

        Ok(())
    }

    pub async fn load_system(&self, id: &Uuid) -> Result<System> {
        // Try cache first
        if let Some(system) = self.cache.get_system(id) {
            return Ok(system);
        }

        // Load metadata from SQLite
        let metadata = self.sqlite.get_metadata(id)?
            .ok_or_else(|| Error::Storage(format!("System not found: {}", id)))?;

        // Convert JSON metadata to HashMap<String, String>
        let system_metadata: HashMap<String, String> = metadata.properties.as_object()
            .ok_or_else(|| Error::Storage("Invalid metadata format".into()))?
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
            .collect();

        // Create system from metadata
        let system = System {
            id: metadata.id,
            name: metadata.name,
            description: metadata.description,
            created_at: metadata.created_at,
            updated_at: metadata.modified_at,
            components: HashMap::new(),
            relationships: HashMap::new(),
            metadata: system_metadata,
        };

        // Update cache
        self.cache.store_system(system.clone());

        Ok(system)
    }

    pub async fn store_component(&self, component: &Component) -> Result<()> {
        // Store in RocksDB
        self.rocks_db.store_component(component)?;

        // Store metadata in SQLite
        self.sqlite.store_component_metadata(component)?;

        // Update cache
        self.cache.store_component(component.clone());

        Ok(())
    }

    pub async fn load_component(&self, id: &Uuid) -> Result<Component> {
        // Try cache first
        if let Some(component) = self.cache.get_component(id) {
            return Ok(component);
        }

        // Load from RocksDB
        let component = self.rocks_db.get_component(id)?
            .ok_or_else(|| Error::Storage(format!("Component not found: {}", id)))?;

        // Update cache
        self.cache.store_component(component.clone());

        Ok(component)
    }

    pub async fn store_relationship(&self, relationship: &Relationship) -> Result<()> {
        // Store in RocksDB
        self.rocks_db.store_relationship(relationship)?;

        // Update cache
        self.cache.store_relationship(relationship.clone());

        Ok(())
    }

    pub async fn load_relationships(&self, component_id: &Uuid) -> Result<Vec<Relationship>> {
        // Load from RocksDB
        let relationships = self.rocks_db.get_relationships_for_component(component_id)?;

        // Update cache
        for relationship in &relationships {
            self.cache.store_relationship(relationship.clone());
        }

        Ok(relationships)
    }

    pub async fn backup_database(&self, path: &Path) -> Result<()> {
        // Create backup directory
        std::fs::create_dir_all(path)
            .map_err(|e| Error::Storage(format!("Failed to create backup directory: {}", e)))?;

        // Create snapshot
        let snapshot = self.rocks_db.create_snapshot()?;

        // TODO: Implement backup logic using the snapshot
        // This would involve copying the snapshot to the backup location

        Ok(())
    }

    pub async fn restore_database(&self, path: &Path) -> Result<()> {
        // TODO: Implement restore logic
        // This would involve stopping the current database,
        // copying the backup files, and restarting

        Ok(())
    }

    pub fn get_storage_stats(&self) -> StorageStats {
        StorageStats {
            cache: self.cache.get_stats(),
            // Add more stats as needed
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub cache: CacheStats,
    // Add more stats as needed
} 