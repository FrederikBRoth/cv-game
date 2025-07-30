use std::collections::HashSet;

use crate::helpers::animation::{
    AnimationHandler, AnimationStep, AnimationTransition, EaseInEaseOut,
};
use crate::{entity::entity::Instance, helpers::animation::AnimationType};
use cgmath::Vector3;
use dot_vox::load;
use rand::seq::SliceRandom;
use rand::{rng, thread_rng};

pub struct Object {
    pub cubes: Vec<Vector3<f32>>,
}

pub struct VoxelHandler {
    pub voxels: Vec<Object>,
}
impl VoxelHandler {
    pub fn new() -> Self {
        Self { voxels: vec![] }
    }

    pub fn add_voxel(&mut self, path: &str) {
        match load(path) {
            Ok(scene) => {
                for model in scene.models {
                    println!("Model size: {:?}", model.size);

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

    pub fn transition_to_object(
        &self,
        index: usize,
        instances: &mut Vec<Instance>,
        animation_handler: &mut AnimationHandler,
    ) {
        if let Some(object) = &self.get_object(index) {
            if object.cubes.len() > instances.len() {
                println!("Object too large to show");
                return;
            }
            let mut instance_indices: Vec<usize> = (0..instances.len()).collect();
            let instance_indices_in_order: Vec<usize> = (0..instances.len()).collect();

            let mut rng = rng();
            instance_indices.shuffle(&mut rng);

            let cubes_indices: Vec<usize> = (0..object.cubes.len()).collect();
            for &i in &cubes_indices {
                let cube = object.cubes.get(i).unwrap();
                let instance_index = instance_indices.pop().unwrap();
                let animation = animation_handler.movement_list.get(instance_index).unwrap();

                let instance = instances.get(instance_index).unwrap();
                let movement_vector = cube - animation.start;
                let animation = AnimationType::Step(AnimationStep::new(
                    movement_vector,
                    false,
                    false,
                    AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                ));
                animation_handler.set_animation(instance_index, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(instance_index, true);
            }
            let instance_indices_remaining: HashSet<usize> =
                instance_indices.iter().cloned().collect();

            // Filter instance_indices to exclude anything in cubes_set
            let remaining_indices: Vec<usize> = instance_indices_in_order
                .clone()
                .into_iter()
                .filter(|i| instance_indices_remaining.contains(i))
                .collect();

            for i in remaining_indices {
                let movement_vector = Vector3 {
                    x: 0.0,
                    y: 40.0,
                    z: 0.0,
                };
                let animation = AnimationType::Step(AnimationStep::new(
                    movement_vector,
                    false,
                    false,
                    AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                ));
                animation_handler.set_animation(i, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(i, true);
            }
        } else {
            println!("Object does not exist!");
            return;
        }
    }
}

fn flatten_index(pos: Vector3<f32>, size: Vector3<usize>) -> usize {
    let x = pos.x.floor() as usize;
    let y = pos.y.floor() as usize;
    let z = pos.z.floor() as usize;

    x + z * size.x + y * size.x * size.z
}
