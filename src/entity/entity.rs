use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{
    core::game_loop::Chunk,
    entity::{
        entities::cube::{PrimitiveCube, TexturedCube},
        primitive_texture::PrimitiveTexture,
        texture::Texture,
    },
};
use cgmath::{prelude::*, Vector2, Vector3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, TextureFormat};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}
impl TexturedVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl PrimitiveVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PrimitiveVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub const NUM_INSTANCES_PER_ROW: u32 = 10;
pub const NUM_INSTANCES: u32 = 100;
pub const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32,
    0.0,
    NUM_INSTANCES_PER_ROW as f32,
);

pub struct InstanceController {
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
    pub entity_buffers: MeshBuffer,
    pub buffer_address: u64,
    pub render: Renderer,
    capacity: usize,
    pub count: usize,
    pub atomic_usize: Arc<AtomicUsize>,
}

impl InstanceController {
    pub fn new(
        instances: Vec<Instance>,
        buffer_address: u64,
        entity_buffers: MeshBuffer,
        render: Renderer,
        device: &wgpu::Device,
    ) -> InstanceController {
        let len = instances
            .clone()
            .iter()
            .filter(|instance| instance.should_render)
            .map(Instance::to_raw)
            .collect::<Vec<_>>()
            .len();
        InstanceController {
            buffer_address,
            instances: instances.clone(),
            entity_buffers,
            render,
            capacity: instances.len(),
            atomic_usize: Arc::new(AtomicUsize::new(len)),
            count: len,
            instance_buffer: {
                let instance_data = instances
                    .clone()
                    .iter()
                    .filter(|instance| instance.should_render)
                    .map(Instance::to_raw)
                    .collect::<Vec<_>>();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                })
            },
        }
    }
    fn grow_buffer(
        &mut self,
        _queue: &wgpu::Queue,
        device: &wgpu::Device,

        instance_size: wgpu::BufferAddress,
    ) {
        // New capacity: double the current or start with 4
        let new_capacity = (self.capacity.max(4)) * 2;
        let new_size = instance_size * new_capacity as u64;

        // Create a new larger buffer
        let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer (Resized)"),
            size: new_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Replace old buffer
        self.instance_buffer = new_buffer;
        self.capacity = new_capacity;
    }

    pub fn add_instance(&mut self, instance: Instance, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.instances.push(instance);
        let instance_size = std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress;
        let required = self.instances.len();

        // If we exceed capacity, grow the buffer
        if required > self.capacity {
            self.grow_buffer(queue, device, instance_size);
        }
        let data = self.to_raw();

        queue.write_buffer(
            &self.instance_buffer,
            self.buffer_address,
            bytemuck::cast_slice(&data),
        );
    }

    pub fn remove_instance(&mut self, index: usize, _queue: &wgpu::Queue) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.should_render = false;
            self.count -= 1;
        }
        // let data = self.to_raw();
        // self.count = data.len();
        // queue.write_buffer(
        //     &self.instance_buffer,
        //     self.buffer_address,
        //     bytemuck::cast_slice(&data),
        // );
    }

    pub fn remove_instance_at_pos(
        &mut self,
        pos: Vector3<i32>,
        queue: &wgpu::Queue,
        chunk_size: &Vector2<u32>,
    ) -> bool {
        let grid_x = pos.x;
        let grid_z = pos.z;
        if grid_x < 0
            || grid_x >= chunk_size.x as i32
            || grid_z < 0
            || grid_z >= chunk_size.y as i32
            || pos.y != 0
        {
            return false;
        }

        let index = (grid_z * chunk_size.y as i32 + grid_x) as usize;
        if let Some(instance) = self.instances.get_mut(index) {
            if !instance.should_render {
                println!("Test");
                return false;
            }
        }
        self.remove_instance(index, queue);
        true
    }

    pub fn update_buffer_multithreaded(&mut self, queue: Arc<wgpu::Queue>) {
        let instances = self.instances.clone();
        let instance_buffer = self.instance_buffer.clone(); // If clonable
        let count_clone = Arc::clone(&self.atomic_usize);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let queue = Arc::clone(&queue);
            std::thread::spawn(move || {
                use std::sync::atomic::Ordering;

                let data = instances
                    .iter()
                    .filter(|instance| instance.should_render)
                    .map(Instance::to_raw)
                    .collect::<Vec<InstanceRaw>>();
                queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&data));
                count_clone.store(data.len(), Ordering::Relaxed);
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;

            let queue = queue.clone(); // or Arc
            spawn_local(async move {
                let data = instances
                    .iter()
                    .filter(|instance| instance.should_render)
                    .map(Instance::to_raw)
                    .collect::<Vec<InstanceRaw>>();
                queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&data));
                count_clone.store(data.len(), Ordering::Relaxed);
            });
        }
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue) {
        let data = self.to_raw();
        self.atomic_usize.store(data.len(), Ordering::Relaxed);
        queue.write_buffer(
            &self.instance_buffer,
            self.buffer_address,
            bytemuck::cast_slice(&data),
        );
    }
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, camera_bind: wgpu::BindGroup) {
        render_pass.set_bind_group(0, &camera_bind, &[]);
        render_pass.set_bind_group(1, &self.render.light_bind_group, &[]);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_pipeline(&self.render.pipeline);
        if let Some(diffuse) = &self.render.diffuse {
            render_pass.set_bind_group(2, diffuse, &[]);
        }
        // render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        // for polygon in &self.entity_buffer {
        let polygon = &self.entity_buffers;
        render_pass.set_vertex_buffer(0, polygon.vertex_buffer.slice(..));
        render_pass.set_index_buffer(polygon.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..polygon.num_indices,
            0,
            0..(*(&self.atomic_usize.load(Ordering::Relaxed).clone()) as usize) as _,
        );
    }

    fn to_raw(&mut self) -> Vec<InstanceRaw> {
        self.instances
            .clone()
            .iter()
            .filter(|instance| instance.should_render) // only include visible instances
            .map(Instance::to_raw)
            .collect()
    }
}

