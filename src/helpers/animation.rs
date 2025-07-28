use crate::entity::entity::Instance;
use crate::entity::entity::InstanceController;
use cgmath::{
    num_traits::{pow, ToPrimitive},
    Vector3,
};

// pub fn ease_in_ease_out_loop(dt: u64, delay: u64, freq: u64) -> f32 {
//     if dt < delay {
//         return 0.0;
//     }
//     let elapsed = (dt - delay) % (freq * 2);
//     if elapsed >= freq {
//         let time = ((freq * 2) - elapsed).to_f32().unwrap() / freq as f32;
//         let sqr = time * time;
//         sqr / (2.0 * (sqr - time) + 1.0)
//     } else {
//         let time = elapsed.to_f32().unwrap() / freq as f32;
//         let sqr = time * time;
//         sqr / (2.0 * (sqr - time) + 1.0)
//     }
// }

pub fn ease_in_ease_out_loop(dt: f32, delay: f32, freq: f32) -> f32 {
    if dt < delay {
        return 0.0;
    }
    let elapsed = (dt - delay) % (freq * 2.0);
    let time = if elapsed >= freq {
        (2.0 * freq - elapsed) / freq
    } else {
        elapsed / freq
    };
    let sqr = time * time;
    sqr / (2.0 * (sqr - time) + 1.0)
}

pub fn get_height_color(height: f32) -> Vector3<f32> {
    // high color rgb(255, 153, 230)
    //low color rgb(204, 0, 153)

    let high_color = Vector3::new(0.9, 0.4, 0.702);
    let low_color = Vector3::new(0.8, 0.0, 0.6);
    low_color + (high_color - low_color) * height
}

pub struct EaseInEaseOut;
impl EaseInEaseOut {
    pub fn ease_in_ease_out_cubic(number: f32) -> f32 {
        let number = number.clamp(0.0, 1.0);
        return if number < 0.5 {
            4.0 * number * number * number
        } else {
            1.0 - pow(-2.0 * number + 2.0, 3) / 2.0
        };
    }
}

enum AnimationTransition {
    EaseInEaseOut(EaseInEaseOut),
}

impl AnimationTransition {
    pub fn lerp(&self, start: Vector3<f32>, end: Vector3<f32>, number: f32) -> Vector3<f32> {
        match self {
            AnimationTransition::EaseInEaseOut(_) => {
                let lerp_value = EaseInEaseOut::ease_in_ease_out_cubic(number);
                start + (end - start) * lerp_value
            }
        }
    }
}

pub struct Animation {
    activated: bool,
    time: f32,
    reversed: bool,
    start: Vector3<f32>,
    end: Vector3<f32>,
    pub current_pos: Vector3<f32>,
    animation_transition: AnimationTransition,
}

impl Animation {
    pub fn set_animation(&mut self, start: &Vector3<f32>, end: &Vector3<f32>) {
        self.start = start.clone();
        self.end = end.clone();
    }

    pub fn set_animation_state(&mut self, state: bool) {
        self.activated = state
    }

    pub fn reverse(&mut self, state: bool) {
        self.reversed = state
    }
}

pub struct AnimationHandler {
    pub movement_list: Vec<Animation>,
    pub disabled: bool,
}

impl AnimationHandler {
    pub fn new(instance_controller: &InstanceController) -> AnimationHandler {
        AnimationHandler {
            disabled: false,
            movement_list: {
                instance_controller
                    .instances
                    .iter()
                    .map(|instance| Animation {
                        activated: false,
                        start: instance.position,
                        end: instance.position,
                        current_pos: instance.position,
                        time: 0.0,
                        reversed: false,
                        animation_transition: AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                    })
                    .collect()
            },
        }
    }

    pub fn disable(&mut self) {
        self.disabled = true;
    }
    pub fn enable(&mut self) {
        self.disabled = false;
    }

    pub fn set_animation(&mut self, index: usize, start: &Vector3<f32>, end: &Vector3<f32>) {
        if self.disabled {
            return;
        }
        if let Some(animation) = self.movement_list.get_mut(index) {
            if !animation.activated {
                animation.set_animation(start, end);
            }
        }
    }

    pub fn set_animation_state(&mut self, index: usize, state: bool) {
        if self.disabled {
            return;
        }
        if let Some(animation) = self.movement_list.get_mut(index) {
            animation.set_animation_state(state);
        }
    }

    pub fn reset_animation_time(&mut self, index: usize) {
        if self.disabled {
            return;
        }
        if self.disabled {
            return;
        }
        if let Some(animation) = self.movement_list.get_mut(index) {
            if !animation.activated {
                animation.time = 0.0;
            }
        }
    }

    pub fn reverse(&mut self, index: usize, state: bool) {
        if self.disabled {
            return;
        }
        if let Some(animation) = self.movement_list.get_mut(index) {
            animation.reverse(state);
        }
    }

    pub fn animate(&mut self, dt: f32) {
        if self.disabled {
            return;
        }
        for animation in self.movement_list.iter_mut() {
            let mut delta = dt;
            if !animation.activated {
                continue;
            }
            if animation.reversed {
                delta *= -1.0;
            }
            animation.time += delta;
            animation.time = animation.time.clamp(0.0, 1.0);
            animation.current_pos =
                animation
                    .animation_transition
                    .lerp(animation.start, animation.end, animation.time);
            if animation.time == 1.0 || animation.time == 0.0 {
                animation.activated = false;
            }
        }
    }

    pub fn update_instance(&mut self, index: usize, instance: &mut Instance) {
        if let Some(animation) = self.movement_list.get_mut(index) {
            if !animation.activated {
                return;
            }
            instance.position = animation.current_pos;
            instance.bounding = instance.size + animation.current_pos;
        }
    }
}
