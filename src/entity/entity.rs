use std::u32;

use crate::entity::entities::cube::Cube;
use cgmath::prelude::*;
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}
impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub const NUM_INSTANCES_PER_ROW: u32 = 10;
pub const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct InstanceController {
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
    pub entity_buffers: Vec<MeshBuffer>,
    pub buffer_address: u64,
}

impl InstanceController {
    pub fn new(
        instances: Vec<Instance>,
        buffer_address: u64,
        entity_buffers: Vec<MeshBuffer>,
        device: &wgpu::Device,
    ) -> InstanceController {
        InstanceController {
            buffer_address,
            instances: instances.clone(),
            entity_buffers,
            instance_buffer: {
                let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                })
            },
        }
    }

    pub fn add_instance(&mut self, instance: Instance, queue: &wgpu::Queue) {
        self.instances.push(instance);

        let data = self.to_raw();

        queue.write_buffer(
            &self.instance_buffer,
            self.buffer_address,
            bytemuck::cast_slice(&data),
        );
    }

    pub fn remove_instance(&mut self, index: usize, queue: &wgpu::Queue) {
        self.instances.remove(index);
        let data = self.to_raw();
        queue.write_buffer(
            &self.instance_buffer,
            self.buffer_address,
            bytemuck::cast_slice(&data),
        );
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue) {
        let data = self.to_raw();
        queue.write_buffer(
            &self.instance_buffer,
            self.buffer_address,
            bytemuck::cast_slice(&data),
        );
    }

    fn to_raw(&mut self) -> Vec<InstanceRaw> {
        self.instances
            .clone()
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>()
    }
}

pub fn instances_list() -> Vec<Instance> {
    (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 {
                    x: x as f32,
                    y: 0.0,
                    z: z as f32,
                } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
                };

                Instance {
                    position,
                    rotation,
                    scale: 0.5,
                }
            })
        })
        .collect::<Vec<_>>()
}

#[derive(Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: f32,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: ((cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
                * self.scale)
                .into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}
pub fn make_cube(device: &wgpu::Device) -> MeshBuffer {
    let cube = Cube::new();
    let polygon: Mesh = Mesh {
        vertices: cube.vertices,
        indices: cube.indices,
    };

    MeshBuffer {
        vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&polygon.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&polygon.indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        num_indices: polygon.indices.len() as u32,
    }
}

pub fn make_pentagon(device: &wgpu::Device) -> MeshBuffer {
    let mut polygon: Mesh = Mesh {
        vertices: vec![],
        indices: vec![0, 1, 4, 1, 2, 4, 2, 3, 4],
    };

    polygon.vertices.append(&mut vec![
        Vertex {
            position: [-0.0868241, 0.49240386, 0.0],
            tex_coords: [0.4131759, 1.0 - 0.99240386],
        }, // A
        Vertex {
            position: [-0.49513406, 0.06958647, 0.0],
            tex_coords: [0.0048659444, 1.0 - 0.56958647],
        }, // B
        Vertex {
            position: [-0.21918549, -0.44939706, 0.0],
            tex_coords: [0.28081453, 1.0 - 0.05060294],
        }, // C
        Vertex {
            position: [0.35966998, -0.3473291, 0.0],
            tex_coords: [0.85967, 1.0 - 0.1526709],
        }, // D
        Vertex {
            position: [0.44147372, 0.2347359, 0.0],
            tex_coords: [0.9414737, 1.0 - 0.7347359],
        }, // E
    ]);

    MeshBuffer {
        vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&polygon.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&polygon.indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        num_indices: polygon.indices.len() as u32,
    }
}