#[derive(Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub should_render: bool,
    pub scale: f32,
    pub color: cgmath::Vector3<f32>,
    pub size: cgmath::Vector3<f32>,
    pub bounding: cgmath::Vector3<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: ((cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
                * self.scale)
                .into(),
            color: cgmath::Vector3::from(self.color).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    pub model: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub normal: [[f32; 3]; 3],
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 25]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub mesh_type: MeshType,
}

pub enum MeshType {
    Primitive,
    Textured,
}

pub struct Renderer {
    pub pipeline: wgpu::RenderPipeline,
    pub light_bind_group: wgpu::BindGroup,
    pub diffuse: Option<wgpu::BindGroup>,
}
pub struct TexturedMesh {
    pub vertices: Vec<TexturedVertex>,
    pub indices: Vec<u16>,
    pub texture_bytes: Vec<u8>,
}

impl TexturedMesh {
    pub fn get_mesh_buffer(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: TextureFormat,
        queue: &wgpu::Queue,
        camera_bind_group_layout: BindGroupLayout,
        light_bind_group_layout: BindGroupLayout,
        light_bind_group: BindGroup,
        size: PhysicalSize<u32>,
    ) -> (MeshBuffer, Renderer) {
        let diffuse_bytes = &self.texture_bytes;
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();
        log::warn!("Texture");

        // Create bind group layout for texture and sampler
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // Create bind group for the texture
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TexturedVertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let mb = MeshBuffer {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            num_indices: self.indices.len() as u32,
            mesh_type: MeshType::Textured,
        };

        let render = Renderer {
            diffuse: Some(diffuse_bind_group),
            light_bind_group,
            pipeline: render_pipeline,
        };

        (mb, render)
    }
}
pub struct PrimitiveMesh {
    pub vertices: Vec<PrimitiveVertex>,
    pub indices: Vec<u16>,
}

