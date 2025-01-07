use complex_systems_analyzer as csa;
use tracing::info;

#[tokio::main]
async fn main() -> csa::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Complex Systems Analyzer v{}", csa::VERSION);

    // Initialize the system
    let _system_manager = csa::init().await?;

    info!("System initialized successfully");

    // Initialize UI
    let ui_config = csa::ui::UIConfig::default();
    let mut app = csa::ui::App::new(ui_config)?;
    app.initialize()?;

    info!("UI initialized successfully");

    // Start event loop
    loop {
        app.update()?;
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }
}
