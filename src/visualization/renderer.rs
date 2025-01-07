use wgpu::{Instance, Device, Queue, Surface, SurfaceConfiguration, Buffer};
use winit::window::Window;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;
use crate::error::{Result, Error};
use crate::core::System;
use std::collections::HashMap;
use uuid::Uuid;
use glam;

#[derive(Debug)]
pub struct Camera {
    position: [f32; 2],
    zoom: f32,
    view: glam::Mat4,
    projection: glam::Mat4,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let mut camera = Self {
            position: [0.0, 0.0],
            zoom: 1.0,
            view: glam::Mat4::IDENTITY,
            projection: glam::Mat4::IDENTITY,
        };
        camera.update_matrices(width, height);
        camera
    }

    pub fn set_target(&mut self, position: [f32; 2]) {
        self.position = position;
    }

    pub fn update_matrices(&mut self, width: f32, height: f32) {
        // Create view matrix (camera transform)
        self.view = glam::Mat4::from_translation(glam::Vec3::new(-self.position[0], -self.position[1], 0.0))
            * glam::Mat4::from_scale(glam::Vec3::splat(self.zoom));

        // Create orthographic projection matrix
        let aspect = width / height;
        let scale = 1.0 / self.zoom;
        self.projection = glam::Mat4::orthographic_rh(
            -aspect * scale,
            aspect * scale,
            -scale,
            scale,
            -1.0,
            1.0,
        );
    }
}

#[derive(Debug)]
pub struct Viewport {
    width: f32,
    height: f32,
    scale_factor: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    pub fn screen_to_world(&self, screen_pos: [f32; 2]) -> [f32; 2] {
        [
            (screen_pos[0] - self.width / 2.0) / self.scale_factor,
            (screen_pos[1] - self.height / 2.0) / self.scale_factor,
        ]
    }

    pub fn world_to_screen(&self, world_pos: [f32; 2]) -> [f32; 2] {
        [
            world_pos[0] * self.scale_factor + self.width / 2.0,
            world_pos[1] * self.scale_factor + self.height / 2.0,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: Uuid,
    pub position: [f32; 2],
    pub size: f32,
    pub color: [f32; 4],
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub weight: f32,
    pub color: [f32; 4],
}

pub struct Renderer {
    instance: Instance,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    
    // Render pipeline and bindings
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    uniform_bind_group: wgpu::BindGroup,
    
    // Graph data
    nodes: Vec<NodeData>,
    edges: Vec<EdgeData>,
    selected_nodes: Vec<Uuid>,
    highlighted_nodes: Vec<Uuid>,
    
    // Camera and viewport
    camera: Camera,
    viewport: Viewport,
    
    // Current system reference
    current_system: Option<Arc<System>>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        println!("Creating new renderer...");
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });
        println!("Created wgpu instance");
        
        // Create surface using the window reference
        let surface = match instance.create_surface(window.clone()) {
            Ok(s) => {
                println!("Successfully created surface");
                s
            },
            Err(e) => {
                println!("Failed to create surface: {:?}", e);
                return Err(Error::system(format!("Failed to create surface: {:?}", e)));
            }
        };
        
