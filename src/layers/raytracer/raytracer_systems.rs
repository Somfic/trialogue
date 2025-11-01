use crate::prelude::*;
use encase::{StorageBuffer, UniformBuffer};
use image::ImageDecoder;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use wgpu::util::DeviceExt;

/// System to collect all spheres and lights and create/update the GPU scene buffer
pub fn update_raytracer_scene(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    sphere_query: Query<(&Sphere, &Transform)>,
    light_query: Query<(&Light, &Transform)>,
    changed_spheres: Query<&Sphere, Or<(Changed<Sphere>, Changed<Transform>)>>,
    changed_lights: Query<&Light, Or<(Changed<Light>, Changed<Transform>)>>,
    mut scene_query: Query<(Entity, &mut GpuRaytracerScene)>,
) {
    // Check if any spheres or lights have changed
    let spheres_changed = !changed_spheres.is_empty();
    let lights_changed = !changed_lights.is_empty();

    // Collect all spheres (position from Transform, radius from Transform.scale.x)
    let spheres: Vec<RaytracerSphere> = sphere_query
        .iter()
        .map(|(sphere, transform)| RaytracerSphere {
            center: Vector3::new(
                transform.position.x,
                transform.position.y,
                transform.position.z,
            ),
            radius: transform.scale.x, // Use x component of scale as radius
            color: Vector3::from_row_slice(&sphere.color),
            material_type: sphere.material_type,
        })
        .collect();

    // Collect all lights (position from Transform)
    let lights: Vec<RaytracerLight> = light_query
        .iter()
        .map(|(light, transform)| RaytracerLight {
            position: Vector3::new(
                transform.position.x,
                transform.position.y,
                transform.position.z,
            ),
            intensity: light.intensity,
            color: Vector3::from_row_slice(&light.color),
        })
        .collect();

    let sphere_count = spheres.len() as u32;
    let light_count = lights.len() as u32;

    // If no scene entity exists, create one
    if scene_query.iter().count() == 0 {
        if sphere_count > 0 || light_count > 0 {
            let mut spheres_data = StorageBuffer::new(Vec::new());
            spheres_data.write(&spheres).unwrap();
            let spheres_bytes = spheres_data.into_inner();

            let spheres_buffer = device
                .0
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Raytracer Spheres Buffer"),
                    contents: &spheres_bytes,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

            let mut lights_data = StorageBuffer::new(Vec::new());
            lights_data.write(&lights).unwrap();
            let lights_bytes = lights_data.into_inner();

            let lights_buffer = device
                .0
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Raytracer Lights Buffer"),
                    contents: &lights_bytes,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

            let scene = GpuRaytracerScene {
                spheres_buffer,
                lights_buffer,
                sphere_count,
                light_count,
            };

            commands.spawn(scene);
            log::debug!(
                "Created GpuRaytracerScene with {} spheres and {} lights",
                sphere_count,
                light_count
            );
        }
    } else {
        // Update existing scene
        for (_entity, mut scene) in scene_query.iter_mut() {
            let count_changed =
                scene.sphere_count != sphere_count || scene.light_count != light_count;

            if count_changed {
                // Recreate buffers if counts changed
                let mut spheres_data = StorageBuffer::new(Vec::new());
                spheres_data.write(&spheres).unwrap();
                let spheres_bytes = spheres_data.into_inner();

                scene.spheres_buffer =
                    device
                        .0
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Raytracer Spheres Buffer"),
                            contents: &spheres_bytes,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        });

                let mut lights_data = StorageBuffer::new(Vec::new());
                lights_data.write(&lights).unwrap();
                let lights_bytes = lights_data.into_inner();

                scene.lights_buffer =
                    device
                        .0
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Raytracer Lights Buffer"),
                            contents: &lights_bytes,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        });

                scene.sphere_count = sphere_count;
                scene.light_count = light_count;

                log::debug!(
                    "Recreated buffers - {} spheres and {} lights",
                    sphere_count,
                    light_count
                );
            } else if spheres_changed || lights_changed {
                // Update buffer data if properties changed but counts are the same
                if spheres_changed && !spheres.is_empty() {
                    let mut spheres_data = StorageBuffer::new(Vec::new());
                    spheres_data.write(&spheres).unwrap();
                    queue
                        .0
                        .write_buffer(&scene.spheres_buffer, 0, &spheres_data.into_inner());
                }

                if lights_changed && !lights.is_empty() {
                    let mut lights_data = StorageBuffer::new(Vec::new());
                    lights_data.write(&lights).unwrap();
                    queue
                        .0
                        .write_buffer(&scene.lights_buffer, 0, &lights_data.into_inner());
                }
            }
        }
    }
}

