use std::{collections::BTreeMap, sync::Arc};

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector2, Vector3};
use log::warn;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    core::camera::CameraController,
    entity::{
        entity::{
            instance_cube, instances_list, instances_list_cube, make_cube_primitive,
            make_cube_textured, InstanceController, Light, MeshType,
        },
        primitive_texture::PrimitiveTexture,
        texture::Texture,
    },
    helpers::{
        animation::{
            AnimationHandler, AnimationPersistent, AnimationStep, AnimationTransition,
            AnimationType, EaseInEaseOutLoop, EaseOut,
        },
        line_trace::{aabb_sphere_intersect, line_trace},
        transition::{CameraPositions, TransitionHandler, VoxelObjects},
        voxel_builder::VoxelHandler,
    },
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
}

pub struct Gameloop {
    pub name: String,
    pub cursor_position: PhysicalPosition<f32>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub light_source: Light,
    pub instance_controllers: Vec<InstanceController>,
    pub camera_controller: CameraController,
    pub elapsed_time: f32,
    pub animation_handler: AnimationHandler,
    pub voxel_helper: VoxelHandler<VoxelObjects>,
    pub transition_handler: TransitionHandler<VoxelObjects>,
    pub camera_transition_handler: TransitionHandler<CameraPositions>,
}

