use crate::prelude::*;
use encase::{StorageBuffer, UniformBuffer};
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

            let camera_data = RaytracerCamera {
                position: Vector3::new(
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                ),
                look_at: Vector3::new(camera.target.x, camera.target.y, camera.target.z),
                up: Vector3::new(0.0, 1.0, 0.0),
                fov: camera.fovy.to_degrees(),
                aspect_ratio,
                aperture: camera.aperture,
                focus_distance: camera.focus_distance,
            };

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
