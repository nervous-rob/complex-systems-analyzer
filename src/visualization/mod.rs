mod renderer;

use winit::{
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    event::*,
};
use crate::error::{Result, Error};
use crate::core::System;
use crate::ui::LayoutConfig;

pub struct VisualizationEngine {
    layout_config: LayoutConfig,
    initialized: bool,
    event_loop: Option<EventLoop<()>>,
    renderer: Option<renderer::Renderer>,
}

impl VisualizationEngine {
    pub fn new(config: LayoutConfig) -> Self {
        Self {
            layout_config: config,
            initialized: false,
            event_loop: None,
            renderer: None,
        }
    }

    pub fn get_layout_config(&self) -> &LayoutConfig {
        &self.layout_config
    }

    pub fn initialize(&mut self) -> Result<()> {
        let event_loop = EventLoop::new().map_err(|e| Error::system(e.to_string()))?;
        let window = std::sync::Arc::new(
            WindowBuilder::new()
                .with_title("Complex Systems Analyzer")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
                .build(&event_loop)
                .map_err(|e| Error::system(e.to_string()))?
        );

        let renderer = pollster::block_on(renderer::Renderer::new(window.clone()))?;
        
        self.renderer = Some(renderer);
        self.event_loop = Some(event_loop);
        self.initialized = true;
        Ok(())
    }

    pub fn update_graph(&mut self, _system: &System) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }

    pub fn update_layout(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }

    pub fn update_selection(&mut self, _selected_ids: &[String]) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }

    pub fn update_viewport(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }

    pub fn render_frame(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        if let Some(renderer) = &mut self.renderer {
            renderer.render()?;
        }
        
        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        let event_loop = self.event_loop.take().expect("Event loop not initialized");
        let mut renderer = self.renderer.take().expect("Renderer not initialized");

        event_loop.run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { window_id, event } if window_id == renderer.window().id() => {
                    match event {
                        WindowEvent::CloseRequested => window_target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            if let Err(e) = renderer.render() {
                                eprintln!("Render error: {:?}", e);
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
        }).map_err(|e| Error::system(e.to_string()))?;

        Ok(())
    }

    pub fn zoom_in(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }

    pub fn zoom_out(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }

    pub fn fit_view(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        Ok(())
    }
} 