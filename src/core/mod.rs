use std::sync::Arc;
use uuid::Uuid;
use std::collections::HashMap;

pub mod system;
pub mod types;

pub use system::{System, Component, Relationship};
pub use types::*;

use crate::error::{Error, Result};
use crate::storage::StorageManager;
use crate::compute::ComputeEngine;
use crate::events::EventBus;

pub trait SystemExt {
    fn components(&self) -> &HashMap<Uuid, Component>;
    fn relationships(&self) -> &HashMap<Uuid, Relationship>;
    fn is_empty(&self) -> bool;
    fn component_types(&self) -> Vec<String>;
    fn weight_range(&self) -> Option<(f32, f32)>;
    fn date_range(&self) -> Option<(f32, f32)>;
}

impl SystemExt for System {
    fn components(&self) -> &HashMap<Uuid, Component> {
        &self.components
    }

    fn relationships(&self) -> &HashMap<Uuid, Relationship> {
        &self.relationships
    }

    fn is_empty(&self) -> bool {
        self.components.is_empty() && self.relationships.is_empty()
    }

    fn component_types(&self) -> Vec<String> {
        let mut types: Vec<_> = self.components.values()
            .map(|c| c.type_name().to_string())
            .collect();
        types.sort();
        types.dedup();
        types
    }

    fn weight_range(&self) -> Option<(f32, f32)> {
        let weights: Vec<_> = self.relationships.values()
            .filter_map(|r| r.weight())
            .collect();
        
        if weights.is_empty() {
            None
        } else {
            Some((
                weights.iter().cloned().fold(f32::INFINITY, f32::min),
                weights.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            ))
        }
    }

    fn date_range(&self) -> Option<(f32, f32)> {
        let dates: Vec<_> = self.components.values()
            .filter_map(|c| c.timestamp())
            .collect();
        
        if dates.is_empty() {
            None
        } else {
            Some((
                dates.iter().cloned().fold(f32::INFINITY, f32::min),
                dates.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            ))
        }
    }
}

pub struct SystemManager {
    storage: Arc<StorageManager>,
    compute: Arc<ComputeEngine>,
    event_bus: Arc<EventBus>,
}

impl SystemManager {
    pub fn new(
        storage: Arc<StorageManager>,
        compute: Arc<ComputeEngine>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            storage,
            compute,
            event_bus,
        }
    }

    pub async fn create_system(&self, name: String, description: String) -> Result<System> {
        let system = System::new(name, description);
        self.storage.store_system(&system).await?;
        Ok(system)
    }

    pub async fn load_system(&self, id: &Uuid) -> Result<System> {
        self.storage.load_system(id).await
    }

    pub async fn save_system(&self, system: &System) -> Result<()> {
        system.validate()?;
        self.storage.store_system(system).await
    }

    pub async fn add_component(&self, system: &mut System, component: Component) -> Result<()> {
        system.add_component(component.clone())?;
        self.storage.store_component(&component).await?;
        Ok(())
    }

    pub async fn remove_component(&self, system: &mut System, id: &Uuid) -> Result<()> {
        system.remove_component(id)?;
        // Storage cleanup would be handled by the storage manager
        Ok(())
    }

    pub async fn add_relationship(&self, system: &mut System, relationship: Relationship) -> Result<()> {
        system.add_relationship(relationship.clone())?;
        self.storage.store_relationship(&relationship).await?;
        Ok(())
    }

    pub async fn remove_relationship(&self, system: &mut System, id: &Uuid) -> Result<()> {
        system.remove_relationship(id)?;
        // Storage cleanup would be handled by the storage manager
        Ok(())
    }

    pub async fn update_component_state(
        &self,
        system: &mut System,
        id: &Uuid,
        state: ComponentState,
    ) -> Result<()> {
        if let Some(component) = system.get_component_mut(id) {
            component.update_state(state);
            self.storage.store_component(component).await?;
        } else {
            return Err(Error::component_not_found(*id));
        }
        Ok(())
    }

    pub fn get_system_metrics(&self, system: &System) -> SystemMetrics {
        let active_components = system.components.values()
            .filter(|c| matches!(c.state.status, ComponentStatus::Active))
            .count();
        let error_components = system.components.values()
            .filter(|c| matches!(c.state.status, ComponentStatus::Error))
            .count();

        SystemMetrics::new(
            system.components.len(),
            system.relationships.len(),
            active_components,
            error_components,
        )
    }

    pub fn validate_system(&self, system: &System) -> Result<()> {
        system.validate()
    }
} 