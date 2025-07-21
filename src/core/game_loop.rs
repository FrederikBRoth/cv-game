use std::sync::Arc;

use cgmath::{InnerSpace, Rotation3};
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    entity::entity::{Instance, InstanceController},
    helpers::animation::ease_in_ease_out,
};

pub struct Gameloop {
    pub name: String,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub instance_controller: InstanceController,
    pub elapsed_time: u64,
}

impl Gameloop {
    pub fn update(&mut self) {
        let mut test = 0;
        for instance in self.instance_controller.instances.iter_mut() {
            instance.position.y = 1.0 * ease_in_ease_out(self.elapsed_time, test);
            test += 15;
        }
        self.instance_controller.update_buffer(&self.queue);
        self.elapsed_time += 1;
    }
    pub fn process_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
            ..
        } = event
        {
            match keycode {
                KeyCode::Delete => {
                    self.instance_controller.remove_instance(
                        self.instance_controller.instances.len() - 10,
                        &self.queue,
                    );
                }
                KeyCode::Insert => {
                    let position = cgmath::Vector3 {
                        x: 5.0,
                        y: 0.0,
                        z: 5.0,
                    };

                    let instance = Instance {
                        position,
                        rotation: cgmath::Quaternion::from_axis_angle(
                            position.normalize(),
                            cgmath::Deg(45.0),
                        ),
                        scale: 0.5,
                    };
                    self.instance_controller.add_instance(instance, &self.queue);
                }
                _ => {}
            }
        }
    }
}
