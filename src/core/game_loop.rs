use std::{collections::HashMap, sync::Arc};

use cgmath::{InnerSpace, Rotation3, Vector2, Vector3};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    core::{camera::Camera, state::State},
    entity::entity::{Instance, InstanceController},
    helpers::{
        animation::{ease_in_ease_out_loop, get_height_color, AnimationHandler},
        line_trace::{line_trace_animate_hit, line_trace_cursor, line_trace_remove},
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
    pub chunk_map: HashMap<Chunk, InstanceController>,
    pub elapsed_time: f32,
    pub chunk_size: Vector2<u32>,
    pub animation_handler: AnimationHandler,
}

impl Gameloop {
    pub fn update(&mut self, dt: std::time::Duration) {
        let dts = dt.as_secs_f32();
        for (chunk, instance_controller) in self.chunk_map.iter_mut() {
            self.animation_handler.animate(dt.as_secs_f32());

            for (i, instance) in instance_controller.instances.iter_mut().enumerate() {
                let local_x = (i % self.chunk_size.x as usize) as u64;
                let local_y = (i / self.chunk_size.y as usize) as u64;
                let delay = ((chunk.x as f32 + chunk.y as f32) * 5.0)
                    + ((local_x as f32 + local_y as f32) * 0.05);
                // Diagonal wave offset for this tile
                let lerp = 1.0 * ease_in_ease_out_loop(self.elapsed_time, delay as f32, 1.0);
                if (i == 1) {
                    println!("{:?}", lerp);
                }
                self.animation_handler.update_instance(i, instance);

                // if (i == 200) {
                //     println!("{:?}", height);
                // }
                if self.animation_handler.disabled {
                    let pos = Vector3::new(0.0, lerp, 0.0);

                    if let Some(animation) = self.animation_handler.movement_list.get_mut(i) {
                        instance.position = animation.current_pos + pos;
                        instance.bounding = instance.size + animation.current_pos + pos;
                    }
                }
                instance.color = get_height_color(lerp)
                // test += 15;
            }

            instance_controller.update_buffer(&self.queue);
        }
        if self.animation_handler.disabled {
            self.elapsed_time += dt.as_secs_f32();
        }
    }
    pub fn process_event(
        &mut self,
        event: &WindowEvent,
        camera: &Camera,
        screen: &PhysicalSize<u32>,
    ) {
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
                KeyCode::Delete => {
                    let target_chunk = Chunk { x: 0, y: 0 };

                    if let Some(controller) = self.chunk_map.get_mut(&target_chunk) {
                        controller.remove_instance(controller.instances.len() - 50, &self.queue);
                    }
                }
                KeyCode::Insert => match state {
                    winit::event::ElementState::Pressed => {
                        if (self.animation_handler.disabled) {
                            self.animation_handler.enable();
                            println!("Enabled animations")
                        } else {
                            self.animation_handler.disable();
                            println!("Disabled animations")
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                match button {
                    winit::event::MouseButton::Left => {
                        match state {
                            winit::event::ElementState::Pressed => {
                                let test = camera.screen_to_world_ray(
                                    self.cursor_position.x,
                                    self.cursor_position.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                println!("{:?}", test);
                                // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);
                                let target_chunk = Chunk { x: 0, y: 0 };

                                if let Some(controller) = self.chunk_map.get_mut(&target_chunk) {
                                    // line_trace_cursor(
                                    //     controller,
                                    //     &self.chunk_size,
                                    //     &self.queue,
                                    //     test,
                                    // );
                                    line_trace_animate_hit(
                                        controller,
                                        &mut self.animation_handler,
                                        &self.queue,
                                        test,
                                    )
                                }

                                log::warn!("CLickedm ouse!");
                            }
                            _ => {}
                        }
                    }
                    winit::event::MouseButton::Right => match state {
                        winit::event::ElementState::Pressed => {}
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
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                self.cursor_position = PhysicalPosition::new(position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }
    pub fn new(
        name: String,
        cursor_position: PhysicalPosition<f32>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        chunk_size: Vector2<u32>,
        chunk_map: HashMap<Chunk, InstanceController>,
    ) -> Self {
        // Create a merged AnimationHandler based on all instances in chunk_map
        let instance_controller = &chunk_map.get(&Chunk { x: 0, y: 0 }).unwrap();

        let animation_handler = AnimationHandler::new(&instance_controller);

        Gameloop {
            name,
            cursor_position,
            device,
            queue,
            chunk_map,
            elapsed_time: 0.0,

            chunk_size,
            animation_handler,
        }
    }
}
