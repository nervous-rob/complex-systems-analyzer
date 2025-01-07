use complex_systems_analyzer as csa;
use tracing::info;

#[tokio::main]
async fn main() -> csa::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Complex Systems Analyzer v{}", csa::VERSION);

    // Initialize the system
    let system_manager = csa::init().await?;

    info!("System initialized successfully");

    // TODO: Initialize UI and start event loop

    Ok(())
}
