use crate::prelude::*;

use wgpu::util::DeviceExt;

pub fn initialize_mesh_buffers(
    mut commands: Commands,
    device: Res<GpuDevice>,
    query: Query<(Entity, &Mesh), Without<GpuMesh>>,
) {
    for (entity, mesh) in query.iter() {
        let vertex_buffer = device
            .0
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = device
            .0
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let gpu_mesh = GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        };

        commands.entity(entity).insert(gpu_mesh);

        log::debug!("Created GpuMesh for Entity {:?}", entity);
    }
}

pub fn update_mesh_buffers(
    device: Res<GpuDevice>,
    mut query: Query<(Entity, &Mesh, &mut GpuMesh), Changed<Mesh>>,
) {
    for (entity, mesh, mut gpu_mesh) in query.iter_mut() {
        // Recreate vertex buffer with updated mesh data
        let vertex_buffer = device
            .0
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Recreate index buffer with updated mesh data
        let index_buffer = device
            .0
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Update the GpuMesh component with new buffers
        gpu_mesh.vertex_buffer = vertex_buffer;
        gpu_mesh.index_buffer = index_buffer;
        gpu_mesh.index_count = mesh.indices.len() as u32;

        log::debug!("Updated GpuMesh for Entity {:?}", entity);
    }
}
