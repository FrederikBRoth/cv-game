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
    let direction = click_vector.1.normalize();

    let mut closest_hit_index: Option<usize> = None;
    let mut closest_distance = f32::MAX;

    for (i, instance) in state
        .instances
        .iter()
        .filter(|inst| inst.should_render)
        .enumerate()
    {
        let center = instance.position + instance.size * 0.5;
        let half_size = instance.size * 0.5;

        if let Some(distance) = ray_aabb_intersect(origin, direction, center, half_size) {
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
    direction: Vector3<f32>,
    aabb_center: Vector3<f32>,
    aabb_half_size: Vector3<f32>,
) -> Option<f32> {
    let inv_dir = Vector3::new(direction.x, direction.y, direction.z);
    let inv_dir = Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);

    let min = aabb_center - aabb_half_size;
    let max = aabb_center + aabb_half_size;

    let mut tmin = (min.x - origin.x) * inv_dir.x;
    let mut tmax = (max.x - origin.x) * inv_dir.x;

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    let mut tymin = (min.y - origin.y) * inv_dir.y;
    let mut tymax = (max.y - origin.y) * inv_dir.y;

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if (tmin > tymax) || (tymin > tmax) {
        return None;
    }

    if tymin > tmin {
        tmin = tymin;
    }
    if tymax < tmax {
        tmax = tymax;
    }

    let mut tzmin = (min.z - origin.z) * inv_dir.z;
    let mut tzmax = (max.z - origin.z) * inv_dir.z;

    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
    }

    if (tmin > tzmax) || (tzmin > tmax) {
        return None;
    }

    if tzmin > tmin {
        tmin = tzmin;
    }

    Some(tmin.max(0.0)) // return positive intersection distance
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
