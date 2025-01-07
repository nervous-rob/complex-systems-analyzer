use std::path::Path;
use rocksdb::{DB, ColumnFamily, Options, WriteBatch, IteratorMode, Snapshot, AsColumnFamilyRef, BoundColumnFamily};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::core::{System, Component, Relationship};

const CF_NODES: &str = "nodes";
const CF_EDGES: &str = "edges";
const CF_METADATA: &str = "metadata";

pub struct RocksDB {
    db: Arc<DB>,
}

impl RocksDB {
    pub fn new(path: &Path) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Define column families
        let cfs = vec![CF_NODES, CF_EDGES, CF_METADATA];
        
        // Open database with column families
        let db = Arc::new(DB::open_cf(&opts, path, &cfs)
            .map_err(|e| Error::Storage(format!("Failed to open RocksDB: {}", e)))?);

        Ok(Self { db })
    }

    fn get_cf(&self, name: &str) -> Result<Arc<BoundColumnFamily>> {
        self.db.cf_handle(name)
            .ok_or_else(|| Error::Storage(format!("Failed to get column family: {}", name)))
            .map(|cf| cf.clone())
    }

    pub fn store_node(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let cf = self.get_cf(CF_NODES)?;
        self.db.put_cf(&cf, key, value)
            .map_err(|e| Error::Storage(format!("Failed to store node: {}", e)))
    }

    pub fn get_node(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let cf = self.get_cf(CF_NODES)?;
        self.db.get_cf(&cf, key)
            .map_err(|e| Error::Storage(format!("Failed to get node: {}", e)))
    }

    pub fn store_edge(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let cf = self.get_cf(CF_EDGES)?;
        self.db.put_cf(&cf, key, value)
            .map_err(|e| Error::Storage(format!("Failed to store edge: {}", e)))
    }

    pub fn get_edges(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let cf = self.get_cf(CF_EDGES)?;
        let mut edges = Vec::new();
        let iter = self.db.iterator_cf(&cf, IteratorMode::From(prefix, rocksdb::Direction::Forward));
        
        for item in iter {
            let (key, value) = item.map_err(|e| Error::Storage(format!("Failed to iterate edges: {}", e)))?;
            if !key.starts_with(prefix) {
                break;
            }
            edges.push((key.to_vec(), value.to_vec()));
        }

        Ok(edges)
    }

    pub fn store_batch(&self, batch: WriteBatch) -> Result<()> {
        self.db.write(batch)
            .map_err(|e| Error::Storage(format!("Failed to write batch: {}", e)))
    }

    pub fn create_snapshot(&self) -> Result<Snapshot> {
        Ok(self.db.snapshot())
    }

    pub fn compact_range(&self, cf: &impl AsColumnFamilyRef, start: Option<&[u8]>, end: Option<&[u8]>) {
        self.db.compact_range_cf(cf, start, end);
    }

    pub fn store_component(&self, component: &Component) -> Result<()> {
        let key = component.id.as_bytes();
        let value = serde_json::to_vec(component)
            .map_err(|e| Error::Storage(format!("Failed to serialize component: {}", e)))?;
        self.store_node(key, &value)
    }

    pub fn get_component(&self, id: &Uuid) -> Result<Option<Component>> {
        if let Some(data) = self.get_node(id.as_bytes())? {
            let component = serde_json::from_slice(&data)
                .map_err(|e| Error::Storage(format!("Failed to deserialize component: {}", e)))?;
            Ok(Some(component))
        } else {
            Ok(None)
        }
    }

    pub fn store_relationship(&self, relationship: &Relationship) -> Result<()> {
        let key = relationship.id.as_bytes();
        let value = serde_json::to_vec(relationship)
            .map_err(|e| Error::Storage(format!("Failed to serialize relationship: {}", e)))?;
        self.store_edge(key, &value)
    }

    pub fn get_relationships_for_component(&self, component_id: &Uuid) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();
        let prefix = component_id.as_bytes();
        
        for (_, value) in self.get_edges(prefix)? {
            let relationship = serde_json::from_slice(&value)
                .map_err(|e| Error::Storage(format!("Failed to deserialize relationship: {}", e)))?;
            relationships.push(relationship);
        }

        Ok(relationships)
    }

    pub fn store_system_metadata(&self, system_id: &Uuid, metadata: &serde_json::Value) -> Result<()> {
        let key = system_id.as_bytes();
        let value = serde_json::to_vec(metadata)
            .map_err(|e| Error::Storage(format!("Failed to serialize metadata: {}", e)))?;
        
        self.store_node(key, &value)
    }

    pub fn get_system_metadata(&self, system_id: &Uuid) -> Result<Option<serde_json::Value>> {
        if let Some(data) = self.get_node(system_id.as_bytes())? {
            let metadata = serde_json::from_slice(&data)
                .map_err(|e| Error::Storage(format!("Failed to deserialize metadata: {}", e)))?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }
}