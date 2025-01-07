use std::sync::Arc;
use uuid::Uuid;
use crate::error::Result;
use crate::core::{Component, Relationship, SystemExt};
use super::View;
use crate::ui::{AppState, UIEvent};
use crate::ui::widgets::{Button, Checkbox, Slider};

pub struct SidebarView {
    state: Arc<AppState>,
    property_panel: PropertyPanel,
    filter_panel: FilterPanel,
}

struct PropertyPanel {
    title_button: Button,
    properties: Vec<PropertyField>,
    is_expanded: bool,
}

struct FilterPanel {
    title_button: Button,
    type_filters: Vec<Checkbox>,
    weight_range: Slider,
    date_range: (Slider, Slider),
    is_expanded: bool,
}

struct PropertyField {
    label: String,
    value: String,
    is_editable: bool,
}

impl SidebarView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state: Arc::clone(&state),
            property_panel: PropertyPanel {
                title_button: Button::new("Properties"),
                properties: Vec::new(),
                is_expanded: true,
            },
            filter_panel: FilterPanel {
                title_button: Button::new("Filters"),
                type_filters: Vec::new(),
                weight_range: Slider::new(0.0, 1.0),
                date_range: (
                    Slider::new(0.0, 100.0),
                    Slider::new(0.0, 100.0)
                ),
                is_expanded: true,
            },
        }
    }

    fn update_property_panel(&mut self) -> Result<()> {
        let selected = self.state.get_selected_components()?;
        self.property_panel.properties.clear();

        if selected.len() == 1 {
            let system = self.state.get_system();
            let system = system.read()?;
            if let Some(id) = selected.first() {
                if let Ok(uuid) = Uuid::parse_str(&*id) {
                    if let Some(component) = system.get_component(&uuid) {
                        self.add_component_properties(component)?;
                    }
                }
            }
        } else if selected.len() > 1 {
            let system = self.state.get_system();
            let system = system.read()?;
            let components: Vec<_> = selected.iter()
                .filter_map(|id| Uuid::parse_str(&*id).ok())
                .filter_map(|uuid| system.get_component(&uuid))
                .collect();
            if !components.is_empty() {
                self.add_common_properties(&components)?;
            }
        }
        Ok(())
    }

    fn add_component_properties(&mut self, component: &Component) -> Result<()> {
        self.property_panel.properties.push(PropertyField {
            label: "ID".to_string(),
            value: component.id().to_string(),
            is_editable: false,
        });

        self.property_panel.properties.push(PropertyField {
            label: "Type".to_string(),
            value: component.type_name().to_string(),
            is_editable: false,
        });

        // Add other component-specific properties
        for (key, value) in component.properties() {
            self.property_panel.properties.push(PropertyField {
                label: key.to_string(),
                value: value.to_string(),
                is_editable: true,
            });
        }

        Ok(())
    }

    fn add_common_properties(&mut self, components: &[&Component]) -> Result<()> {
        // Find and display properties common to all selected components
        if let Some(first) = components.first() {
            let common_props: Vec<_> = first.properties()
                .iter()
                .filter(|(key, value)| {
                    components[1..].iter().all(|c| {
                        c.properties().get(*key).map_or(false, |v| v == *value)
                    })
                })
                .collect();

            for (key, value) in common_props {
                self.property_panel.properties.push(PropertyField {
                    label: key.to_string(),
                    value: value.to_string(),
                    is_editable: true,
                });
            }
        }

        Ok(())
    }

    fn update_filter_panel(&mut self) -> Result<()> {
        let system = self.state.get_system();
        let system = system.read()?;

        // Update component type filters
        self.filter_panel.type_filters = system
            .component_types()
            .iter()
            .map(|t| {
                let mut checkbox = Checkbox::new(&t.to_string());
                checkbox.set_checked(true)?;
                Ok(checkbox)
            })
            .collect::<Result<Vec<_>>>()?;

        // Update range sliders based on system metadata
        if let Some((min, max)) = system.weight_range() {
            self.filter_panel.weight_range = Slider::new(min, max);
        }

        if let Some((min, max)) = system.date_range() {
            self.filter_panel.date_range = (
                Slider::new(min, max),
                Slider::new(min, max)
            );
        }

        Ok(())
    }
}

impl View for SidebarView {
    fn initialize(&mut self) -> Result<()> {
        // Initialize panels
        self.update_property_panel()?;
        self.update_filter_panel()?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        // Update panels based on current state
        self.update_property_panel()?;
        self.update_filter_panel()?;
        Ok(())
    }

    fn handle_event(&mut self, event: &UIEvent) -> Result<()> {
        match event {
            UIEvent::SelectionChanged(_) => {
                // Update property panel when selection changes
                self.update_property_panel()?;
            }
            _ => {}
        }
        Ok(())
    }
} 