use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use crate::error::{Error, Result};
use crate::core::types::{ComponentState, ComponentType, RelationshipType};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct System {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) components: HashMap<Uuid, Component>,
    pub(crate) relationships: HashMap<Uuid, Relationship>,
    pub(crate) metadata: HashMap<String, String>,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: Uuid,
    pub name: String,
    pub component_type: ComponentType,
    pub properties: HashMap<String, String>,
    pub state: ComponentState,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: RelationshipType,
    pub properties: HashMap<String, String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Node => write!(f, "Node"),
            ComponentType::Agent => write!(f, "Agent"),
            ComponentType::Process => write!(f, "Process"),
            ComponentType::Resource => write!(f, "Resource"),
            ComponentType::Interface => write!(f, "Interface"),
            ComponentType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl System {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            components: HashMap::new(),
            relationships: HashMap::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_component(&mut self, component: Component) -> Result<()> {
        if self.components.contains_key(&component.id) {
            return Err(Error::duplicate_component(component.id));
        }
        self.components.insert(component.id, component);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_relationship(&mut self, relationship: Relationship) -> Result<()> {
        if self.relationships.contains_key(&relationship.id) {
            return Err(Error::duplicate_relationship(relationship.id));
        }
        
        // Verify that both source and target components exist
        if !self.components.contains_key(&relationship.source_id) {
            return Err(Error::component_not_found(relationship.source_id));
        }
        if !self.components.contains_key(&relationship.target_id) {
            return Err(Error::component_not_found(relationship.target_id));
        }

        self.relationships.insert(relationship.id, relationship);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_component(&self, id: &Uuid) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn get_component_mut(&mut self, id: &Uuid) -> Option<&mut Component> {
        self.components.get_mut(id)
    }

    pub fn get_relationship(&self, id: &Uuid) -> Option<&Relationship> {
        self.relationships.get(id)
    }

    pub fn remove_component(&mut self, id: &Uuid) -> Result<()> {
        if !self.components.contains_key(id) {
            return Err(Error::component_not_found(*id));
        }
        
        self.components.remove(id);
        // Remove any relationships connected to this component
        self.relationships.retain(|_, rel| {
            rel.source_id != *id && rel.target_id != *id
        });
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_relationship(&mut self, id: &Uuid) -> Result<()> {
        if !self.relationships.contains_key(id) {
            return Err(Error::relationship_not_found(*id));
        }
        
        self.relationships.remove(id);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty() && self.relationships.is_empty()
    }

    pub fn component_types(&self) -> Vec<&ComponentType> {
        self.components.values()
            .map(|c| &c.component_type)
            .collect()
    }

    pub fn validate(&self) -> Result<()> {
        // Check for orphaned relationships
        for relationship in self.relationships.values() {
            if !self.components.contains_key(&relationship.source_id) {
                return Err(Error::orphaned_relationship(relationship.id, relationship.source_id));
            }
            if !self.components.contains_key(&relationship.target_id) {
                return Err(Error::orphaned_relationship(relationship.id, relationship.target_id));
            }
        }

        // Check for circular dependencies
        self.check_circular_dependencies()?;

        Ok(())
    }

    fn check_circular_dependencies(&self) -> Result<()> {
        let mut visited = HashMap::new();
        let mut stack = Vec::new();

        for component_id in self.components.keys() {
            if !visited.contains_key(component_id) {
                self.detect_cycle(component_id, &mut visited, &mut stack)?;
            }
        }

        Ok(())
    }

    fn detect_cycle(
        &self,
        current: &Uuid,
        visited: &mut HashMap<Uuid, bool>,
        stack: &mut Vec<Uuid>,
    ) -> Result<()> {
        visited.insert(*current, true);
        stack.push(*current);

        let dependencies: Vec<_> = self.relationships.values()
            .filter(|r| r.source_id == *current)
            .map(|r| r.target_id)
            .collect();

        for &next in dependencies.iter() {
            if !visited.contains_key(&next) {
                self.detect_cycle(&next, visited, stack)?;
            } else if stack.contains(&next) {
                return Err(Error::circular_dependency(*current, next));
            }
        }

        stack.pop();
        Ok(())
    }

    pub fn analyze_connectivity(&self) -> HashMap<Uuid, f64> {
        let mut connectivity: HashMap<Uuid, f64> = HashMap::new();
        
        for component_id in self.components.keys() {
            let mut connections = 0.0;
            for relationship in self.relationships.values() {
                if relationship.source_id == *component_id || relationship.target_id == *component_id {
                    connections += 1.0;
                }
            }
            connectivity.insert(*component_id, connections);
        }
        
        connectivity
    }

    pub fn get_component_neighbors(&self, component_id: &Uuid) -> Vec<&Component> {
        let mut neighbors = Vec::new();
        
        for relationship in self.relationships.values() {
            if relationship.source_id == *component_id {
                if let Some(target) = self.components.get(&relationship.target_id) {
                    neighbors.push(target);
                }
            } else if relationship.target_id == *component_id {
                if let Some(source) = self.components.get(&relationship.source_id) {
                    neighbors.push(source);
                }
            }
        }
        
        neighbors
    }

    pub fn update_component_state(&mut self, component_id: &Uuid, new_state: ComponentState) -> Result<()> {
        if let Some(component) = self.components.get_mut(component_id) {
            component.update_state(new_state);
            component.updated_at = Utc::now();
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(Error::ComponentNotFound(*component_id))
        }
    }

    pub fn get_relationships_between(&self, source_id: &Uuid, target_id: &Uuid) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|r| (r.source_id == *source_id && r.target_id == *target_id) ||
                       (r.source_id == *target_id && r.target_id == *source_id))
            .collect()
    }

    pub fn calculate_system_complexity(&self) -> f64 {
        let component_count = self.components.len() as f64;
        let relationship_count = self.relationships.len() as f64;
        
        if component_count == 0.0 {
            return 0.0;
        }
        
        // Calculate complexity as a ratio of relationships to possible relationships
        let max_possible_relationships = component_count * (component_count - 1.0) / 2.0;
        if max_possible_relationships == 0.0 {
            return 0.0;
        }
        
        relationship_count / max_possible_relationships
    }
}

impl Component {
    pub fn new(name: String, component_type: ComponentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            component_type,
            properties: HashMap::new(),
            state: ComponentState::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn type_name(&self) -> String {
        match &self.component_type {
            ComponentType::Node => "Node".to_string(),
            ComponentType::Agent => "Agent".to_string(),
            ComponentType::Process => "Process".to_string(),
            ComponentType::Resource => "Resource".to_string(),
            ComponentType::Interface => "Interface".to_string(),
            ComponentType::Custom(name) => name.clone(),
        }
    }

    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub fn update_state(&mut self, state: ComponentState) {
        self.state = state;
        self.updated_at = Utc::now();
    }

    pub fn timestamp(&self) -> Option<f32> {
        Some(self.created_at.timestamp() as f32)
    }
}

impl Relationship {
    pub fn new(source_id: Uuid, target_id: Uuid, relationship_type: RelationshipType) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id,
            target_id,
            relationship_type,
            properties: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn weight(&self) -> Option<f32> {
        self.properties.get("weight").and_then(|w| w.parse().ok())
    }
}