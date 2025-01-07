use std::path::Path;
use rusqlite::{Connection, params, Result as SqliteResult};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Mutex;

use crate::error::{Error, Result};
use crate::core::{Component, System};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub version: u32,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub id: Uuid,
    pub system_id: Uuid,
    pub name: String,
    pub component_type: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub properties: serde_json::Value,
}

pub struct SQLiteDB {
    connection: Mutex<Connection>,
}

impl SQLiteDB {
    pub fn new(path: &Path) -> Result<Self> {
        let connection = Connection::open(path)
            .map_err(|e| Error::Storage(format!("Failed to open SQLite database: {}", e)))?;

        let db = Self { 
            connection: Mutex::new(connection)
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn init_schema(&self) -> Result<()> {
        self.connection.lock().unwrap().execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS systems (
                id BLOB PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                version INTEGER NOT NULL,
                properties TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS components (
                id BLOB PRIMARY KEY,
                system_id BLOB NOT NULL,
                name TEXT NOT NULL,
                component_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                properties TEXT NOT NULL,
                FOREIGN KEY (system_id) REFERENCES systems(id)
            );

            CREATE INDEX IF NOT EXISTS idx_components_system_id ON components(system_id);
            "#,
        ).map_err(|e| Error::Storage(format!("Failed to initialize schema: {}", e)))?;

        // Check and update schema version
        let version: Option<u32> = self.connection.lock().unwrap()
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .ok();

        match version {
            None => {
                self.connection.lock().unwrap().execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    params![SCHEMA_VERSION],
                ).map_err(|e| Error::Storage(format!("Failed to set schema version: {}", e)))?;
            }
            Some(v) if v < SCHEMA_VERSION => {
                self.run_migration(v)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn store_metadata(&self, system_id: &Uuid, metadata: &SystemMetadata) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO systems (id, name, description, created_at, modified_at, version, properties)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                system_id.as_bytes(),
                metadata.name,
                metadata.description,
                metadata.created_at.to_rfc3339(),
                metadata.modified_at.to_rfc3339(),
                metadata.version,
                serde_json::to_string(&metadata.properties)
                    .map_err(|e| Error::Storage(format!("Failed to serialize properties: {}", e)))?
            ],
        ).map_err(|e| Error::Storage(format!("Failed to store system metadata: {}", e)))?;

        Ok(())
    }

    pub fn get_metadata(&self, system_id: &Uuid) -> Result<Option<SystemMetadata>> {
        let result = self.connection.lock().unwrap().query_row(
            r#"
            SELECT name, description, created_at, modified_at, version, properties
            FROM systems WHERE id = ?1
            "#,
            params![system_id.as_bytes()],
            |row| {
                Ok(SystemMetadata {
                    id: *system_id,
                    name: row.get(0)?,
                    description: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    version: row.get(4)?,
                    properties: serde_json::from_str(&row.get::<_, String>(5)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                })
            },
        );

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Storage(format!("Failed to get system metadata: {}", e))),
        }
    }

    pub fn store_component_metadata(&self, component: &Component) -> Result<()> {
        self.connection.lock().unwrap().execute(
            r#"
            INSERT OR REPLACE INTO components (id, name, component_type, created_at, modified_at, properties)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                component.id.as_bytes(),
                component.name,
                format!("{:?}", component.component_type),
                component.created_at.to_rfc3339(),
                component.updated_at.to_rfc3339(),
                serde_json::to_string(&component.properties)
                    .map_err(|e| Error::Storage(format!("Failed to serialize properties: {}", e)))?
            ],
        ).map_err(|e| Error::Storage(format!("Failed to store component metadata: {}", e)))?;

        Ok(())
    }

    pub fn get_component_metadata(&self, id: &Uuid) -> Result<Option<ComponentMetadata>> {
        let result = self.connection.lock().unwrap().query_row(
            r#"
            SELECT system_id, name, component_type, created_at, modified_at, properties
            FROM components WHERE id = ?1
            "#,
            params![id.as_bytes()],
            |row| {
                Ok(ComponentMetadata {
                    id: *id,
                    system_id: Uuid::from_slice(&row.get::<_, Vec<u8>>(0)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Blob,
                            Box::new(e),
                        ))?,
                    name: row.get(1)?,
                    component_type: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    properties: serde_json::from_str(&row.get::<_, String>(5)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                })
            },
        );

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Storage(format!("Failed to get component metadata: {}", e))),
        }
    }

    pub fn run_migration(&self, current_version: u32) -> Result<()> {
        let guard = &mut *self.connection.lock().unwrap();
        let tx = guard.transaction()
            .map_err(|e| Error::Storage(format!("Failed to start migration transaction: {}", e)))?;

        // Run migrations based on version
        match current_version {
            1 => {},
            _ => {}
        }

        tx.execute(
            "UPDATE schema_version SET version = ?1",
            params![SCHEMA_VERSION],
        ).map_err(|e| Error::Storage(format!("Failed to update schema version: {}", e)))?;

        tx.commit()
            .map_err(|e| Error::Storage(format!("Failed to commit migration: {}", e)))?;

        Ok(())
    }
} 