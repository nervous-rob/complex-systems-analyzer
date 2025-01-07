use complex_systems_analyzer as csa;
use tracing::info;
use csa::ui::{UIConfig, App};
use csa::{Component, ComponentType, System, Relationship, RelationshipType};
use winit::event_loop::EventLoop;

#[tokio::main]
async fn main() -> csa::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Complex Systems Analyzer v{}", csa::VERSION);

    // Initialize the system
    let system_manager = csa::init().await?;

    info!("System initialized successfully");

    // Create a test system
    let mut system = System::new(
        "Test System".to_string(),
        "A test system for visualization".to_string(),
    );

    // Add some test components in a grid pattern
    let mut components = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            let component = Component::new(
                format!("Node {},{}", i, j),
                ComponentType::Node,
            );
            system.add_component(component.clone())?;
            components.push(component);
        }
    }

    // Add some test relationships
    for i in 0..components.len() {
        for j in (i+1)..components.len() {
            if i % 2 == 0 && j % 2 == 0 {
                let relationship = Relationship::new(
                    components[i].id,
                    components[j].id,
                    RelationshipType::Dependency,
                );
                system.add_relationship(relationship)?;
            }
        }
    }

    // Initialize UI with custom config
    let ui_config = UIConfig {
        window_size: (1280, 720),
        theme: csa::ui::Theme::Dark,
        layout: csa::ui::LayoutConfig::default(),
        window_title: "Complex Systems Analyzer - Test Visualization".to_string(),
    };

    // Create and run the application
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new(ui_config, event_loop)?;
    app.initialize()?;
    app.load_system(&system)?;
    
    info!("Application initialized successfully");
    
    // Run the application - this will block until the window is closed
    app.run()?;

    Ok(())
}
