use crate::entity::entity::Instance;
use crate::entity::entity::InstanceController;
use cgmath::num_traits::float;
use cgmath::{num_traits::pow, Vector3};

#[derive(Copy, Clone)]
pub struct EaseInEaseOutLoop;
impl EaseInEaseOutLoop {
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
}
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

#[derive(Copy, Clone)]
pub struct EaseOut;
impl EaseOut {
    pub fn ease_out_cubic(number: f32) -> f32 {
        let t = number.clamp(0.0, 1.0);
        1.0 - (1.0 - t).powi(3)
    }
}
#[derive(Copy, Clone)]

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
#[derive(Copy, Clone)]
pub enum AnimationTransition {
    EaseOut(EaseOut),
    EaseInEaseOut(EaseInEaseOut),
    EaseInEaseOutLoop(EaseInEaseOutLoop),
}

impl AnimationTransition {
    pub fn lerp(
        &self,
        start: Vector3<f32>,
        end: Vector3<f32>,
        number: f32,
        delay: f32,
    ) -> Vector3<f32> {
        match self {
            AnimationTransition::EaseInEaseOut(_) => {
                let lerp_value = EaseInEaseOut::ease_in_ease_out_cubic(number);
                start + (end - start) * lerp_value
            }
            AnimationTransition::EaseInEaseOutLoop(_) => {
                let lerp_value = EaseInEaseOutLoop::ease_in_ease_out_loop(number, delay, 1.0) - 0.5;
                start + (end - start) * lerp_value
            }
            AnimationTransition::EaseOut(_) => {
                let lerp_value = EaseOut::ease_out_cubic(number);
                start + (end - start) * lerp_value
            }
        }
    }
}

//Send og Sync, trådhelvede
pub enum AnimationType {
    Persistent(AnimationPersistent),
    Step(AnimationStep),
}

#[derive(Clone)]
pub struct AnimationPersistent {
    time: f32,
    movement_vector: Vector3<f32>,
    animation_transition: AnimationTransition,
}
impl AnimationPersistent {
    pub fn new(movement_vector: Vector3<f32>, animation_transition: AnimationTransition) -> Self {
        Self {
            time: 0.0,
            movement_vector,
            animation_transition,
        }
    }
}
#[derive(Clone)]
pub struct AnimationStep {
    movement_vector: Vector3<f32>,
    time: f32,
    reversed: bool,
    activated: bool,
    animating: bool,
    speed: f32,
    animation_transition: AnimationTransition,
    one_time_animation: bool,
}

impl AnimationStep {
    /// Constructs a new AnimationStep
    pub fn new(
        movement_vector: Vector3<f32>,
        speed: f32,
        reversed: bool,
        activated: bool,
        one_time_animation: bool,
        animation_transition: AnimationTransition,
    ) -> Self {
        Self {
            movement_vector,
            time: 0.0,
            reversed,
            activated,
            speed,
            one_time_animation,
            animating: false,
            animation_transition,
        }
    }
}
#[derive(Clone)]

pub struct Animation {
    activated: bool,
    time: f32,
    pub start: Vector3<f32>,
    pub current_pos: Vector3<f32>,
    pub grid_pos: Vector3<f32>,
    persistent_animation: Vec<AnimationPersistent>,

    animations: Vec<AnimationStep>,
    color: Vector3<f32>,
}

pub struct AnimationHandler {
    pub movement_list: Vec<Animation>,
    pub disabled: bool,
}

