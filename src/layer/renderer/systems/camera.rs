use wgpu::util::DeviceExt;

use crate::layer::renderer::GpuQueue;
use crate::layer::renderer::components::{Camera, CameraBindGroupLayout, GpuCamera, Transform};
use crate::prelude::*;

use crate::layer::renderer::components::GpuDevice;

#[rustfmt::skip]
const OPENGL_TO_WGPU: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub fn initialize_camera_buffers(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    bind_group_layout: Res<CameraBindGroupLayout>,
    query: Query<(Entity, &Camera, &Transform), Without<GpuCamera>>,
) {
    let device = &device.0;
    let queue = &queue.0;
    let bind_group_layout = &bind_group_layout.0;

    for (entity, camera, transform) in query.iter() {
        let view = Isometry3::look_at_rh(
            &transform.position,
            &camera.target,
            &Unit::new_normalize(transform.up),
        )
        .to_homogeneous();

        // todo: store this in own component so we dont have to do this every time
        let proj = OPENGL_TO_WGPU
            * Perspective3::new(camera.aspect, camera.fovy, camera.znear, camera.zfar)
                .to_homogeneous();

        let matrix = proj * view;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        commands
            .entity(entity)
            .insert(GpuCamera { buffer, bind_group });
    }
}