impl PrimitiveMesh {
    pub fn get_mesh_buffer(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: TextureFormat,
        _queue: &wgpu::Queue,
        camera_bind_group_layout: BindGroupLayout,
        light_bind_group_layout: BindGroupLayout,
        light_bind_group: BindGroup,
        size: PhysicalSize<u32>,
    ) -> (MeshBuffer, Renderer) {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
        let test = wgpu::DepthBiasState::default();
        println!("{:?}", test);
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[PrimitiveVertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // standard depth test
                stencil: wgpu::StencilState::default(),     // no stencil operations
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        let mb = MeshBuffer {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            num_indices: self.indices.len() as u32,
            mesh_type: MeshType::Primitive,
        };
        let renderer = Renderer {
            pipeline: render_pipeline,
            light_bind_group,
            diffuse: None,
        };

        (mb, renderer)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

pub struct Light {
    pub position: Vector3<f32>,
    color: Vector3<f32>,
    pub instance_controller: Option<InstanceController>,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>, device: &wgpu::Device) -> Self {
        let uniform = LightUniform {
            position: cgmath::Vector3::from(position.clone()).into(),
            _padding: 0,
            color: cgmath::Vector3::from(color.clone()).into(),
            _padding2: 0,
        };
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            position,
            color,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
            instance_controller: None,
        }
    }

    pub fn to_raw(&self) -> LightUniform {
        LightUniform {
            position: cgmath::Vector3::from(self.position).into(),
            _padding: 0,
            color: cgmath::Vector3::from(self.color).into(),
            _padding2: 0,
        }
    }
}

pub fn make_cube_textured() -> TexturedMesh {
    let cube = TexturedCube::new();

    let polygon: TexturedMesh = TexturedMesh {
        vertices: cube.vertices,
        indices: cube.indices,
        texture_bytes: include_bytes!("../happy-tree.png").to_vec(),
    };
    polygon
}

pub fn make_cube_primitive() -> PrimitiveMesh {
    let cube = PrimitiveCube::new();
    let polygon: PrimitiveMesh = PrimitiveMesh {
        vertices: cube.vertices,
        indices: cube.indices,
    };

    polygon
}

pub fn instances_list(chunk: Chunk, chunk_size: Vector2<u32>) -> Vec<Instance> {
    (0..(chunk_size.x * chunk_size.y))
        .map(move |n| {
            let x = n % chunk_size.x;
            let z = n / chunk_size.y;
            let position = cgmath::Vector3 {
                x: x as f32 + (chunk.x * chunk_size.x as i32) as f32,
                y: 0.0,
                z: z as f32 + (chunk.y * chunk_size.y as i32) as f32,
            };

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
            };
            let default_color = cgmath::Vector3::new(1.0, 0.0, 0.0);
            let default_size = cgmath::Vector3::new(1.0, 1.0, 1.0);
            let default_bounding = default_size + position;

            Instance {
                position,
                rotation,
                scale: 0.5,
                should_render: true,
                color: default_color,
                size: default_size,
                bounding: default_bounding,
            }
        })
        .collect::<Vec<_>>()
}
pub fn instances_list_cube(chunk: Chunk, chunk_size: Vector3<u32>) -> Vec<Instance> {
    (0..(chunk_size.x * chunk_size.y * chunk_size.z))
        .map(move |n| {
            let x = n % chunk_size.x;
            let z = (n / chunk_size.x) % chunk_size.z;
            let y = n / (chunk_size.x * chunk_size.z);

            let position = cgmath::Vector3 {
                x: x as f32 + (chunk.x * chunk_size.x as i32) as f32,
                y: y as f32,
                z: z as f32 + (chunk.y * chunk_size.z as i32) as f32,
            };

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
            };
            let default_color = cgmath::Vector3::new(1.0, 0.0, 0.0);
            let default_size = cgmath::Vector3::new(1.0, 1.0, 1.0);
            let default_bounding = default_size + position;

            Instance {
                position,
                rotation,
                scale: 0.5,
                should_render: true,
                color: default_color,
                size: default_size,
                bounding: default_bounding,
            }
        })
        .collect::<Vec<_>>()
}

