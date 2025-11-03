use crate::prelude::*;

use wgpu::util::DeviceExt;

/// Custom camera update system that handles aspect ratio changes from GpuCamera
/// This supplements the trait-based system by watching for GpuCamera changes too
pub fn update_camera_buffers_custom(
    queue: Res<GpuQueue>,
    query: Query<
        (&Camera, &Transform, &GpuCamera),
        Or<(Changed<Camera>, Changed<Transform>, Changed<GpuCamera>)>,
    >,
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

        queue.write_buffer(&gpu_camera.buffer, 0, bytemuck::cast_slice(&[matrix]));
    }
}

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

        queue.write_buffer(&gpu_camera.buffer, 0, bytemuck::cast_slice(&[matrix]));
    }
}

pub fn initialize_depth_textures(
    mut commands: Commands,
    device: Res<GpuDevice>,
    window_size: Res<WindowSize>,
    query: Query<Entity, (With<RenderTarget>, Without<GpuDepthTexture>)>,
) {
    let device = &device.0;

    for entity in query.iter() {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        commands
            .entity(entity)
            .insert(GpuDepthTexture { texture, view });
    }
}

pub fn update_depth_textures(
    mut commands: Commands,
    device: Res<GpuDevice>,
    window_size: Res<WindowSize>,
    query: Query<(Entity, Option<&GpuDepthTexture>), With<RenderTarget>>,
) {
    if !window_size.is_changed() {
        return;
    }

    let device = &device.0;

    for (entity, gpu_depth) in query.iter() {
        // Recreate depth texture if it exists
        if gpu_depth.is_some() {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: window_size.width,
                    height: window_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            commands
                .entity(entity)
                .insert(GpuDepthTexture { texture, view });
        }
    }
}

pub fn initialize_shadow_maps(
    mut commands: Commands,
    device: Res<GpuDevice>,
    shadow_layout: Res<ShadowBindGroupLayout>,
    shadow_uniform_layout: Res<ShadowUniformLayout>,
    camera_query: Query<Entity, (With<RenderTarget>, Without<GpuShadowMap>)>,
    light_query: Query<(&Light, &Transform), With<Light>>,
) {
    use wgpu::util::DeviceExt;

    let device = &device.0;
    let shadow_map_size = 4096u32; // Higher resolution for smoother shadows

    // Get the first light, or use default if none exists
    let (light_pos, light_dir, light_intensity, light_color) = if let Some((light, light_transform)) = light_query.iter().next() {
        let pos = light_transform.position;
        let dir = pos.coords.normalize(); // Treat light position as direction from origin
        (pos, dir, light.intensity, light.color)
    } else {
        // Default light direction
        let dir = nalgebra::Vector3::new(0.5f32, 1.0, 0.3).normalize();
        let pos = Point3::from(-dir * 5.0);
        (pos, dir, 1.0, [1.0, 1.0, 1.0])
    };

    for entity in camera_query.iter() {
        // Create shadow map texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: shadow_map_size,
                height: shadow_map_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create comparison sampler for shadow testing
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // Create light projection matrix (orthographic for directional light)
        let light_target = nalgebra::Point3::origin();
        let light_up = nalgebra::Vector3::y();

        let light_view =
            nalgebra::Isometry3::look_at_rh(&light_pos.into(), &light_target, &light_up)
                .to_homogeneous();

        // Orthographic projection to cover the planet
        // Make it large enough to cover the planet but not too large (loses precision)
        let light_proj = nalgebra::Orthographic3::new(-3.0, 3.0, -3.0, 3.0, 1.0, 20.0)
            .to_homogeneous();

        let light_space_matrix = OPENGL_TO_WGPU * light_proj * light_view;

        // Create uniform buffer for light space matrix
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Space Matrix Buffer"),
            contents: bytemuck::cast_slice(&[light_space_matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create uniform buffer for light direction (vec4 for alignment)
        let light_dir_padded = [light_dir.x, light_dir.y, light_dir.z, 0.0f32];
        let light_dir_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Direction Buffer"),
            contents: bytemuck::cast_slice(&light_dir_padded),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create uniform buffer for light properties (intensity + color, aligned to vec4)
        let light_properties = [light_intensity, light_color[0], light_color[1], light_color[2]];
        let light_properties_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Properties Buffer"),
            contents: bytemuck::cast_slice(&light_properties),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group for main pass (includes texture, sampler, light matrix, light direction, and light properties)
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_layout.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_dir_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: light_properties_buffer.as_entire_binding(),
                },
            ],
            label: Some("shadow_bind_group"),
        });

        // Create bind group for shadow pass (only light matrix)
        let shadow_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_uniform_layout.0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("shadow_uniform_bind_group"),
        });

        commands.entity(entity).insert(GpuShadowMap {
            texture,
            view,
            sampler,
            bind_group,
            light_buffer,
            light_dir_buffer,
            light_properties_buffer,
            shadow_uniform_bind_group,
            light_dir,
            light_intensity,
            light_color,
        });
    }
}

pub fn update_shadow_maps(
    queue: Res<GpuQueue>,
    light_query: Query<(&Light, &Transform), Or<(Changed<Light>, Changed<Transform>)>>,
    mut shadow_query: Query<&mut GpuShadowMap>,
) {
    // Only update if light changed
    if light_query.is_empty() {
        return;
    }

    // Get the first light
    let (light, light_transform) = if let Some(l) = light_query.iter().next() {
        l
    } else {
        return;
    };

    let light_pos = light_transform.position;
    let light_dir = light_pos.coords.normalize();

    for mut shadow_map in shadow_query.iter_mut() {
        // Check if anything changed
        let dir_changed = (shadow_map.light_dir - light_dir).norm() > 0.001;
        let intensity_changed = (shadow_map.light_intensity - light.intensity).abs() > 0.001;
        let color_changed = shadow_map.light_color != light.color;

        if !dir_changed && !intensity_changed && !color_changed {
            continue;
        }

        // Update stored values
        shadow_map.light_dir = light_dir;
        shadow_map.light_intensity = light.intensity;
        shadow_map.light_color = light.color;

        // Recalculate light space matrix if direction changed
        if dir_changed {
            let light_target = nalgebra::Point3::origin();
            let light_up = nalgebra::Vector3::y();

            let light_view =
                nalgebra::Isometry3::look_at_rh(&light_pos.into(), &light_target, &light_up)
                    .to_homogeneous();

            let light_proj = nalgebra::Orthographic3::new(-3.0, 3.0, -3.0, 3.0, 1.0, 20.0)
                .to_homogeneous();

            let light_space_matrix = OPENGL_TO_WGPU * light_proj * light_view;

            queue
                .0
                .write_buffer(&shadow_map.light_buffer, 0, bytemuck::cast_slice(&[light_space_matrix]));

            let light_dir_padded = [light_dir.x, light_dir.y, light_dir.z, 0.0f32];
            queue
                .0
                .write_buffer(&shadow_map.light_dir_buffer, 0, bytemuck::cast_slice(&light_dir_padded));
        }

        // Update light properties if intensity or color changed
        if intensity_changed || color_changed {
            let light_properties = [light.intensity, light.color[0], light.color[1], light.color[2]];
            queue
                .0
                .write_buffer(&shadow_map.light_properties_buffer, 0, bytemuck::cast_slice(&light_properties));
        }
    }
}
