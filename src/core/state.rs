use std::collections::HashMap;
use std::iter;
use std::sync::Arc;

use cgmath::Vector2;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::core::game_loop::Chunk;
use crate::entity::entity::{
    instances_list, instances_list_circle, make_cube_primitive, make_cube_textured,
    InstanceController, Mesh,
};
use crate::entity::primitive_texture::PrimitiveTexture;
use crate::entity::texture::Texture;

use super::camera::{Camera, CameraController, CameraUniform};
use super::game_loop::Gameloop;
// The main application state holding all GPU resources and game logic
pub struct State {
    pub surface: wgpu::Surface<'static>,     // GPU rendering surface
    pub surface_configured: bool,            // Tracks if surface is configured
    pub device: Arc<wgpu::Device>,           // Logical GPU device
    pub queue: Arc<wgpu::Queue>,             // Command queue for GPU
    pub config: wgpu::SurfaceConfiguration,  // Surface configuration settings
    pub size: winit::dpi::PhysicalSize<u32>, // Window size
    #[allow(dead_code)]
    pub camera: Camera, // Camera object
    pub camera_controller: CameraController, // Handles input-based camera movement
    pub camera_uniform: CameraUniform,       // Uniform buffer for camera
    pub camera_buffer: wgpu::Buffer,         // GPU buffer for camera data
    pub camera_bind_group: wgpu::BindGroup,  // Bind group for camera
    #[allow(dead_code)]
    pub depth_texture: Texture,
    pub depth_texture_primitive: PrimitiveTexture,
    pub window: Arc<Window>, // Application window
    pub game_loop: Gameloop,
    //temp solution
    //--TODO change
    pub chunk_size: Vector2<u32>,
    pub mesh: Mesh, // Game logic loop
}

impl State {
    // Creates a new State object, initializing all required resources
    pub async fn new(window: Arc<Window>) -> State {
        let size = window.inner_size();

        // Create a new GPU instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // Create surface linked to window
        let surface = instance.create_surface(window.clone()).unwrap();

        // Select appropriate GPU adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::warn!("{:?}", adapter.get_info());

        // Request device and queue from adapter
        let (tdevice, tqueue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits {
                        max_texture_dimension_1d: 4096,
                        max_texture_dimension_2d: 4096,
                        ..wgpu::Limits::downlevel_webgl2_defaults()
                    }
                } else {
                    wgpu::Limits::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let device = Arc::new(tdevice);
        let queue = Arc::new(tqueue);

        log::warn!("Surface");

        // Get surface capabilities and select preferred format
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Configure surface
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

        // Setup camera
        let camera = Camera {
            eye: (-18.0, 23.0, -18.0).into(),
            target: (15.0, 0.0, 15.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 20.0,
            znear: 0.1,
            zfar: 1.0,
        };
        let camera_controller = CameraController::new(0.2);
        log::warn!("Camera");

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // Create uniform buffer for camera
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create layout and bind group for camera
        let camera_bind_group_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        log::warn!("Shader");

        // Load shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        // Load shaders
        let primitive_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PrimitiveShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/primitive.wgsl").into()),
        });

        // Create depth texture for texture meshes
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let depth_texture_primitive =
            PrimitiveTexture::create_depth_texture(&device, &config, "depth_texture_prim");
        // Create depth texture for primitive

        log::warn!("Pipeline");

        // Create render pipeline

        // Create instance controller and game loop

        let chunk_size = Vector2::new(35, 35);
        let mut chunk_map: HashMap<Chunk, InstanceController> = HashMap::new();
        let mesh = make_cube_primitive();
        match mesh {
            Mesh::Primitive(_) => {
                for n in 0..1 {
                    for y in 0..1 {
                        let origin = Chunk { x: n, y: y };
                        let mesh = make_cube_primitive();
                        let (mb, renderer) = mesh.get_mesh_buffer(
                            &device,
                            &primitive_shader,
                            surface_format,
                            &queue,
                            camera_bind_group_layout.clone(),
                        );
                        let instance_controller = InstanceController::new(
                            instances_list_circle(origin, chunk_size),
                            0,
                            mb,
                            renderer,
                            &device,
                        );
                        chunk_map.insert(origin, instance_controller);
                    }
                }
            }
            Mesh::Textured(_) => {
                for n in 0..3 {
                    for y in 0..3 {
                        let origin = Chunk { x: n, y: y };
                        let mesh = make_cube_textured();
                        let (mb, renderer) = mesh.get_mesh_buffer(
                            &device,
                            &shader,
                            surface_format,
                            &queue,
                            camera_bind_group_layout.clone(),
                        );
                        let instance_controller = InstanceController::new(
                            instances_list(origin, chunk_size),
                            0,
                            mb,
                            renderer,
                            &device,
                        );
                        // let instance_controller2 = InstanceController::new(instances_list2(), 0, make_cube(&device), &device);
                        chunk_map.insert(origin, instance_controller);
                    }
                }
            }
        }

        let game_loop = Gameloop::new(
            "Loop".to_string(),
            PhysicalPosition::new(0.0, 0.0),
            Arc::clone(&device),
            Arc::clone(&queue),
            chunk_size,
            chunk_map,
        );
        log::warn!("Done");

        // Return initialized State
        Self {
            surface,
            surface_configured: false,
            device,
            queue,
            config,
            size,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            depth_texture,
            depth_texture_primitive,
            window,
            game_loop,
            chunk_size,
            mesh,
        }
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.surface_configured = true;
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;
            // NEW!
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.depth_texture_primitive = PrimitiveTexture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture_primitive",
            );
        } else {
            println!("Not configured");
            self.surface_configured = false;
        }
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.game_loop
            .process_event(event, &self.camera, &self.size);
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.game_loop.update(dt);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.surface_configured {
            return Ok(());
        }
        self.window.request_redraw();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: {
                    match self.mesh {
                        Mesh::Primitive(_) => Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_primitive.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0), // Clear depth buffer to far plane
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        Mesh::Textured(_) => Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                    }
                },
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            for instance_controller in self.game_loop.chunk_map.values_mut() {
                instance_controller.render(&mut render_pass);
            }
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
