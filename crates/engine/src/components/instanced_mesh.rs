use crate::prelude::*;

/// Component for LOD meshes that use GPU instancing
/// Single entity can render many instances with different transforms
#[derive(Component)]
pub struct InstancedLodMesh {
    /// Base mesh template (all instances share this geometry)
    pub base_mesh: Mesh,
    /// Active chunks in the LOD quadtree
    pub chunks: Vec<LodChunk>,
    /// Needs GPU buffer update
    pub dirty: bool,
}

impl InstancedLodMesh {
    pub fn new(base_mesh: Mesh) -> Self {
        Self {
            base_mesh,
            chunks: Vec::new(),
            dirty: true,
        }
    }

    /// Get list of visible chunks for rendering
    pub fn visible_chunks(&self) -> Vec<&LodChunk> {
        self.chunks.iter().filter(|c| c.visible).collect()
    }

    /// Mark as needing GPU update
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

/// Represents one chunk in the LOD quadtree
#[derive(Clone, Debug)]
pub struct LodChunk {
    /// Bounds: (x_min, x_max, z_min, z_max) for quads, or (u_min, u_max, v_min, v_max) for spheres
    pub bounds: (f32, f32, f32, f32),
    /// Current depth in the quadtree
    pub depth: u32,
    /// World-space center point (for distance calculations)
    pub center: Point3<f32>,
    /// World transform for this instance
    pub transform: Matrix4<f32>,
    /// Whether this chunk should be rendered
    pub visible: bool,
    /// Child chunk indices (if subdivided)
    pub children: Option<[usize; 4]>,
}

impl LodChunk {
    pub fn new(bounds: (f32, f32, f32, f32), depth: u32, center: Point3<f32>, transform: Matrix4<f32>) -> Self {
        Self {
            bounds,
            depth,
            center,
            transform,
            visible: true,
            children: None,
        }
    }
}

/// GPU component for instanced rendering
#[derive(Component)]
pub struct GpuInstancedLodMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
    pub index_count: u32,
}

/// Per-instance data sent to GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// 4x4 transform matrix (stored as 4 vec4s for alignment)
    pub model_matrix: [[f32; 4]; 4],
}

impl InstanceData {
    pub fn from_matrix(matrix: &Matrix4<f32>) -> Self {
        // Convert Matrix4 to [[f32; 4]; 4]
        // Extract matrix data directly - nalgebra stores in column-major order
        let m = matrix;
        Self {
            model_matrix: [
                [m.m11, m.m21, m.m31, m.m41],
                [m.m12, m.m22, m.m32, m.m42],
                [m.m13, m.m23, m.m33, m.m43],
                [m.m14, m.m24, m.m34, m.m44],
            ],
        }
    }

    /// Vertex buffer layout for instance data
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model matrix (4 vec4s)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// GPU Component trait implementations
impl GpuComponent for InstancedLodMesh {
    type UserComponent = InstancedLodMesh;
    type GpuVariant = GpuInstancedLodMesh;
}

impl GpuInitialize for InstancedLodMesh {
    type Dependencies = ();

    fn initialize(
        user: &Self::UserComponent,
        _dependencies: Option<&Self::Dependencies>,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _context: &GpuContext,
    ) -> Self::GpuVariant {
        use wgpu::util::DeviceExt;

        // Create vertex and index buffers from base mesh
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instanced Vertex Buffer"),
            contents: bytemuck::cast_slice(&user.base_mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instanced Index Buffer"),
            contents: bytemuck::cast_slice(&user.base_mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create instance buffer with visible chunks
        let visible_chunks = user.visible_chunks();
        let instance_data: Vec<InstanceData> = visible_chunks
            .iter()
            .map(|chunk| InstanceData::from_matrix(&chunk.transform))
            .collect();

        let instance_buffer = if instance_data.is_empty() {
            // Create empty buffer
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: std::mem::size_of::<InstanceData>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        };

        let result = GpuInstancedLodMesh {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_count: visible_chunks.len() as u32,
            index_count: user.base_mesh.indices.len() as u32,
        };
        
        log::info!("Initialized GpuInstancedLodMesh: {} instances, {} indices, {} vertices",
            result.instance_count, result.index_count, user.base_mesh.vertices.len());
        
        result
    }
}

impl GpuUpdate for InstancedLodMesh {
    fn update(
        user: &Self::UserComponent,
        gpu: &mut Self::GpuVariant,
        _dependencies: Option<&()>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // Only update if marked dirty
        if !user.dirty {
            return;
        }

        // Rebuild instance buffer with current visible chunks
        let visible_chunks = user.visible_chunks();
        let instance_data: Vec<InstanceData> = visible_chunks
            .iter()
            .map(|chunk| InstanceData::from_matrix(&chunk.transform))
            .collect();

        gpu.instance_count = visible_chunks.len() as u32;

        if instance_data.is_empty() {
            return;
        }

        // Check if we need to recreate buffer (size changed significantly)
        let needed_size = (instance_data.len() * std::mem::size_of::<InstanceData>()) as u64;
        let current_size = gpu.instance_buffer.size();

        if needed_size > current_size {
            // Recreate larger buffer
            use wgpu::util::DeviceExt;
            gpu.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            // Update existing buffer
            queue.write_buffer(&gpu.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
        }

        log::debug!("Updated instance buffer with {} instances", gpu.instance_count);
    }
}