pub fn instance_cube(position: Vector3<f32>) -> Instance {
    let rotation = if position.is_zero() {
        // this is needed so an object at (0, 0, 0) won't get scaled to zero
        // as Quaternions can effect scale if they're not created correctly
        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
    } else {
        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
    };
    let default_color = cgmath::Vector3::new(1.0, 0.0, 0.0);
    let default_size = cgmath::Vector3::new(1.0, 1.0, 1.0);
    let default_bounding = default_size + position;

    Instance {
        position,
        rotation,
        scale: 0.5,
        should_render: true,
        color: default_color,
        size: default_size,
        bounding: default_bounding,
    }
}
pub fn instances_list_circle(chunk: Chunk, chunk_size: Vector2<u32>) -> Vec<Instance> {
    let center = (chunk_size.x / 2, chunk_size.y / 2);
    let radius = center.0 as i32;
    (0..(chunk_size.x * chunk_size.y))
        .map(move |n| {
            let x = n % chunk_size.x;
            let z = n / chunk_size.y;

            let dx = x as i32 - center.0 as i32;
            let dy = z as i32 - center.1 as i32;

            let distance_squared = dx * dx + dy * dy;
            let position = cgmath::Vector3 {
                x: x as f32 + (chunk.x * chunk_size.x as i32) as f32,
                y: 0.0,
                z: z as f32 + (chunk.y * chunk_size.y as i32) as f32,
            };

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
            };
            let default_color = cgmath::Vector3::new(1.0, 0.0, 0.0);
            let default_size = cgmath::Vector3::new(1.0, 1.0, 1.0);
            let default_bounding = default_size + position;

            if distance_squared > radius * radius
                || x == 0
                || x == radius as u32
                || z == 0
                || z == radius as u32
            {
                Instance {
                    position,
                    rotation,
                    scale: 0.5,
                    should_render: false,
                    color: default_color,
                    size: default_size,
                    bounding: default_bounding,
                }
            } else {
                Instance {
                    position,
                    rotation,
                    scale: 0.5,
                    should_render: true,
                    color: default_color,
                    size: default_size,
                    bounding: default_bounding,
                }
            }
        })
        .collect::<Vec<_>>()
}
pub fn instances_list_cylinder(chunk: Chunk, chunk_size: Vector3<u32>) -> Vec<Instance> {
    let center = (chunk_size.x / 2, chunk_size.z / 2);
    let radius = center.0 as i32;
    (0..(chunk_size.x * chunk_size.y * chunk_size.z))
        .map(move |n| {
            let x = n % chunk_size.x;
            let z = (n / chunk_size.x) % chunk_size.z;
            let y = n / (chunk_size.x * chunk_size.z);

            let dx = x as i32 - center.0 as i32;
            let dy = z as i32 - center.1 as i32;

            let distance_squared = dx * dx + dy * dy;
            let position = cgmath::Vector3 {
                x: x as f32 + (chunk.x * chunk_size.x as i32) as f32,
                y: y as f32,
                z: z as f32 + (chunk.y * chunk_size.z as i32) as f32,
            };

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
            };
            let default_color = cgmath::Vector3::new(1.0, 0.0, 0.0);
            let default_size = cgmath::Vector3::new(1.0, 1.0, 1.0);
            let default_bounding = default_size + position;

            if distance_squared > radius * radius
                || x == 0
                || x == radius as u32
                || z == 0
                || z == radius as u32
            {
                Instance {
                    position,
                    rotation,
                    scale: 0.5,
                    should_render: false,
                    color: default_color,
                    size: default_size,
                    bounding: default_bounding,
                }
            } else {
                Instance {
                    position,
                    rotation,
                    scale: 0.5,
                    should_render: true,
                    color: default_color,
                    size: default_size,
                    bounding: default_bounding,
                }
            }
        })
        .collect::<Vec<_>>()
}
pub fn instances_list2() -> Vec<Instance> {
    (0..NUM_INSTANCES)
        .map(move |n| {
            let x = n % NUM_INSTANCES_PER_ROW;
            let z = n / NUM_INSTANCES_PER_ROW;
            let position = cgmath::Vector3 {
                x: x as f32 + 10.0,
                y: 0.0,
                z: z as f32 + 10.0,
            };

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
            };

            let default_color = cgmath::Vector3::new(1.0, 0.0, 0.0);
            let default_size = cgmath::Vector3::new(1.0, 1.0, 1.0);
            let default_bounding = default_size + position;

            Instance {
                position,
                rotation,
                scale: 0.5,
                should_render: true,
                color: default_color,
                size: default_size,
                bounding: default_bounding,
            }
        })
        .collect::<Vec<_>>()
    // Vec::new()
}
