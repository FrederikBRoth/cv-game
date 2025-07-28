use cgmath::{InnerSpace, Point3, Rotation3, Vector2, Vector3};
use winit::dpi::PhysicalPosition;

use crate::{
    core::{camera::Camera, state::State},
    entity::entity::{Instance, InstanceController},
    helpers::animation::AnimationHandler,
};

const STEPSIZE: f32 = 0.1;
const DISTANCE: f32 = 100.0;
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
        // print!("{:?}", world_coord);
        let position = cgmath::Vector3 {
            x: world_x as f32,
            y: world_y as f32,
            z: world_z as f32,
        };

        // state.add_instance(instance, queue, device);
        let result = state.remove_instance_at_pos(world_coord, &queue, chunk_size);
        if result {
            break;
        }
    }
}

pub fn line_trace_remove(
    state: &mut InstanceController,
    queue: &wgpu::Queue,
    click_vector: (Point3<f32>, Vector3<f32>),
) {
    'trace: for n in 0..(DISTANCE / STEPSIZE) as u64 {
        let step = click_vector.0 - (click_vector.1 * (n as f32 * STEPSIZE));

        for instance in state.instances.iter_mut() {
            if (instance.should_render
                && aabb_intersect(&step, &instance.position, &instance.bounding))
            {
                instance.should_render = false;
                state.update_buffer(queue);
                break 'trace;
            }
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
    click_vector: (Point3<f32>, Vector3<f32>),
) {
    'trace: for n in 0..(DISTANCE / STEPSIZE) as u64 {
        let step = click_vector.0 - (click_vector.1 * (n as f32 * STEPSIZE));

        for (index, instance) in state.instances.iter_mut().enumerate() {
            if !instance.should_render {
                continue;
            }
            if (aabb_intersect(&step, &instance.position, &instance.bounding)) {
                let mut animation_end = instance.position.clone();
                animation_end.y = animation_end.y + 1.0;
                animation_handler.set_animation(index, &instance.position, &animation_end);
                animation_handler.reset_animation_time(index);
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
