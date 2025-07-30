use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector2, Vector3};

use crate::entity::entity::InstanceController;
use crate::helpers::animation::{AnimationHandler, AnimationType};

const STEPSIZE: f32 = 0.1;
const DISTANCE: f32 = 50.0;
pub fn line_trace_cursor(
    state: &mut InstanceController,
    chunk_size: &Vector2<u32>,
    queue: &wgpu::Queue,
    click_vector: (Point3<f32>, Vector3<f32>),
) {
    for n in 0..(DISTANCE / STEPSIZE) as u64 {
        let step = click_vector.0 - (click_vector.1 * (n as f32 * STEPSIZE));
        let world_x = f32::floor(step.x) as i32;
        let world_y = f32::floor(step.y) as i32;
        let world_z = f32::floor(step.z) as i32;
        let world_coord: Vector3<i32> = Vector3 {
            x: world_x,
            y: world_y,
            z: world_z,
        };

        // state.add_instance(instance, queue, device);
        let result = state.remove_instance_at_pos(world_coord, &queue, chunk_size);
        if result {
            break;
        }
    }
}

// pub fn line_trace_animate_hit(
//     state: &mut InstanceController,
//     animation_handler: &mut AnimationHandler,
//     queue: &wgpu::Queue,
//     click_vector: (Point3<f32>, Vector3<f32>),
// ) {
//     'trace: for n in 0..(DISTANCE / STEPSIZE) as u64 {
//         let step = click_vector.0 - (click_vector.1 * (n as f32 * STEPSIZE));

//         for (index, instance) in state.instances.iter_mut().enumerate() {
//             if !instance.should_render {
//                 continue;
//             }
//             if (aabb_intersect(&step, &instance.position, &instance.bounding)) {
//                 let mut animation_end = instance.position.clone();
//                 animation_end.y = animation_end.y + 1.0;
//                 animation_handler.set_animation(index, &instance.position, &animation_end);
//                 animation_handler.set_animation_state(index, true);
//                 state.update_buffer(queue);
//                 break 'trace;
//             }
//         }
//     }
// }

pub fn line_trace_animate_hit(
    state: &mut InstanceController,
    animation_handler: &mut AnimationHandler,
    queue: &wgpu::Queue,
    animation: AnimationType,
    click_vector: (Point3<f32>, Vector3<f32>),
) {
    'trace: for n in 0..(DISTANCE / STEPSIZE) as u64 {
        let step = click_vector.0 - (click_vector.1 * (n as f32 * STEPSIZE));

        for (index, instance) in state.instances.iter_mut().enumerate() {
            if !instance.should_render {
                continue;
            }
            if aabb_intersect(&step, &instance.position, &instance.bounding) {
                //This will add as many as you can click on. Needs to be taking care of

                animation_handler.set_animation(index, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(index, true);
                break 'trace;
            }
        }
    }
    state.update_buffer(queue);
}

pub fn line_trace_animate_explosion(
    state: &mut InstanceController,
    animation_handler: &mut AnimationHandler,
    queue: &wgpu::Queue,
    animation: AnimationType,
    click_vector: (Point3<f32>, Vector3<f32>),
) {
    'trace: for n in 0..(DISTANCE / STEPSIZE) as u64 {
        let step = click_vector.0 - (click_vector.1 * (n as f32 * STEPSIZE));

        for (index, instance) in state
            .instances
            .iter_mut()
            .filter(|inst| inst.should_render)
            .enumerate()
        {
            if aabb_intersect(&step, &instance.position, &instance.bounding) {
                //This will add as many as you can click on. Needs to be taking care of

                animation_handler.set_animation(index, animation);
                // animation_handler.reset_animation_time(index);
                animation_handler.set_animation_state(index, true);
                break 'trace;
            }
        }
    }
    state.update_buffer(queue);
}

fn aabb_intersect(
    point: &cgmath::Point3<f32>,
    bounding_min: &cgmath::Vector3<f32>,
    bounding_max: &cgmath::Vector3<f32>,
) -> bool {
    return point.x >= bounding_min.x
        && point.x <= bounding_max.x
        && point.y >= bounding_min.y
        && point.y <= bounding_max.y
        && point.z >= bounding_min.z
        && point.z <= bounding_max.z;
}
pub fn line_trace(
    state: &mut InstanceController,
    click_vector: (Point3<f32>, Vector3<f32>),
) -> Option<usize> {
    let origin = click_vector.0;
    //Notice negation of vector
    let direction = click_vector.1.normalize();

    let mut closest_hit_index: Option<usize> = None;
    let mut closest_distance = f32::MAX;

    for (i, instance) in state.instances.iter().enumerate() {
        if (!instance.should_render) {
            continue;
        }
        let max = instance.position + instance.size;
        let min = instance.position;

        if let Some(distance) = ray_aabb_intersect(origin, direction, min, max) {
            // println!("{:?}", i);

            if distance < closest_distance {
                closest_distance = distance;
                closest_hit_index = Some(i);
            }
        }
    }

    closest_hit_index
}
pub fn ray_aabb_intersect(
    origin: Point3<f32>,
    dir: Vector3<f32>,
    min: Vector3<f32>,
    max: Vector3<f32>,
) -> Option<f32> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;

    for i in 0..3 {
        let o = origin[i];
        let d = dir[i];

        if d.abs() < 1e-6 {
            // Ray is parallel to slab
            if o < min[i] || o > max[i] {
                return None;
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (min[i] - o) * inv_d;
            let mut t2 = (max[i] - o) * inv_d;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            tmin = tmin.max(t1);
            tmax = tmax.min(t2);

            if tmin > tmax {
                return None;
            }
        }
    }

    if tmax < 0.0 {
        return None; // Intersection behind ray origin
    }

    Some(if tmin >= 0.0 { tmin } else { tmax }) // Return positive distance
}

pub fn aabb_sphere_intersect(
    aabb_min: Vector3<f32>,
    aabb_max: Vector3<f32>,
    sphere_center: Vector3<f32>,
    sphere_radius: f32,
) -> bool {
    let mut closest_point = sphere_center;

    // Clamp sphere center to the AABB
    if sphere_center.x < aabb_min.x {
        closest_point.x = aabb_min.x;
    } else if sphere_center.x > aabb_max.x {
        closest_point.x = aabb_max.x;
    }

    if sphere_center.y < aabb_min.y {
        closest_point.y = aabb_min.y;
    } else if sphere_center.y > aabb_max.y {
        closest_point.y = aabb_max.y;
    }

    if sphere_center.z < aabb_min.z {
        closest_point.z = aabb_min.z;
    } else if sphere_center.z > aabb_max.z {
        closest_point.z = aabb_max.z;
    }

    // Compute squared distance from sphere center to closest point on AABB
    let distance_squared = (closest_point - sphere_center).magnitude2();

    distance_squared <= sphere_radius * sphere_radius
}
