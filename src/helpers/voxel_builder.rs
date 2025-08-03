use crate::helpers::animation::AnimationType;
use crate::helpers::animation::{
    AnimationHandler, AnimationStep, AnimationTransition, EaseInEaseOut,
};
use cgmath::{MetricSpace, Vector3};
use dot_vox::load_bytes;
use rand::rng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

pub struct Object {
    pub cubes: Vec<Vector3<f32>>,
    pub color: Vec<Vector3<f32>>,
}

pub struct VoxelHandler<T: Eq + std::hash::Hash> {
    pub voxels: Vec<Object>,
    pub voxels_map: HashMap<T, Object>,
    pub current_voxel: Option<T>,
    pub current_cubes: Vec<usize>,
    pub current_object: usize,
}
impl<T: Eq + std::hash::Hash + Clone> VoxelHandler<T> {
    pub fn new() -> Self {
        Self {
            voxels: vec![],
            voxels_map: HashMap::new(),
            current_cubes: vec![],
            current_object: 0,
            current_voxel: None,
        }
    }

    pub fn add_voxel(&mut self, path: &[u8], voxel_type: T) {
        match load_bytes(path) {
            Ok(scene) => {
                let palette = scene.palette.clone();
                for model in scene.models {
                    let new_voxel = Object {
                        cubes: model
                            .voxels
                            .clone()
                            .iter()
                            .map(|v| Vector3::new(v.x as f32, v.z as f32, v.y as f32))
                            .collect(),
                        color: model
                            .voxels
                            .clone()
                            .iter()
                            .map(|v| {
                                let color = palette.get(v.i as usize).unwrap();
                                Vector3::new(
                                    get_srgb(color.r),
                                    get_srgb(color.g),
                                    get_srgb(color.b),
                                )
                            })
                            .collect(),
                    };
                    self.voxels_map.insert(voxel_type.clone(), new_voxel);
                }
            }
            Err(err) => {
                log::warn!("Failed to load voxel file");
                eprintln!("Failed to load .vox file: {}", err);
            }
        }
    }

    pub fn get_object(&self, current_object: T) -> Option<&Object> {
        if let Some(object) = &self.voxels_map.get(&current_object) {
            Some(*&object)
        } else {
            None
        }
    }

    pub fn explode_object(&mut self, animation_handler: &mut AnimationHandler, amplify: f32) {
        let current_voxel = self.current_voxel.as_mut().unwrap().clone();
        self.transition_to_object_base(current_voxel, animation_handler, amplify, false, false);
    }
    pub fn transition_to_object(&mut self, object: T, animation_handler: &mut AnimationHandler) {
        self.transition_to_object_base(object.clone(), animation_handler, 1.0, true, false);
    }

    fn transition_to_object_base(
        &mut self,
        object: T,
        animation_handler: &mut AnimationHandler,
        amplify: f32,
        is_onetime: bool,
        use_object_color: bool,
    ) {
        self.current_voxel = Some(object.clone());
        let mut current_cubes = self.current_cubes.clone();
        let mut new_current_cubes: Vec<usize> = vec![];

        if let Some(object) = self.get_object(object.clone()) {
            if object.cubes.len() > animation_handler.movement_list.len() {
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
                if use_object_color {
                    let &color = object.color.get(i).unwrap();

                    animation_handler.set_manual_animation_color(instance_index, color);
                }
                animation_handler.set_animation(instance_index, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(instance_index, true);
            }
            let instance_indices_remaining: HashSet<usize> =
                new_current_cubes.iter().cloned().collect();

            // Filter instance_indices to exclude anything in cubes_set
            let remaining_indices: Vec<usize> = instance_indices_in_order
                .clone()
                .into_iter()
                .filter(|i| !instance_indices_remaining.contains(i))
                .collect();

            let mut circle = fibonacci_sphere(remaining_indices.clone().len(), 750.0);

            for i in remaining_indices {
                let animation = animation_handler.movement_list.get(i).unwrap();

                let point = circle.pop().unwrap();
                let mut movement_vector = Vector3::new(0.0, 0.0, 0.0);
                if animation.grid_pos.distance(Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }) <= 500.0
                {
                    movement_vector = Vector3::new(point.x, point.y, point.z);
                    movement_vector = movement_vector - animation.grid_pos;
                }
                let animation = AnimationType::Step(AnimationStep::new(
                    movement_vector,
                    0.25,
                    false,
                    false,
                    is_onetime,
                    AnimationTransition::EaseInEaseOut(EaseInEaseOut),
                ));

                animation_handler.set_animated_color(i);
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

fn get_srgb(color: u8) -> f32 {
    ((color as f32 / 255 as f32 + 0.055) / 1.055).powf(2.4)
}
