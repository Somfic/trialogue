use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

use crate::layers::renderer::components::{GpuDevice, GpuMesh, Mesh};

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
