use crate::prelude::*;
use wgpu::util::DeviceExt;

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
    bind_group_layout: Res<CameraBindGroupLayout>,
    query: Query<(Entity, &Camera, &Transform), Without<GpuCamera>>,
) {
    let device = &device.0;
    let bind_group_layout = &bind_group_layout.0;

    for (entity, camera, transform) in query.iter() {
        // compute the up vector from the rotation quaternion
        let up = transform.rotation * Vector3::y_axis();

        let view = Isometry3::look_at_rh(&transform.position, &camera.target, &up).to_homogeneous();

        // todo: store this in own component so we dont have to do this every time
        let proj = OPENGL_TO_WGPU
            * Perspective3::new(1.0, camera.fovy, camera.znear, camera.zfar).to_homogeneous();

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

        commands.entity(entity).insert(GpuCamera {
            buffer,
            bind_group,
            aspect: 1.0,
        });
    }
}

pub fn initialize_render_targets(
    mut commands: Commands,
    device: Res<GpuDevice>,
    window_size: Res<WindowSize>,
    query: Query<Entity, (With<RenderTarget>, Without<GpuRenderTarget>)>,
) {
    let device = &device.0;

    for entity in query.iter() {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Camera Render Target"),
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        commands.entity(entity).insert(GpuRenderTarget { texture });
    }
}

pub fn update_render_targets(
    mut commands: Commands,
    device: Res<GpuDevice>,
    window_size: Res<WindowSize>,
    mut query: Query<(Entity, &mut GpuCamera, Option<&GpuRenderTarget>), With<RenderTarget>>,
) {
    if !window_size.is_changed() {
        return;
    }

    let device = &device.0;
    let aspect = window_size.width as f32 / window_size.height as f32;

    for (entity, mut camera, gpu_target) in query.iter_mut() {
        // Only update aspect if it actually changed (avoid triggering change detection unnecessarily)
        if (camera.aspect - aspect).abs() > f32::EPSILON {
            camera.aspect = aspect;
        }

        // Recreate render target if it exists
        if gpu_target.is_some() {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Camera Render Target"),
                size: wgpu::Extent3d {
                    width: window_size.width,
                    height: window_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            commands.entity(entity).insert(GpuRenderTarget { texture });
        }
    }
}

pub fn update_camera_buffers(
    queue: Res<GpuQueue>,
    query: Query<(&Camera, &Transform, &GpuCamera), Or<(Changed<GpuCamera>, Changed<Transform>)>>,
) {
    let queue = &queue.0;

    for (camera, transform, gpu_camera) in query.iter() {
        // Compute the up vector from the rotation quaternion
        let up = transform.rotation * Vector3::y_axis();

        let view = Isometry3::look_at_rh(&transform.position, &camera.target, &up).to_homogeneous();

        let proj = OPENGL_TO_WGPU
            * Perspective3::new(gpu_camera.aspect, camera.fovy, camera.znear, camera.zfar)
                .to_homogeneous();

        let matrix = proj * view;

        // Update the existing buffer
        queue.write_buffer(&gpu_camera.buffer, 0, bytemuck::cast_slice(&[matrix]));
    }
}