impl AnimationHandler {
    pub fn new(
        instance_controller: &InstanceController,
        animations: Vec<AnimationType>,
    ) -> AnimationHandler {
        let mut steps = Vec::new();
        let mut persistents = Vec::new();

        for anim in animations {
            match anim {
                AnimationType::Step(step) => steps.push(step),
                AnimationType::Persistent(persistent) => persistents.push(persistent),
                // add other variants here as needed
            }
        }
        AnimationHandler {
            disabled: false,

            movement_list: {
                instance_controller
                    .instances
                    .iter()
                    .map(|instance| Animation {
                        activated: true,
                        start: instance.position,
                        current_pos: instance.position,
                        grid_pos: instance.position,
                        persistent_animation: persistents.clone(),
                        animations: steps.clone(),
                        color: Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        time: 0.0,
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

    pub fn set_animation(&mut self, index: usize, animation_type: AnimationType) {
        if self.disabled {
            return;
        }
        if let Some(animation) = self.movement_list.get_mut(index) {
            match animation_type {
                AnimationType::Persistent(animation_persistent) => {
                    animation.persistent_animation.push(animation_persistent);
                }
                AnimationType::Step(animation_step) => {
                    animation.animations.push(animation_step);
                }
            }
        }
    }

    pub fn is_locked(&mut self) -> bool {
        let mut locked = false;
        for animation in self.movement_list.iter_mut() {
            for step in animation.animations.iter_mut() {
                if (step.one_time_animation) {
                    locked = step.one_time_animation
                }
            }
        }
        locked
    }

    pub fn set_animation_state(&mut self, index: usize, state: bool) {
        if self.disabled {
            return;
        }
        if let Some(animation) = self.movement_list.get_mut(index) {
            for step in animation.animations.iter_mut() {
                step.activated = state;
                step.animating = state;
            }
        }
    }

    pub fn reverse(&mut self) {
        for animation in self.movement_list.iter_mut() {
            for step in animation.animations.iter_mut() {
                step.reversed = true;
            }
        }
    }

    pub fn reset_instance_position_to_current_position(&mut self, instances: &mut Vec<Instance>) {
        for (i, animation) in self.movement_list.iter_mut().enumerate() {
            if let Some(instance) = instances.get_mut(i) {
                if animation.animations.is_empty() {
                    continue;
                }

                let delay = ((animation.current_pos.x + animation.current_pos.z) * 0.05);
                let mut total_movement = Vector3::new(0.0, 0.0, 0.0);
                for persistent in animation.persistent_animation.iter_mut() {
                    total_movement += persistent.animation_transition.lerp(
                        animation.start,
                        animation.start + persistent.movement_vector,
                        persistent.time,
                        delay,
                    ) - animation.start;
                }
                instance.position = animation.current_pos - total_movement;
                instance.bounding = animation.current_pos + animation.current_pos - total_movement;
                animation.start = instance.position;
            } else {
                continue;
            };
            animation.animations.clear();
        }
    }
    pub fn animate(&mut self, dt: f32) {
        if self.disabled {
            return;
        }
        for (i, animation) in self
            .movement_list
            .iter_mut()
            .filter(|ani| ani.activated)
            .enumerate()
        {
            let delta = dt;

            animation.time += dt;

            let delay = ((animation.current_pos.x + animation.current_pos.z) * 0.05);
            let mut total_movement = animation.start.clone();
            for persistent in animation.persistent_animation.iter_mut() {
                persistent.time += delta;
                total_movement += persistent.animation_transition.lerp(
                    animation.start,
                    animation.start + persistent.movement_vector,
                    persistent.time,
                    delay,
                ) - animation.start;
            }
            let lerp = 1.0 * ease_in_ease_out_loop(animation.time, delay as f32, 1.0);
            animation.color = get_height_color(lerp);
            let mut step_delta = Vector3::new(0.0, 0.0, 0.0);
            for (step_i, step) in animation.animations.iter_mut().enumerate() {
                let mut step_movement = Vector3::new(0.0, 0.0, 0.0);
                if !step.activated {
                    continue;
                }

                if step.reversed {
                    step.time -= delta * step.speed;
                    step.time = step.time.clamp(0.0, 1.0);
                    step_movement -= step.animation_transition.lerp(
                        animation.start + step.movement_vector,
                        animation.start,
                        step.time,
                        0.0,
                    ) - (animation.start + step.movement_vector);
                } else {
                    step.time += delta * step.speed;
                    step.time = step.time.clamp(0.0, 1.0);
                    step_movement += step.animation_transition.lerp(
                        animation.start,
                        animation.start + step.movement_vector,
                        step.time,
                        0.0,
                    ) - animation.start;
                };
                if step.time == 0.0 || step.time == 1.0 {
                    step.animating = false
                }

                if step.one_time_animation && step.time == 1.0 {
                    animation.start = animation.start + step_movement;
                    animation.grid_pos = animation.start + step_movement;
                }
                step_delta += step_movement;
            }
            animation.grid_pos = animation.start + step_delta;
            total_movement += step_delta;
            animation.current_pos = total_movement;
            animation.animations.retain(|step| {
                !((step.reversed && step.time == 0.0)
                    || (step.one_time_animation && step.time == 1.0))
            });
        }
    }

    pub fn update_instance(&mut self, index: usize, instance: &mut Instance) {
        if let Some(animation) = self.movement_list.get_mut(index) {
            if !animation.activated {
                return;
            }
            instance.position = animation.current_pos;
            instance.bounding = instance.size + animation.current_pos;
            instance.color = animation.color;
        }
    }
}
