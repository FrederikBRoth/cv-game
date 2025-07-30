use std::collections::HashSet;
use std::f32::consts::PI;

use crate::helpers::animation::{
    AnimationHandler, AnimationStep, AnimationTransition, EaseInEaseOut,
};
use crate::{entity::entity::Instance, helpers::animation::AnimationType};
use cgmath::Vector3;
use dot_vox::{load, load_bytes};
use rand::seq::SliceRandom;
use rand::{rng, thread_rng};

pub struct Object {
    pub cubes: Vec<Vector3<f32>>,
}

pub struct VoxelHandler {
    pub voxels: Vec<Object>,
    pub current_cubes: Vec<usize>,
    pub current_object: usize,
}
impl VoxelHandler {
    pub fn new() -> Self {
        Self {
            voxels: vec![],
            current_cubes: vec![],
            current_object: 1,
        }
    }

    pub fn add_voxel(&mut self, path: &[u8]) {
        match load_bytes(path) {
            Ok(scene) => {
                for model in scene.models {
                    let new_voxel = Object {
                        cubes: model
                            .voxels
                            .iter()
                            .map(|v| Vector3::new(v.x as f32, v.z as f32, v.y as f32))
                            .collect(),
                    };
                    self.voxels.push(new_voxel);
                }
            }
            Err(err) => {
                log::warn!("Failed to load voxel file");
                eprintln!("Failed to load .vox file: {}", err);
            }
        }
    }

    pub fn get_object(&self, index: usize) -> Option<&Object> {
        if let Some(object) = &self.voxels.get(index) {
            Some(*&object)
        } else {
            None
        }
    }

    pub fn explode_object(
        &mut self,
        instances: &mut Vec<Instance>,
        animation_handler: &mut AnimationHandler,
        amplify: f32,
    ) {
        self.transition_to_object_base(
            self.current_object,
            instances,
            animation_handler,
            amplify,
            false,
        );
    }
    pub fn transition_to_object(
        &mut self,
        index: usize,
        instances: &mut Vec<Instance>,
        animation_handler: &mut AnimationHandler,
    ) {
        self.transition_to_object_base(index, instances, animation_handler, 1.0, true);
    }
    fn transition_to_object_base(
        &mut self,
        index: usize,
        instances: &mut Vec<Instance>,
        animation_handler: &mut AnimationHandler,
        amplify: f32,
        is_onetime: bool,
    ) {
        self.current_object = index;
        let mut current_cubes = self.current_cubes.clone();
        let mut new_current_cubes: Vec<usize> = vec![];
        if let Some(object) = self.get_object(index) {
            if object.cubes.len() > instances.len() {
                println!("Object too large to show");
                return;
            }
            let cube_indices: Vec<usize> = (0..object.cubes.len()).collect();
            let instance_indices: Vec<usize> = (0..animation_handler.movement_list.len()).collect();
            if current_cubes.is_empty() {
                current_cubes = instance_indices.clone();
            }
            let instance_indices_in_order: Vec<usize> =
                (0..animation_handler.movement_list.len()).collect();

            let cube_indices_len = cube_indices.len();
            let current_cubes_len = current_cubes.len();
            let mut rng = rng();

            if cube_indices_len > current_cubes_len {
                let current_cubes_set: HashSet<_> = current_cubes.iter().copied().collect();

                let mut cube_indicees_excluded: Vec<usize> = instance_indices
                    .iter()
                    .filter(|i| !current_cubes_set.contains(i))
                    .copied()
                    .collect();

                cube_indicees_excluded.shuffle(&mut rng);
                for n in 0..(cube_indices_len - current_cubes_len) {
                    let elem = cube_indicees_excluded.get(n).unwrap();
                    current_cubes.push(*elem);
                }
                println!("Bigger!")
            } else {
                println!("Smaller or the same")
            }

            // current_cubes.shuffle(&mut rng);
            current_cubes.shuffle(&mut rng);
            let cubes_indices: Vec<usize> = (0..object.cubes.len()).collect();
            for &i in &cubes_indices {
                let cube = object.cubes.get(i).unwrap();
                let instance_index = current_cubes.pop().unwrap();
                let animation = animation_handler.movement_list.get(instance_index).unwrap();
                new_current_cubes.push(instance_index);
                let movement_vector = cube - animation.grid_pos;
                let animation = AnimationType::Step(AnimationStep::new(
                    movement_vector * amplify,
                    0.4,
                    false,
                    false,
                    is_onetime,
                    AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                ));
                animation_handler.set_animation(instance_index, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(instance_index, true);
            }
            let instance_indices_remaining: HashSet<usize> =
                current_cubes.iter().cloned().collect();

            // Filter instance_indices to exclude anything in cubes_set
            let remaining_indices: Vec<usize> = instance_indices_in_order
                .clone()
                .into_iter()
                .filter(|i| instance_indices_remaining.contains(i))
                .collect();

            let mut circle = fibonacci_sphere(remaining_indices.clone().len(), 750.0);

            for i in remaining_indices {
                let animation = animation_handler.movement_list.get(i).unwrap();

                let point = circle.pop().unwrap();
                let movement_vector = Vector3::new(point.x, point.y, point.z);
                let movement_vector = movement_vector - animation.current_pos;
                let animation = AnimationType::Step(AnimationStep::new(
                    movement_vector,
                    0.25,
                    false,
                    false,
                    is_onetime,
                    AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                ));
                animation_handler.set_animation(i, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(i, true);
            }
        } else {
            println!("Object does not exist!");
            log::warn!("Objet does not exists");

            return;
        }
        self.current_cubes = new_current_cubes;
    }
}

fn fibonacci_sphere(points: usize, scalar: f32) -> Vec<Vector3<f32>> {
    let mut vecs: Vec<Vector3<f32>> = vec![];
    let phi = PI * (f32::sqrt(5.0) - 1.0);

    for n in 0..points {
        let y = 1.0 - (n as f32 / (points as f32 - 1.0)) * 2.0;
        let radius = f32::sqrt(1.0 - y * y);
        let theta = phi * n as f32;

        let x = f32::cos(theta) * radius;
        let z = f32::sin(theta) * radius;

        vecs.push(Vector3 { x: x, y: y, z: z } * scalar);
    }

    vecs
}