/// System to initialize/update the camera buffer for raytracing
pub fn update_raytracer_camera(
    queue: Res<GpuQueue>,
    camera_buffer: Option<Res<RaytracerCameraBuffer>>,
    camera_query: Query<(&Camera, &Transform)>,
    window_size: Res<WindowSize>,
) {
    if let Some(buffer) = camera_buffer {
        if let Some((camera, transform)) = camera_query.iter().find(|(cam, _)| cam.is_main) {
            let aspect_ratio = window_size.width as f32 / window_size.height as f32;

            let camera_data = RaytracerCamera::new(
                Vector3::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                ),
                Vector3::new(camera.target.x, camera.target.y, camera.target.z),
                Vector3::new(0.0, 1.0, 0.0),
                camera.fovy.to_degrees(),
                aspect_ratio,
                camera.aperture,
                camera.focus_distance,
            );

            let mut buffer_data = UniformBuffer::new(Vec::new());
            buffer_data.write(&camera_data).unwrap();
            queue
                .0
                .write_buffer(&buffer.0, 0, &buffer_data.into_inner());
        } else {
            log::warn!("No main camera found for raytracer");
        }
    } else {
        log::warn!("No raytracer camera buffer resource found");
    }
}

/// System to load environment map textures (for new entities)
pub fn load_environment_map(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    query: Query<(Entity, &EnvironmentMap), Without<GpuEnvironmentMap>>,
) {
    for (entity, env_map) in query.iter() {
        // Skip if bytes are empty
        if env_map.bytes.is_empty() {
            log::warn!("Environment map has no data - skipping");
            continue;
        }

        // Try to load as HDR first, fall back to regular image
        let (width, height, data) = if let Ok(decoder) =
            image::codecs::hdr::HdrDecoder::new(std::io::Cursor::new(&env_map.bytes))
        {
            let metadata = decoder.metadata();
            let width = metadata.width;
            let height = metadata.height;

            // Read raw HDR data as bytes (RGB f32)
            let total_bytes = decoder.total_bytes() as usize;
            let mut raw_data = vec![0u8; total_bytes];
            decoder.read_image(&mut raw_data).unwrap();

            // Convert RGB f32 bytes to RGBA f32 bytes
            let pixel_count = (width * height) as usize;
            let mut rgba_data = Vec::with_capacity(pixel_count * 4 * 4);

            for i in 0..pixel_count {
                // Copy RGB (3 * 4 bytes)
                let offset = i * 12; // 3 channels * 4 bytes
                rgba_data.extend_from_slice(&raw_data[offset..offset + 12]);
                // Add alpha = 1.0
                rgba_data.extend_from_slice(&1.0f32.to_ne_bytes());
            }

            (width, height, rgba_data)
        } else {
            // Fall back to regular image loading
            let image = image::load_from_memory(&env_map.bytes).unwrap();
            let image = image.to_rgba8();
            let (width, height) = image.dimensions();

            // Convert to f32 and normalize to [0, 1]
            let mut rgba_data = Vec::with_capacity(width as usize * height as usize * 4 * 4);
            for pixel in image.pixels() {
                rgba_data.extend_from_slice(&(pixel.0[0] as f32 / 255.0).to_ne_bytes());
                rgba_data.extend_from_slice(&(pixel.0[1] as f32 / 255.0).to_ne_bytes());
                rgba_data.extend_from_slice(&(pixel.0[2] as f32 / 255.0).to_ne_bytes());
                rgba_data.extend_from_slice(&(pixel.0[3] as f32 / 255.0).to_ne_bytes());
            }

            (width, height, rgba_data)
        };

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.0.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Environment Map"),
            view_formats: &[],
        });

        queue.0.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(16 * width), // 4 channels * 4 bytes per f32
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.0.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Calculate hash of bytes
        let mut hasher = DefaultHasher::new();
        env_map.bytes.hash(&mut hasher);
        let bytes_hash = hasher.finish();

        commands.entity(entity).insert(GpuEnvironmentMap {
            texture,
            view,
            sampler,
            bytes_hash,
        });

        log::debug!("Loaded environment map: {}x{}", width, height);
    }
}

