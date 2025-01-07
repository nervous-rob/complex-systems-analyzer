use std::sync::Arc;
use crate::error::{Result, Error};
use crate::core::System;
use super::{AppState, UIConfig, UIEvent, UICommand, CommandResponse, views::ViewManager};
use super::views::{ToolbarView, AnalysisView};
use winit::event_loop::EventLoop;
use crate::visualization::Visualization;
use crate::visualization::renderer::Renderer;
use winit::window::WindowBuilder;

pub struct App {
    state: Arc<AppState>,
    bridge: super::UIBridge,
    view_manager: ViewManager,
    event_loop: Option<EventLoop<()>>,
    visualization: Option<Visualization>,
}

impl App {
    pub fn new(config: UIConfig, event_loop: EventLoop<()>) -> Result<Self> {
        let state = Arc::new(AppState::new(config.clone()));
        let bridge = super::UIBridge::new(config);
        let view_manager = ViewManager::new(Arc::clone(&state));

        Ok(Self {
            state,
            bridge,
            view_manager,
            event_loop: Some(event_loop),
            visualization: None,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        // Initialize views
        self.setup_views()?;
        
        // Initialize event handlers
        self.setup_event_handlers()?;
        
        // Create window
        let event_loop = self.event_loop.as_ref().ok_or_else(|| Error::system("Event loop not initialized"))?;
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Complex Systems Analyzer")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
                .build(event_loop)
                .map_err(|e| Error::system(e.to_string()))?
        );
        
        // Initialize visualization
        let vis = self.state.get_visualization();
        let mut vis_guard = vis.write().map_err(|_| Error::system("Failed to lock visualization"))?;
        vis_guard.initialize(window)?;
        
        // Ensure initial render
        if let Some(renderer) = &mut vis_guard.renderer {
            renderer.render()?;
        }
        
        drop(vis_guard);

        Ok(())
    }

    pub fn load_system(&mut self, system: &System) -> Result<()> {
        // Update state with new system
        let sys = self.state.get_system();
        let mut sys_guard = sys.write().map_err(|_| Error::system("Failed to lock system"))?;
        *sys_guard = system.clone();
        drop(sys_guard);
        
        // Update visualization
        let vis = self.state.get_visualization();
        let mut vis_guard = vis.write().map_err(|_| Error::system("Failed to lock visualization"))?;
        vis_guard.update_graph(system)?;
        vis_guard.fit_to_view()?;
        drop(vis_guard);
        
        // Update UI
        self.bridge.handle_command(UICommand::UpdateComponent(super::ComponentUpdate {
            id: String::new(),
            properties: serde_json::json!({
                "type": "graph_update",
                "data": {
                    "node_count": system.components.len(),
                }
            }),
        }))?;
        
        Ok(())
    }

    fn setup_views(&mut self) -> Result<()> {
        // Initialize all views
        self.view_manager.initialize()?;
        
        Ok(())
    }

    fn setup_event_handlers(&mut self) -> Result<()> {
        // Register event handlers that don't access visualization directly
        self.bridge.register_callback(UIEvent::GraphUpdated, Box::new(|_| {
            // Handle graph updates through command system
            ()
        }))?;

        self.bridge.register_callback(UIEvent::MenuAction(super::MenuAction::FitToView), Box::new(|_| {
            // Handle menu actions through command system
            ()
        }))?;

        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        // Take ownership of the event loop
        let event_loop = self.event_loop.take()
            .ok_or_else(|| Error::system("Event loop not initialized"))?;

        // Get the visualization engine and take ownership of the renderer
        let vis = self.state.get_visualization();
        let mut vis_guard = vis.write().map_err(|_| Error::system("Failed to lock visualization"))?;
        let renderer = vis_guard.take_renderer()
            .ok_or_else(|| Error::system("Renderer not initialized"))?;
        drop(vis_guard);
        
        // Run the renderer with the event loop
        run_with_event_loop(event_loop, renderer)
    }
}

fn run_with_event_loop(event_loop: EventLoop<()>, mut renderer: Renderer) -> Result<()> {
    use winit::event_loop::ControlFlow;
    use winit::event::{Event, WindowEvent};

    event_loop.run(move |event, window_target| {
        window_target.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent { window_id, event } if window_id == renderer.window().id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        match renderer.render() {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Render error: {}", e);
                                // Reconfigure the surface if lost
                                if e.to_string().contains("surface has changed") {
                                    renderer.resize(renderer.window().inner_size());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                renderer.window().request_redraw();
            }
            _ => {}
        }
    }).map_err(|e| Error::system(e.to_string()))
} 