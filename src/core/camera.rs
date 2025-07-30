use cgmath::{EuclideanSpace, InnerSpace, Point3, SquareMatrix, Vector3, Vector4};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::entity::entity::OPENGL_TO_WGPU_MATRIX;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // let ortho = cgmath::ortho(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
    pub fn screen_to_world_ray(
        &self,
        mouse_x: f32,
        mouse_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> (Point3<f32>, Vector3<f32>) {
        // Convert screen coords to normalized device coordinates (NDC)
        let front = self
            .project_screen_to_world(mouse_x, mouse_y, 1.0, screen_width, screen_height)
            .unwrap();
        let back = self
            .project_screen_to_world(mouse_x, mouse_y, 0.0, screen_width, screen_height)
            .unwrap();

        (Point3::from_vec(back), -(front - back).normalize())
    }

    pub fn project_screen_to_world(
        &self,
        mouse_x: f32,
        mouse_y: f32,
        mouse_z: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<Vector3<f32>> {
        let view_projection = OPENGL_TO_WGPU_MATRIX * self.build_view_projection_matrix();
        if let Some(inv_view_projection) = view_projection.invert() {
            let world = Vector4::new(
                (mouse_x) / (screen_width as f32) * 2.0 - 1.0,
                // Screen Origin is Top Left    (Mouse Origin is Top Left)
                //          (screen.y - (viewport.y as f32)) / (viewport.w as f32) * 2.0 - 1.0,
                // Screen Origin is Bottom Left (Mouse Origin is Top Left)
                (1.0 - (mouse_y) / (screen_height as f32)) * 2.0 - 1.0,
                mouse_z * 2.0 - 1.0,
                1.0,
            );
            let world = inv_view_projection * world;

            if world.w != 0.0 {
                Some(world.truncate() * (1.0 / world.w))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

pub struct CameraController {
    pub speed: f32,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let var_name = *state == ElementState::Pressed;
                let is_pressed = var_name;
                match keycode {
                    KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;

                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }

            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
