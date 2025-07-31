use std::{collections::HashMap, sync::Arc};

use cgmath::{InnerSpace, Vector2, Vector3};
use wgpu::CommandEncoder;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    core::camera::{Camera, CameraController},
    entity::entity::{
        instances_list, instances_list_cube, make_cube_primitive, make_cube_textured,
        InstanceController, MeshType,
    },
    helpers::{
        animation::{
            ease_in_ease_out_loop, get_height_color, AnimationHandler, AnimationPersistent,
            AnimationStep, AnimationTransition, AnimationType, EaseInEaseOut, EaseInEaseOutLoop,
            EaseOut,
        },
        line_trace::{aabb_sphere_intersect, line_trace, line_trace_animate_hit},
        voxel_builder::{self, VoxelHandler},
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
    pub instance_controllers: Vec<InstanceController>,
    pub camera_controller: CameraController,
    pub elapsed_time: f32,
    pub animation_handler: AnimationHandler,
    pub voxel_helper: VoxelHandler,
}

impl Gameloop {
    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, texture_view: &wgpu::TextureView) {
        for instance_controller in self.instance_controllers.iter_mut() {
            instance_controller.render(
                encoder,
                texture_view,
                self.camera_controller.camera_bind_group.clone(),
            );
        }
    }
    pub fn update(&mut self, dt: std::time::Duration) {
        let dts = dt.as_secs_f32();
        for instance_controller in self.instance_controllers.iter_mut() {
            self.animation_handler.animate(dts);

            for (i, instance) in instance_controller.instances.iter_mut().enumerate() {
                // }
                self.animation_handler.update_instance(i, instance);

                // instance_controller.update_buffer(&self.queue);
                // assuming queue: Arc<wgpu::Queue>
            }
            instance_controller.update_buffer_multithreaded(Arc::clone(&self.queue));

            self.elapsed_time += dts;
        }

        if self.camera_controller.auto {
            if self.elapsed_time < 5.0 {
                return;
            }
            let target_chunk = Chunk { x: 0, y: 0 };
            let animation_time = self.elapsed_time % 24.0;
            let ready = !self.animation_handler.is_locked();
            if let Some(controller) = self.instance_controllers.first_mut() {
                if animation_time.floor() == 0.0 && ready {
                    self.voxel_helper.transition_to_object(
                        0,
                        &mut controller.instances,
                        &mut self.animation_handler,
                    );
                }
                if animation_time.floor() == 8.0 && ready {
                    self.voxel_helper.transition_to_object(
                        1,
                        &mut controller.instances,
                        &mut self.animation_handler,
                    );
                }
                if animation_time.floor() == 16.0 && ready {
                    self.voxel_helper.transition_to_object(
                        2,
                        &mut controller.instances,
                        &mut self.animation_handler,
                    );
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
                        if (self.camera_controller.auto) {
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
                        let target_chunk = Chunk { x: 0, y: 0 };

                        if let Some(controller) = self.instance_controllers.first_mut() {
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
                        let target_chunk = Chunk { x: 0, y: 0 };
                        log::warn!("Clicked");

                        if let Some(controller) = self.instance_controllers.first_mut() {
                            self.voxel_helper.transition_to_object(
                                0,
                                &mut controller.instances,
                                &mut self.animation_handler,
                            );
                        }
                    }
                    _ => {}
                },
                KeyCode::PageUp => match state {
                    winit::event::ElementState::Pressed => {
                        let target_chunk = Chunk { x: 0, y: 0 };
                        println!("{:?}", self.camera_controller.camera.eye);
                        if let Some(controller) = self.instance_controllers.first_mut() {
                            self.voxel_helper.transition_to_object(
                                1,
                                &mut controller.instances,
                                &mut self.animation_handler,
                            );
                        }
                    }
                    _ => {}
                },
                KeyCode::PageDown => match state {
                    winit::event::ElementState::Pressed => {
                        let target_chunk = Chunk { x: 0, y: 0 };

                        if let Some(controller) = self.instance_controllers.first_mut() {
                            self.voxel_helper.transition_to_object(
                                2,
                                &mut controller.instances,
                                &mut self.animation_handler,
                            );
                        }
                    }
                    _ => {}
                },
                KeyCode::ScrollLock => match state {
                    winit::event::ElementState::Pressed => {
                        let target_chunk = Chunk { x: 0, y: 0 };

                        if let Some(controller) = self.instance_controllers.first_mut() {
                            self.voxel_helper.explode_object(
                                &mut controller.instances,
                                &mut self.animation_handler,
                                25.0,
                            );
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::MouseInput { state, button, .. } => {
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
                                println!("{:?}", test);
                                // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);
                                let target_chunk = Chunk { x: 0, y: 0 };

                                if let Some(controller) = self.instance_controllers.first_mut() {
                                    // line_trace_cursor(
                                    //     controller,
                                    //     &self.chunk_size,
                                    //     &self.queue,
                                    //     test,
                                    // );
                                    // let animation = AnimationType::Step(AnimationStep::new(
                                    //     Vector3 {
                                    //         x: 0.0,
                                    //         y: 1.0,
                                    //         z: 0.0,
                                    //     },
                                    //     false,
                                    //     false,
                                    //     AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                                    // ));
                                    // line_trace_animate_hit(
                                    //     controller,
                                    //     &mut self.animation_handler,
                                    //     &self.queue,
                                    //     animation,
                                    //     test,
                                    // )
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
                            let target_chunk = Chunk { x: 0, y: 0 };

                            if let Some(controller) = self.instance_controllers.first_mut() {
                                // line_trace_cursor(
                                //     controller,
                                //     &self.chunk_size,
                                //     &self.queue,
                                //     test,
                                // );
                                let animation = AnimationType::Step(AnimationStep::new(
                                    Vector3 {
                                        x: 0.0,
                                        y: 1.0,
                                        z: 0.0,
                                    },
                                    1.0,
                                    true,
                                    false,
                                    false,
                                    AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                                ));
                                // line_trace_animate_hit(
                                //     controller,
                                //     &mut self.animation_handler,
                                //     &self.queue,
                                //     animation,
                                //     test,
                                // )
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
                let test = self.camera_controller.camera.screen_to_world_ray(
                    self.cursor_position.x,
                    self.cursor_position.y,
                    screen.width as f32,
                    screen.height as f32,
                );
                // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);
                let target_chunk = Chunk { x: 0, y: 0 };

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

        // Create depth texture for texture meshes

        // Create depth texture for primitive

        log::warn!("Pipeline");

        // Create render pipeline

        // Create instance controller and game loop

        let chunk_size = Vector2::new(35, 35);
        let chunk_size_cube = Vector3::new(25, 40, 25);

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
                    size,
                );
                let instance_controller = InstanceController::new(
                    instances_list(origin, chunk_size),
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
                x: 1.0,
                y: 2.0,
                z: 0.0,
            },
            AnimationTransition::EaseInEaseOutLoop(EaseInEaseOutLoop),
        );

        let animation_enums = vec![AnimationType::Persistent(persistent)];
        // let animation_enums = vec![];
        let animation_handler =
            AnimationHandler::new(&instance_controllers.first().unwrap(), animation_enums);
        Gameloop {
            name,
            device,
            queue,
            camera_controller,
            instance_controllers,
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
            elapsed_time: 0.0,
            voxel_helper: VoxelHandler::new(),
            animation_handler,
        }
    }
}
