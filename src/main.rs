use complex_systems_analyzer as csa;
use tracing::info;
use csa::ui::UIConfig;

#[tokio::main]
async fn main() -> csa::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Complex Systems Analyzer v{}", csa::VERSION);

    // Initialize the system
    let _system_manager = csa::init().await?;

    info!("System initialized successfully");

    // Initialize UI
    let ui_config = UIConfig::default();
    let mut visualization = csa::visualization::VisualizationEngine::new(ui_config.layout.clone());
    visualization.initialize()?;

    info!("Visualization initialized successfully");

    // Start event loop
    visualization.run()?;

    Ok(())
}