impl Gameloop {
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        depth_texture: Texture,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
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
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                })
            },
            // depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        for instance_controller in self.instance_controllers.iter_mut() {
            instance_controller.render(
                &mut render_pass,
                self.camera_controller.camera_bind_group.clone(),
            );
        }
    }
    pub fn update(&mut self, dt: std::time::Duration, scroll_y: i64) {
        self.light_source.position = self.camera_controller.camera.eye.to_vec();
        let light_data = self.light_source.to_raw();
        self.queue.write_buffer(
            &self.light_source.light_buffer,
            0,
            bytemuck::cast_slice(&[light_data]),
        );

        let dts = dt.as_secs_f32();
        for instance_controller in self.instance_controllers.iter_mut() {
            self.animation_handler.animate(dts);
            self.camera_controller.animate_camera(dts);

            for (i, instance) in instance_controller.instances.iter_mut().enumerate() {
                self.animation_handler.update_instance(i, instance);
            }
            if let Some(transition) = self.camera_transition_handler.trigger_transition(scroll_y) {
                match transition.clone() {
                    CameraPositions::Home(position) => {
                        self.camera_controller.add_animation(position);
                    }
                    CameraPositions::Rust(position) => {
                        self.camera_controller.add_animation(position);
                    }
                    CameraPositions::CPlusPLus(position) => {
                        self.camera_controller.add_animation(position);
                    }
                    CameraPositions::CSharp(position) => {
                        self.camera_controller.add_animation(position);
                    }
                }
            }
            if let Some(transition) = self.transition_handler.trigger_transition(scroll_y) {
                match transition.clone() {
                    VoxelObjects::Home => {
                        self.camera_controller.auto = true;
                        self.camera_controller.speed = 0.4;
                        self.camera_controller.is_right_pressed = true;
                    }
                    _ => {
                        if self.camera_controller.auto {
                            self.camera_controller.auto = false;
                            self.camera_controller.speed = 1.0;
                            self.camera_controller.is_right_pressed = false;
                        }
                        self.animation_handler
                            .reset_instance_position_to_current_position(
                                &mut instance_controller.instances,
                            );
                        self.voxel_helper
                            .transition_to_object(transition, &mut self.animation_handler);
                    }

                    _ => {}
                }
            }
            instance_controller.update_buffer_multithreaded(Arc::clone(&self.queue));

            self.elapsed_time += dts;
            if self.camera_controller.auto {
                // if self.elapsed_time < 2.0 {
                //     return;
                // }
                let animation_time = self.elapsed_time % 24.0;
                let ready = !self.animation_handler.is_locked();
                if animation_time.floor() == 0.0 && ready {
                    self.animation_handler
                        .reset_instance_position_to_current_position(
                            &mut instance_controller.instances,
                        );
                    self.voxel_helper
                        .transition_to_object(VoxelObjects::Buttplug, &mut self.animation_handler);
                }
                if animation_time.floor() == 8.0 && ready {
                    self.animation_handler
                        .reset_instance_position_to_current_position(
                            &mut instance_controller.instances,
                        );
                    self.voxel_helper
                        .transition_to_object(VoxelObjects::Viking, &mut self.animation_handler);
                }
                if animation_time.floor() == 16.0 && ready {
                    self.animation_handler
                        .reset_instance_position_to_current_position(
                            &mut instance_controller.instances,
                        );
                    self.voxel_helper
                        .transition_to_object(VoxelObjects::Castle, &mut self.animation_handler);
                }
            }
        }
    }
    pub fn process_event(&mut self, event: &WindowEvent, screen: &PhysicalSize<u32>) {
        if !self.camera_controller.auto {
            self.camera_controller.process_events(event);
        }
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => match keycode {
                KeyCode::Space => match state {
                    winit::event::ElementState::Pressed => {
                        self.camera_controller.auto = !self.camera_controller.auto;
                        if self.camera_controller.auto {
                            self.camera_controller.is_right_pressed = true;

                            self.camera_controller.speed = 0.4;
                        } else {
                            self.camera_controller.speed = 1.0;
                            self.camera_controller.is_right_pressed = false;
                        }
                    }
                    _ => {}
                },
                KeyCode::Delete => {
                    self.animation_handler.reverse();
                }
                KeyCode::Insert => match state {
                    winit::event::ElementState::Pressed => {
                        if self.animation_handler.disabled {
                            self.animation_handler.enable();
                            println!("Enabled animations")
                        } else {
                            self.animation_handler.disable();
                            println!("Disabled animations")
                        }
                    }
                    _ => {}
                },
                KeyCode::Home => match state {
                    winit::event::ElementState::Pressed => {
                        if let Some(controller) = self.instance_controllers.first_mut() {
                            self.camera_controller.auto = true;
                            self.animation_handler
                                .reset_instance_position_to_current_position(
                                    &mut controller.instances,
                                );
                            // controller.update_buffer(&self.queue);
                        }
                    }
                    _ => {}
                },
                KeyCode::End => match state {
                    winit::event::ElementState::Pressed => {
                        log::warn!("Clicked");

                        self.voxel_helper.transition_to_object(
                            VoxelObjects::Viking,
                            &mut self.animation_handler,
                        );
                    }
                    _ => {}
                },
                KeyCode::PageUp => match state {
                    winit::event::ElementState::Pressed => {
                        println!("{:?}", self.camera_controller.camera.eye);
                        self.voxel_helper.transition_to_object(
                            VoxelObjects::Buttplug,
                            &mut self.animation_handler,
                        );
                    }
                    _ => {}
                },
                KeyCode::PageDown => match state {
                    winit::event::ElementState::Pressed => {
                        self.voxel_helper.transition_to_object(
                            VoxelObjects::Castle,
                            &mut self.animation_handler,
                        );
                    }
                    _ => {}
                },
                KeyCode::ScrollLock => match state {
                    winit::event::ElementState::Pressed => {
                        self.voxel_helper
                            .explode_object(&mut self.animation_handler, 25.0);
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::MouseInput { state, button, .. } => {
                warn!("test123");
                match button {
                    winit::event::MouseButton::Left => {
                        match state {
                            winit::event::ElementState::Pressed => {
                                let test = self.camera_controller.camera.screen_to_world_ray(
                                    self.cursor_position.x,
                                    self.cursor_position.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                println!("{:?}", self.camera_controller.camera.eye);
                                println!("{:?}", self.camera_controller.camera.target);

                                // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);

                                if let Some(controller) = self.instance_controllers.first_mut() {
                                    if let Some(i) = line_trace(controller, test) {
                                        if let Some(instance) = controller.instances.get_mut(i) {
                                            instance.should_render = false;
                                            controller.count -= 1;
                                        }
                                    }
                                }

                                log::warn!("CLickedm ouse!");
                            }
                            _ => {}
                        }
                    }
                    winit::event::MouseButton::Right => match state {
                        winit::event::ElementState::Pressed => {
                            let test = self.camera_controller.camera.screen_to_world_ray(
                                self.cursor_position.x,
                                self.cursor_position.y,
                                screen.width as f32,
                                screen.height as f32,
                            );
                            println!("{:?}", test);
                            // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);

                            if let Some(controller) = self.instance_controllers.first_mut() {
                                if let Some(index) = line_trace(controller, test) {
                                    let sphere_radius = 5.0;
                                    let sphere_center =
                                        controller.instances.get(index).unwrap().position;
                                    for (i, instance) in controller
                                        .instances
                                        .iter_mut()
                                        // .filter(|inst| inst.should_render)
                                        .enumerate()
                                    {
                                        if aabb_sphere_intersect(
                                            instance.position,
                                            instance.bounding,
                                            sphere_center,
                                            sphere_radius,
                                        ) {
                                            let direction = instance.position - sphere_center;
                                            let distance = direction.magnitude().max(0.001); // Avoid divide-by-zero
                                            let normalized = direction / distance; // This gives correct direction away from sphere_center

                                            let explosion_scale = 8.0; // Tune how far it flies
                                            let upward_boost = 5.0; // Lift upward
                                            let strength = 1.0 / distance; // Closer = stronger push

                                            let mut explosion_vec =
                                                normalized * strength * explosion_scale;
                                            explosion_vec.y += upward_boost;

                                            let target_position = explosion_vec;

                                            let animation =
                                                AnimationType::Step(AnimationStep::new(
                                                    target_position,
                                                    1.0,
                                                    false,
                                                    false,
                                                    false,
                                                    AnimationTransition::EaseOut(EaseOut),
                                                ));
                                            // instance.color = Vector3::new(0.0, 1.0, 0.0);
                                            self.animation_handler.set_animation(i, animation);
                                            // animation_handler.reset_animation_time(index);
                                            self.animation_handler.set_animation_state(i, true);
                                        }
                                    }

                                    controller.update_buffer(&self.queue);
                                };
                            }

                            log::warn!("CLickedm ouse!");
                        }
                        _ => {}
                    },
                    // winit::event::MouseButton::Right => todo!(),
                    // winit::event::MouseButton::Middle => todo!(),
                    // winit::event::MouseButton::Back => todo!(),
                    // winit::event::MouseButton::Forward => todo!(),
                    // winit::event::MouseButton::Other(_) => todo!(),
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = PhysicalPosition::new(position.x as f32, position.y as f32);
                // let test = self.camera_controller.camera.screen_to_world_ray(
                //     self.cursor_position.x,
                //     self.cursor_position.y,
                //     screen.width as f32,
                //     screen.height as f32,
                // );
                // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);

                // if let Some(controller) = self.chunk_map.get_mut(&target_chunk) {
                //     if let Some(i) = line_trace(controller, test) {
                //         controller.remove_instance(i, &self.queue);
                //     }
                // }
            }

            _ => {}
        }
    }
    pub fn new(
        name: String,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        size: PhysicalSize<u32>,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        // Create a merged AnimationHandler based on all instances in chunk_map

        let camera_controller = CameraController::new(0.4, size, &device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        // Load shaders
        let primitive_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PrimitiveShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/primitive.wgsl").into()),
        });
        let light_position = Vector3::new(60.0, 20.0, 60.0);
        let mesh = make_cube_primitive();
        let mut light_source = Light::new(light_position, Vector3::new(1.0, 1.0, 1.0), &device);
        let (mb, renderer) = mesh.get_mesh_buffer(
            &device,
            &primitive_shader,
            surface_format,
            &queue,
            camera_controller.camera_bind_group_layout.clone(),
            light_source.light_bind_group_layout.clone(),
            light_source.light_bind_group.clone(),
            size,
        );
        let instance_controller = InstanceController::new(
            // instances_list_circle(origin, chunk_size),
            vec![instance_cube(light_position.clone())],
            0,
            mb,
            renderer,
            &device,
        );
        light_source.instance_controller = Some(instance_controller);
        log::warn!("Pipeline");

        // Create render pipeline

        // Create instance controller and game loop

        let chunk_size_cube = Vector3::new(40, 40, 40);

        let mesh_type = MeshType::Primitive;
        let instance_controller: InstanceController = match mesh_type {
            MeshType::Primitive => {
                let origin = Chunk { x: 0, y: 0 };
                let mesh = make_cube_primitive();
                let (mb, renderer) = mesh.get_mesh_buffer(
                    &device,
                    &primitive_shader,
                    surface_format,
                    &queue,
                    camera_controller.camera_bind_group_layout.clone(),
                    light_source.light_bind_group_layout.clone(),
                    light_source.light_bind_group.clone(),
                    size,
                );
                let instance_controller = InstanceController::new(
                    // instances_list_circle(origin, chunk_size),
                    instances_list_cube(origin, chunk_size_cube),
                    0,
                    mb,
                    renderer,
                    &device,
                );
                instance_controller
            }
            MeshType::Textured => {
                let origin = Chunk { x: 0, y: 0 };
                let mesh = make_cube_textured();
                let (mb, renderer) = mesh.get_mesh_buffer(
                    &device,
                    &shader,
                    surface_format,
                    &queue,
                    camera_controller.camera_bind_group_layout.clone(),
                    light_source.light_bind_group_layout.clone(),
                    light_source.light_bind_group.clone(),
                    size,
                );
                let instance_controller = InstanceController::new(
                    instances_list_cube(origin, chunk_size_cube),
                    0,
                    mb,
                    renderer,
                    &device,
                );
                instance_controller
            }
        };

        let instance_controllers = vec![instance_controller];

        let persistent = AnimationPersistent::new(
            Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            AnimationTransition::EaseInEaseOutLoop(EaseInEaseOutLoop),
        );

        let animation_enums = vec![AnimationType::Persistent(persistent)];
        // let animation_enums = vec![];
        let animation_handler =
            AnimationHandler::new(&instance_controllers.first().unwrap(), animation_enums);

        let test = include_bytes!("../../src/test.vox");
        let castle = include_bytes!("../../src/castle.vox");
        let chr_knight = include_bytes!("../../src/chr_knight.vox");
        let rust_logo = include_bytes!("../../src/rust.vox");
        let c_plus_plus = include_bytes!("../../src/cplusplus.vox");
        let c_sharp = include_bytes!("../../src/csharp.vox");

        let mut voxel_handler = VoxelHandler::new();
        voxel_handler.add_voxel(test, VoxelObjects::Buttplug);
        voxel_handler.add_voxel(castle, VoxelObjects::Castle);
        voxel_handler.add_voxel(chr_knight, VoxelObjects::Viking);
        voxel_handler.add_voxel(rust_logo, VoxelObjects::Rust);
        voxel_handler.add_voxel(c_plus_plus, VoxelObjects::CPlusPLus);
        voxel_handler.add_voxel(c_sharp, VoxelObjects::CSharp);

        let mut transition_map = BTreeMap::new();
        transition_map.insert(500, VoxelObjects::Home);
        transition_map.insert(1000, VoxelObjects::CSharp);
        transition_map.insert(1500, VoxelObjects::Rust);
        transition_map.insert(2000, VoxelObjects::CPlusPLus);
        transition_map.insert(2500, VoxelObjects::Containerization);

        let transition_handler = TransitionHandler::new(transition_map);

        let mut camera_transition = BTreeMap::new();
        camera_transition.insert(
            500,
            CameraPositions::Home(((-120, 90, -120).into(), (20, 25, 20).into())),
        );
        camera_transition.insert(
            1000,
            CameraPositions::CSharp(((-50, 90, -190).into(), (90, 25, -50).into())),
        );
        camera_transition.insert(
            1500,
            CameraPositions::Rust(((90, 90, -190).into(), (-50, 25, -50).into())),
        );
        camera_transition.insert(
            2000,
            CameraPositions::CPlusPLus(((-40, 90, -190).into(), (90, 25, -40).into())),
        );

        let camera_transition_handler = TransitionHandler::new(camera_transition);
        Gameloop {
            name,
            device,
            queue,
            camera_controller,
            light_source,
            instance_controllers,
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
            elapsed_time: 0.0,
            voxel_helper: voxel_handler,
            animation_handler,
            camera_transition_handler,
            transition_handler,
        }
    }
}
