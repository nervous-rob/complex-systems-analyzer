use serde_json::Value;
use uuid::Uuid;
use std::collections::HashMap;

use crate::core::System;
use crate::core::types::{ComponentType, RelationshipType};
use crate::core::{Component, Relationship};
use crate::validation::{ValidationResult, ValidationError, ValidationMetrics, ValidationSeverity, ValidationContext};
use crate::error::Result;
use super::ImportFormat;

pub trait SystemImporter: Send + Sync {
    fn import_system(&self, data: &[u8]) -> Result<System>;
    fn validate_import(&self, data: &[u8]) -> Result<ValidationResult>;
    fn get_format(&self) -> ImportFormat;
}

pub struct JSONImporter;

impl JSONImporter {
    pub fn new() -> Self {
        Self
    }

    fn validate_json_structure(&self, value: &Value) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metrics: ValidationMetrics::default(),
        };

        let context = ValidationContext {
            system: None,
            component: None,
            relationship: None,
            metadata: HashMap::new(),
        };

        // Check required top-level fields
        let required_fields = ["id", "name", "description", "components", "relationships"];
        for field in required_fields {
            if !value.get(field).is_some() {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: format!("Missing required field: {}", field),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }

        // Validate components array
        if let Some(components) = value.get("components") {
            if let Some(components) = components.as_array() {
                for (i, component) in components.iter().enumerate() {
                    self.validate_component(component, i, &mut result);
                }
            } else {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: "'components' must be an array".to_string(),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }

        // Validate relationships array
        if let Some(relationships) = value.get("relationships") {
            if let Some(relationships) = relationships.as_array() {
                for (i, relationship) in relationships.iter().enumerate() {
                    self.validate_relationship(relationship, i, &mut result);
                }
            } else {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: "'relationships' must be an array".to_string(),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }

        result
    }

    fn validate_component(&self, component: &Value, index: usize, result: &mut ValidationResult) {
        let context = ValidationContext {
            system: None,
            component: None,
            relationship: None,
            metadata: HashMap::new(),
        };

        let required_fields = ["id", "name", "component_type"];
        for field in required_fields {
            if !component.get(field).is_some() {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: format!(
                        "Component at index {} is missing required field: {}",
                        index, field
                    ),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }

        // Validate component type
        if let Some(type_str) = component.get("component_type").and_then(|v| v.as_str()) {
            if !["Node", "Interface", "Custom"].contains(&type_str) {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: format!(
                        "Component at index {} has invalid type: {}",
                        index, type_str
                    ),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }
    }

    fn validate_relationship(
        &self,
        relationship: &Value,
        index: usize,
        result: &mut ValidationResult,
    ) {
        let context = ValidationContext {
            system: None,
            component: None,
            relationship: None,
            metadata: HashMap::new(),
        };

        let required_fields = ["id", "source_id", "target_id", "relationship_type"];
        for field in required_fields {
            if !relationship.get(field).is_some() {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: format!(
                        "Relationship at index {} is missing required field: {}",
                        index, field
                    ),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }

        // Validate relationship type
        if let Some(type_str) = relationship.get("relationship_type").and_then(|v| v.as_str()) {
            if !["Dependency", "Composition", "Association", "Custom"].contains(&type_str) {
                result.errors.push(ValidationError {
                    rule_id: Uuid::new_v4(),
                    message: format!(
                        "Relationship at index {} has invalid type: {}",
                        index, type_str
                    ),
                    severity: ValidationSeverity::Error,
                    context: context.clone(),
                });
                result.is_valid = false;
            }
        }
    }
}

impl SystemImporter for JSONImporter {
    fn import_system(&self, data: &[u8]) -> Result<System> {
        let value: Value = serde_json::from_slice(data)?;
        
        // Create base system
        let mut system = System::new(
            value["name"].as_str().unwrap_or("Imported System").to_string(),
            value["description"]
                .as_str()
                .unwrap_or("Imported from JSON")
                .to_string(),
        );

        // Import components
        if let Some(components) = value["components"].as_array() {
            for component_value in components {
                let name = component_value["name"]
                    .as_str()
                    .unwrap_or("Unnamed Component")
                    .to_string();

                let component_type = match component_value["component_type"].as_str() {
                    Some("Node") => ComponentType::Node,
                    Some("Interface") => ComponentType::Interface,
                    Some(custom) => ComponentType::Custom(custom.to_string()),
                    _ => ComponentType::Node,
                };

                let mut component = Component::new(name, component_type);

                // Add properties
                if let Some(props) = component_value.get("properties") {
                    if let Some(obj) = props.as_object() {
                        for (key, value) in obj {
                            if let Some(value_str) = value.as_str() {
                                component.properties.insert(key.clone(), value_str.to_string());
                            }
                        }
                    }
                }

                system.add_component(component)?;
            }
        }

        // Import relationships
        if let Some(relationships) = value["relationships"].as_array() {
            for relationship_value in relationships {
                let source_id = Uuid::parse_str(
                    relationship_value["source_id"]
                        .as_str()
                        .unwrap_or_default()
                )?;
                let target_id = Uuid::parse_str(
                    relationship_value["target_id"]
                        .as_str()
                        .unwrap_or_default()
                )?;

                let relationship_type = match relationship_value["relationship_type"].as_str() {
                    Some("Dependency") => RelationshipType::Dependency,
                    Some("Composition") => RelationshipType::Composition,
                    Some("Association") => RelationshipType::Association,
                    Some(custom) => RelationshipType::Custom(custom.to_string()),
                    _ => RelationshipType::Dependency,
                };

                let mut relationship = Relationship::new(source_id, target_id, relationship_type);

                // Add properties
                if let Some(props) = relationship_value.get("properties") {
                    if let Some(obj) = props.as_object() {
                        for (key, value) in obj {
                            if let Some(value_str) = value.as_str() {
                                relationship.properties.insert(key.clone(), value_str.to_string());
                            }
                        }
                    }
                }

                system.add_relationship(relationship)?;
            }
        }

        Ok(system)
    }

    fn validate_import(&self, data: &[u8]) -> Result<ValidationResult> {
        let value: Value = serde_json::from_slice(data)?;
        Ok(self.validate_json_structure(&value))
    }

    fn get_format(&self) -> ImportFormat {
        ImportFormat::JSON
    }
} 