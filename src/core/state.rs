use std::iter;
use std::sync::Arc;

use winit::event::WindowEvent;
use winit::window::Window;

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
    // Handles input-based camera movement
    // Bind group for camera
    #[allow(dead_code)]
    pub window: Arc<Window>, // Application window
    pub game_loop: Gameloop,
    pub scroll_y: i64,
    //temp solution
    //--TODO change
}

impl State {
    pub fn update_scroll(&mut self, y: i64) {
        self.scroll_y = y.max(0);
        // #[cfg(target_arch = "wasm32")]
        println!("{:?}", self.scroll_y)
        // log::info!("Scroll position - y: {}", y);
    }
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

        // Create uniform buffer for camera
        log::warn!("Camera");
        // Load shaders

        let mut game_loop = Gameloop::new(
            "Loop".to_string(),
            Arc::clone(&device),
            Arc::clone(&queue),
            size,
            surface_format,
        );

        let test = include_bytes!("../../src/test.vox");
        let castle = include_bytes!("../../src/castle.vox");
        let chr_knight = include_bytes!("../../src/chr_knight.vox");

        game_loop.voxel_helper.add_voxel(test);
        game_loop.voxel_helper.add_voxel(castle);
        game_loop.voxel_helper.add_voxel(chr_knight);

        log::warn!("Done");

        // Return initialized State
        Self {
            surface,
            surface_configured: false,
            device,
            queue,
            config,
            size,
            window,
            game_loop,
            scroll_y: 0,
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
            self.game_loop.camera_controller.camera.aspect =
                self.config.width as f32 / self.config.height as f32;
            // NEW!
            for instance_controller in self.game_loop.instance_controllers.iter_mut() {
                instance_controller.resize(new_size, &self.device);
            }
        } else {
            println!("Not configured");
            self.surface_configured = false;
        }
    }
    pub fn input(&mut self, event: &WindowEvent) {
        self.game_loop.process_event(event, &self.size);
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.game_loop.camera_controller.update_camera();
        self.queue.write_buffer(
            &self.game_loop.camera_controller.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.game_loop.camera_controller.camera_uniform]),
        );
        self.game_loop.update(dt, self.scroll_y);
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
            self.game_loop.render(&mut encoder, &view);
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
