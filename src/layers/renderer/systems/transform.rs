use crate::prelude::*;
use wgpu::util::DeviceExt;

fn compute_model_matrix(transform: &Transform) -> nalgebra::Matrix4<f32> {
    let translation = nalgebra::Matrix4::new_translation(&transform.position.coords);
    let rotation = transform.rotation.to_homogeneous();
    let scale = nalgebra::Matrix4::new_nonuniform_scaling(&transform.scale);
    translation * rotation * scale
}

pub fn initialize_transform_buffers(
    mut commands: Commands,
    device: Res<GpuDevice>,
    layout: Res<TransformBindGroupLayout>,
    query: Query<(Entity, &Transform), Without<GpuTransform>>,
) {
    for (entity, transform) in query.iter() {
        let model_matrix = compute_model_matrix(transform);

        let buffer = device
            .0
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&[model_matrix]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = device.0.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout.0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("transform_bind_group"),
        });

        commands
            .entity(entity)
            .insert(GpuTransform { buffer, bind_group });
    }
}

pub fn update_transform_buffers(
    queue: Res<GpuQueue>,
    query: Query<(&Transform, &GpuTransform), Changed<Transform>>,
) {
    let queue = &queue.0;

    for (transform, gpu_transform) in query.iter() {
        let model_matrix = compute_model_matrix(transform);
        queue.write_buffer(
            &gpu_transform.buffer,
            0,
            bytemuck::cast_slice(&[model_matrix]),
        );
    }
}
