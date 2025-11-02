use crate::prelude::*;

#[derive(Component)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
}

#[derive(Component)]
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub type Index = u16;

pub fn index_format() -> wgpu::IndexFormat {
    wgpu::IndexFormat::Uint16
}

// GPU Component trait implementations
impl GpuComponent for Mesh {
    type UserComponent = Mesh;
    type GpuVariant = GpuMesh;
}

impl GpuInitialize for Mesh {
    type Dependencies = ();

    fn initialize(
        user: &Self::UserComponent,
        _dependencies: Option<&Self::Dependencies>,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _context: &GpuContext,
    ) -> Self::GpuVariant {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&user.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&user.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: user.indices.len() as u32,
        }
    }
}

impl GpuUpdate for Mesh {
    fn update(
        user: &Self::UserComponent,
        gpu: &mut Self::GpuVariant,
        _dependencies: Option<&()>,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        // Recreate vertex buffer with updated mesh data
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&user.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&user.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Update the GpuMesh component with new buffers
        gpu.vertex_buffer = vertex_buffer;
        gpu.index_buffer = index_buffer;
        gpu.index_count = user.indices.len() as u32;
    }
}