        println!("Requesting adapter...");
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance, // Prefer high performance for visualization
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| {
            println!("Failed to create GPU adapter");
            Error::system("Failed to create GPU adapter")
        })?;
        println!("Successfully created adapter");
        
        println!("Requesting device...");
        // Request device with specific features and limits for graph visualization
        let (device, queue) = match adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Graph Renderer Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: 1024 * 1024 * 1024, // 1GB for large graphs
                    ..Default::default()
                },
            },
            None,
        ).await {
            Ok((d, q)) => {
                println!("Successfully created device and queue");
                (d, q)
            },
            Err(e) => {
                println!("Failed to create device: {:?}", e);
                return Err(Error::system(format!("Failed to create device: {:?}", e)));
            }
        };
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
            
        println!("Configuring surface...");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        println!("Successfully configured surface");
        
        println!("Creating render pipeline...");
        // Initialize render pipeline and resources
        let render_pipeline = create_render_pipeline(&device, &config);
        let (vertex_buffer, index_buffer) = create_buffers(&device);
        let (uniform_buffer, uniform_bind_group) = create_uniform_bindings(&device);
        println!("Successfully created render pipeline and resources");
        
        Ok(Self {
            instance,
            device,
            queue,
            surface,
            config,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            nodes: Vec::new(),
            edges: Vec::new(),
            selected_nodes: Vec::new(),
            highlighted_nodes: Vec::new(),
            camera: Camera::new(size.width as f32, size.height as f32),
            viewport: Viewport::new(size.width as f32, size.height as f32),
            current_system: None,
        })
    }
    
    pub fn update_graph_data(&mut self, nodes: Vec<NodeData>, edges: Vec<EdgeData>) -> Result<()> {
        self.nodes = nodes;
        self.edges = edges;
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn apply_force_directed_layout(
        &mut self,
        force_strength: f32,
        link_distance: f32,
        charge_strength: f32
    ) -> Result<()> {
        // Implement force-directed layout algorithm
        let mut positions = self.nodes.iter().map(|n| n.position).collect::<Vec<_>>();
        
        for _ in 0..100 {
            // Apply repulsive forces between nodes
            for i in 0..positions.len() {
                for j in (i + 1)..positions.len() {
                    let dx = positions[j][0] - positions[i][0];
                    let dy = positions[j][1] - positions[i][1];
                    let distance = (dx * dx + dy * dy).sqrt().max(0.1);
                    
                    let force = charge_strength / (distance * distance);
                    let fx = dx * force / distance;
                    let fy = dy * force / distance;
                    
                    positions[i][0] -= fx;
                    positions[i][1] -= fy;
                    positions[j][0] += fx;
                    positions[j][1] += fy;
                }
            }
            
            // Apply attractive forces along edges
            for edge in &self.edges {
                if let (Some(source_idx), Some(target_idx)) = (
                    self.nodes.iter().position(|n| n.id == edge.source_id),
                    self.nodes.iter().position(|n| n.id == edge.target_id)
                ) {
                    let dx = positions[target_idx][0] - positions[source_idx][0];
                    let dy = positions[target_idx][1] - positions[source_idx][1];
                    let distance = (dx * dx + dy * dy).sqrt().max(0.1);
                    
                    let force = (distance - link_distance) * force_strength;
                    let fx = dx * force / distance;
                    let fy = dy * force / distance;
                    
                    positions[source_idx][0] += fx;
                    positions[source_idx][1] += fy;
                    positions[target_idx][0] -= fx;
                    positions[target_idx][1] -= fy;
                }
            }
        }
        
        // Update node positions
        for (node, position) in self.nodes.iter_mut().zip(positions.iter()) {
            node.position = *position;
        }
        
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn update_selection(&mut self, selected_ids: &[Uuid]) -> Result<()> {
        self.selected_nodes = selected_ids.to_vec();
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn highlight_nodes(&mut self, node_ids: &[Uuid]) -> Result<()> {
        self.highlighted_nodes = node_ids.to_vec();
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn pick_node_at_position(&self, mouse_pos: [f32; 2]) -> Result<Option<Uuid>> {
        for node in &self.nodes {
            let dx = mouse_pos[0] - node.position[0];
            let dy = mouse_pos[1] - node.position[1];
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance <= node.size {
                return Ok(Some(node.id));
            }
        }
        Ok(None)
    }
    
    pub fn focus_node(&mut self, node_id: Uuid) -> Result<()> {
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            self.camera.set_target(node.position);
            self.update_camera_matrices()?;
        }
        Ok(())
    }
    
    pub fn highlight_by_value(&mut self, values: HashMap<Uuid, f64>) -> Result<()> {
        let max_value = values.values().fold(0.0_f64, |acc, &x| acc.max(x));
        
        for node in &mut self.nodes {
            if let Some(&value) = values.get(&node.id) {
                let intensity = (value / max_value) as f32;
                node.color = [intensity, intensity, 1.0, 1.0];
            }
        }
        
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn set_global_highlight(&mut self, intensity: f64) -> Result<()> {
        let intensity = intensity as f32;
        for node in &mut self.nodes {
            node.color = [1.0, intensity, intensity, 1.0];
        }
        
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn clear_highlights(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            node.color = [0.5, 0.5, 0.5, 1.0];
        }
        
        self.update_buffers()?;
        Ok(())
    }
    
    pub fn get_current_system(&self) -> Option<&System> {
        self.current_system.as_ref().map(|s| s.as_ref())
    }

    fn update_buffers(&mut self) -> Result<()> {
        // Create vertices for nodes (6 vertices per node for a quad)
        let mut vertices = Vec::with_capacity(self.nodes.len() * 6);
        let mut indices = Vec::with_capacity(self.nodes.len() * 6);
        
        for (i, node) in self.nodes.iter().enumerate() {
            // Add vertices for a quad centered at node position
            let base_index = i * 4;
            
            // Top-left
            vertices.push(Vertex {
                position: [node.position[0] - 0.5, node.position[1] - 0.5],
                color: node.color,
                size: node.size,
            });
            // Top-right
            vertices.push(Vertex {
                position: [node.position[0] + 0.5, node.position[1] - 0.5],
                color: node.color,
                size: node.size,
            });
            // Bottom-right
            vertices.push(Vertex {
                position: [node.position[0] + 0.5, node.position[1] + 0.5],
                color: node.color,
                size: node.size,
            });
            // Bottom-left
            vertices.push(Vertex {
                position: [node.position[0] - 0.5, node.position[1] + 0.5],
                color: node.color,
                size: node.size,
            });
            
            // Add indices for two triangles
            indices.extend_from_slice(&[
                base_index as u32,
                base_index as u32 + 1,
                base_index as u32 + 2,
                base_index as u32,
                base_index as u32 + 2,
                base_index as u32 + 3,
            ]);
        }
        
        // Update vertex buffer
        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
        
        // Update index buffer
        self.queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&indices),
        );
        
        Ok(())
    }

    pub fn update_camera_matrices(&mut self) -> Result<()> {
        self.camera.update_matrices(
            self.viewport.width,
            self.viewport.height
        );
        self.update_viewport_uniforms()?;
        Ok(())
    }

    pub fn update_viewport_uniforms(&mut self) -> Result<()> {
        let uniforms = Uniforms::new(self.camera.view, self.camera.projection);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.camera.update_matrices(new_size.width as f32, new_size.height as f32);
            self.viewport.resize(new_size.width as f32, new_size.height as f32);
            self.update_viewport_uniforms().unwrap_or_default();
        }
    }

    pub fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            
            // Draw nodes and edges
            if !self.nodes.is_empty() {
                let num_indices = (self.nodes.len() * 6) as u32;
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn zoom(&mut self, factor: f32) -> Result<()> {
        self.camera.zoom *= factor;
        self.camera.update_matrices(
            self.viewport.width,
            self.viewport.height
        );
        Ok(())
    }

    pub fn fit_to_view(&mut self) -> Result<()> {
        if self.nodes.is_empty() {
            return Ok(());
        }

        // Calculate bounds
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in &self.nodes {
            let half_size = node.size / 2.0;
            min_x = min_x.min(node.position[0] - half_size);
            min_y = min_y.min(node.position[1] - half_size);
            max_x = max_x.max(node.position[0] + half_size);
            max_y = max_y.max(node.position[1] + half_size);
        }

        // Calculate center and size
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let width = max_x - min_x;
        let height = max_y - min_y;

        // Set camera position and zoom
        self.camera.position = [center_x, center_y];
        self.camera.zoom = 0.9 * (self.viewport.width / width).min(self.viewport.height / height);
        
        self.camera.update_matrices(
            self.viewport.width,
            self.viewport.height
        );
        Ok(())
    }

    pub fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        // Add some test nodes to verify rendering
        let test_nodes = vec![
            NodeData {
                id: Uuid::new_v4(),
                position: [-100.0, 0.0],
                size: 20.0,
                color: [1.0, 0.2, 0.2, 1.0],  // Bright red
                label: "Test Node 1".to_string(),
            },
            NodeData {
                id: Uuid::new_v4(),
                position: [100.0, 0.0],
                size: 20.0,
                color: [0.2, 1.0, 0.2, 1.0],  // Bright green
                label: "Test Node 2".to_string(),
            },
        ];
        
        self.update_graph_data(test_nodes, vec![])?;
        
        // Set initial camera position and zoom
        self.camera.position = [0.0, 0.0];
        self.camera.zoom = 0.5;  // Zoom out to see both nodes
        self.update_viewport_uniforms()?;
        
        Ok(())
    }
}

const VERTEX_SHADER: &str = r#"
struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) size: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = vec4<f32>(in.position * in.size, 0.0, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.color = in.color;
    return out;
}
"#;

const FRAGMENT_SHADER: &str = r#"
@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
    size: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

impl Uniforms {
    fn new(view: glam::Mat4, projection: glam::Mat4) -> Self {
        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
        }
    }
}

fn create_render_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(FRAGMENT_SHADER.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: 8,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: 24,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

fn create_buffers(device: &Device) -> (Buffer, Buffer) {
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: 1024 * std::mem::size_of::<Vertex>() as u64, // Space for 1024 vertices
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Index Buffer"),
        size: 1024 * std::mem::size_of::<u32>() as u64, // Space for 1024 indices
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (vertex_buffer, index_buffer)
}

fn create_uniform_bindings(device: &Device) -> (Buffer, wgpu::BindGroup) {
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });

    (uniform_buffer, bind_group)
} 