/// System to reload environment map when it changes
pub fn reload_environment_map(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    query: Query<(Entity, &EnvironmentMap, &GpuEnvironmentMap), Changed<EnvironmentMap>>,
) {
    for (entity, env_map, gpu_env_map) in query.iter() {
        // Calculate hash of current bytes
        let mut hasher = DefaultHasher::new();
        env_map.bytes.hash(&mut hasher);
        let new_hash = hasher.finish();

        // Only reload if hash is different
        if new_hash == gpu_env_map.bytes_hash {
            log::trace!("Environment map unchanged, skipping reload");
            continue;
        }

        log::debug!("reload_environment_map triggered - reloading environment map");
        // Skip if bytes are empty
        if env_map.bytes.is_empty() {
            log::warn!("Environment map has no data - skipping reload");
            continue;
        }

        // Try to load as HDR first, fall back to regular image
        let (width, height, data) = if let Ok(decoder) =
            image::codecs::hdr::HdrDecoder::new(std::io::Cursor::new(&env_map.bytes))
        {
            let metadata = decoder.metadata();
            let width = metadata.width;
            let height = metadata.height;

            // Read raw HDR data as bytes (RGB f32)
            let total_bytes = decoder.total_bytes() as usize;
            let mut raw_data = vec![0u8; total_bytes];
            decoder.read_image(&mut raw_data).unwrap();

            // Convert RGB f32 bytes to RGBA f32 bytes
            let pixel_count = (width * height) as usize;
            let mut rgba_data = Vec::with_capacity(pixel_count * 4 * 4);

            for i in 0..pixel_count {
                // Copy RGB (3 * 4 bytes)
                let offset = i * 12; // 3 channels * 4 bytes
                rgba_data.extend_from_slice(&raw_data[offset..offset + 12]);
                // Add alpha = 1.0
                rgba_data.extend_from_slice(&1.0f32.to_ne_bytes());
            }

            (width, height, rgba_data)
        } else {
            // Fall back to regular image loading
            let image = image::load_from_memory(&env_map.bytes).unwrap();
            let image = image.to_rgba8();
            let (width, height) = image.dimensions();

            // Convert to f32 and normalize to [0, 1]
            let mut rgba_data = Vec::with_capacity(width as usize * height as usize * 4 * 4);
            for pixel in image.pixels() {
                rgba_data.extend_from_slice(&(pixel.0[0] as f32 / 255.0).to_ne_bytes());
                rgba_data.extend_from_slice(&(pixel.0[1] as f32 / 255.0).to_ne_bytes());
                rgba_data.extend_from_slice(&(pixel.0[2] as f32 / 255.0).to_ne_bytes());
                rgba_data.extend_from_slice(&(pixel.0[3] as f32 / 255.0).to_ne_bytes());
            }

            (width, height, rgba_data)
        };

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.0.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Environment Map"),
            view_formats: &[],
        });

        queue.0.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(16 * width), // 4 channels * 4 bytes per f32
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.0.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Remove old component and insert new one with updated hash
        commands.entity(entity).remove::<GpuEnvironmentMap>();
        commands.entity(entity).insert(GpuEnvironmentMap {
            texture,
            view,
            sampler,
            bytes_hash: new_hash,
        });

        log::debug!("Reloaded environment map: {}x{}", width, height);
    }
}
