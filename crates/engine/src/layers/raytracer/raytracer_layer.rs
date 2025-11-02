
use crate::prelude::*;

use crate::layers::raytracer::{
    load_environment_map, reload_environment_map, update_raytracer_camera, update_raytracer_scene,
};
use crate::shader::{RaytracerShader, create_shader_loader, create_static_shader_loader};
use bevy_ecs::schedule::Schedule;
use encase::UniformBuffer;
use wgpu::util::DeviceExt;

pub struct RaytracerLayer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    display_pipeline: wgpu::RenderPipeline,
    output_texture: Option<wgpu::Texture>,
    output_view: Option<wgpu::TextureView>,
    // Ping-pong accumulation buffers for temporal accumulation
    accumulation_texture_a: Option<wgpu::Texture>,
    accumulation_view_a: Option<wgpu::TextureView>,
    accumulation_texture_b: Option<wgpu::Texture>,
    accumulation_view_b: Option<wgpu::TextureView>,
    accumulation_sampler: wgpu::Sampler,
    current_accumulation_index: bool, // false = A, true = B
    sampler: wgpu::Sampler,
    compute_bind_group: Option<wgpu::BindGroup>,
    display_bind_group: Option<wgpu::BindGroup>,
    camera_buffer: wgpu::Buffer,
    schedule: Schedule,
    compute_bind_group_layout: wgpu::BindGroupLayout,
    display_bind_group_layout: wgpu::BindGroupLayout,
    shader_error: Option<String>,
    default_env_map: Option<(wgpu::TextureView, wgpu::Sampler)>,
    frame_count_buffer: wgpu::Buffer,
    frame_count: u32,
    last_camera_position: Option<Vector3<f32>>,
    last_camera_target: Option<Vector3<f32>>,
}

#[derive(Resource, Clone, Default)]
pub struct ShaderError(pub std::collections::HashMap<Shader, String>);

impl RaytracerLayer {
    pub fn new(context: &LayerContext) -> Self {
        // Retrieve device and queue from world resources
        let (device, queue) = {
            let world = context.world.lock().unwrap();
            let device = world.get_resource::<GpuDevice>().unwrap();
            let queue = world.get_resource::<GpuQueue>().unwrap();
            (device.0.clone(), queue.0.clone())
        };

        // Load shader - use hot-reload in debug, static in release
        #[cfg(debug_assertions)]
        let shader_loader = create_shader_loader(
            "crates/engine/src/layers/raytracer/raytracer.wgsl",
            "Raytracer",
        )
        .expect("Failed to create shader loader");

        #[cfg(not(debug_assertions))]
        let shader_loader =
            create_static_shader_loader(include_str!("raytracer.wgsl"), "Raytracer");

        let shader = shader_loader.get_shader(&device);

        // Create bind group layout for compute shader
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Raytracer Compute Bind Group Layout"),
                entries: &[
                    // Camera uniform (binding 0)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Spheres storage buffer (binding 1)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Lights storage buffer (binding 2)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Output texture (binding 3)
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // Environment map texture (binding 4)
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    // Environment map sampler (binding 5)
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Frame count uniform (binding 6)
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Accumulation texture (binding 7) - previous frame for temporal accumulation
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    // Accumulation sampler (binding 8) - not used but kept for compatibility
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    // Accumulation output (binding 9) - for writing next frame's accumulation
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        // Create bind group layout for display shader
        let display_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Raytracer Display Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create compute pipeline
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Raytracer Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raytracer Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("raytracer"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        // Create display pipeline
        let display_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Raytracer Display Pipeline Layout"),
                bind_group_layouts: &[&display_bind_group_layout],
                push_constant_ranges: &[],
            });

        let surface_format = wgpu::TextureFormat::Rgba8Unorm;
        let display_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raytracer Display Pipeline"),
            layout: Some(&display_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create sampler for display
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create camera buffer (will be updated by systems)
        let camera_data = RaytracerCamera::new(
            Vector3::new(0.0, 2.0, 5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
            16.0 / 9.0,
            1.0,
            10.0,
        );

        let mut buffer_data = UniformBuffer::new(Vec::new());
        buffer_data.write(&camera_data).unwrap();
        let bytes = buffer_data.into_inner();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Raytracer Camera Buffer"),
            contents: &bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let frame_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Count Buffer"),
            contents: &0u32.to_ne_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Clone layouts before storing them in the world
        let compute_bind_group_layout_clone = compute_bind_group_layout.clone();
        let display_bind_group_layout_clone = display_bind_group_layout.clone();

        // Store resources in ECS world
        {
            let mut world = context.world.lock().unwrap();
            world.insert_resource(RaytracerBindGroupLayout(compute_bind_group_layout));
            world.insert_resource(RaytracerDisplayBindGroupLayout(display_bind_group_layout));
            world.insert_resource(RaytracerComputePipeline(compute_pipeline.clone()));
            world.insert_resource(RaytracerDisplayPipeline(display_pipeline.clone()));
            world.insert_resource(RaytracerCameraBuffer(camera_buffer.clone()));

            // Store RaytracerShader resource
            let raytracer_shader = RaytracerShader::new(
                shader_loader,
                compute_pipeline.clone(),
                display_pipeline.clone(),
            );
            world.insert_resource(raytracer_shader);

            // Initialize shader error resource
            world.insert_resource(ShaderError::default());
        }

        // Setup systems
        let mut schedule = Schedule::default();
        schedule.add_systems((
            update_raytracer_scene,
            update_raytracer_camera,
            load_environment_map,
            reload_environment_map,
        ));

        // Create accumulation sampler (non-filtering for compute shader compatibility)
        let accumulation_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Accumulation Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            device,
            queue,
            compute_pipeline,
            display_pipeline,
            output_texture: None,
            output_view: None,
            accumulation_texture_a: None,
            accumulation_view_a: None,
            accumulation_texture_b: None,
            accumulation_view_b: None,
            accumulation_sampler,
            current_accumulation_index: false,
            sampler,
            compute_bind_group: None,
            display_bind_group: None,
            camera_buffer,
            schedule,
            compute_bind_group_layout: compute_bind_group_layout_clone,
            display_bind_group_layout: display_bind_group_layout_clone,
            shader_error: None,
            default_env_map: None,
            frame_count_buffer,
            frame_count: 0,
            last_camera_position: None,
            last_camera_target: None,
        }
    }

    fn reload_shader(
        &mut self,
        world: &mut World,
        shader: wgpu::ShaderModule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Reloading shader...");

        // Recreate compute pipeline
        let compute_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Raytracer Compute Pipeline Layout"),
                    bind_group_layouts: &[&self.compute_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let compute_pipeline =
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Raytracer Compute Pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &shader,
                    entry_point: Some("raytracer"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        // Recreate display pipeline
        let display_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Raytracer Display Pipeline Layout"),
                    bind_group_layouts: &[&self.display_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let surface_format = wgpu::TextureFormat::Rgba8Unorm;
        let display_pipeline =
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Raytracer Display Pipeline"),
                    layout: Some(&display_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: surface_format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        self.compute_pipeline = compute_pipeline.clone();
        self.display_pipeline = display_pipeline.clone();

        // Update pipelines in world resources
        world.insert_resource(RaytracerComputePipeline(compute_pipeline.clone()));
        world.insert_resource(RaytracerDisplayPipeline(display_pipeline.clone()));

        // Update RaytracerShader resource
        if let Some(mut raytracer_shader) = world.get_resource_mut::<RaytracerShader>() {
            raytracer_shader.compute_pipeline = compute_pipeline;
            raytracer_shader.display_pipeline = display_pipeline;
        }

        log::info!("Shader reloaded successfully!");
        Ok(())
    }
}

impl Layer for RaytracerLayer {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        // Get all data we need from the world, then drop the lock
        let (width, height, spheres_buffer, lights_buffer, env_view, env_sampler) = {
            let mut world = context.world.lock().unwrap();

            // Check for shader hot reload using RaytracerShader resource
            let reload_result = {
                if let Some(mut raytracer_shader) = world.get_resource_mut::<RaytracerShader>() {
                    raytracer_shader.check_reload(&self.device)
                } else {
                    None
                }
            };

            if let Some(reload_result) = reload_result {
                match reload_result {
                    Ok((shader, _source)) => {
                        // Successfully reloaded - update pipelines
                        if let Err(e) = self.reload_shader(&mut world, shader) {
                            log::error!("Failed to recreate pipelines after shader reload: {}", e);
                        } else {
                            // Clear any previous error
                            self.shader_error = None;
                            let mut errors = world.get_resource_mut::<ShaderError>().unwrap();
                            errors.0.remove(&Shader::Raytracer);
                        }
                    }
                    Err(error_msg) => {
                        // Shader reload failed - store error
                        self.shader_error = Some(error_msg.clone());
                        let mut errors = world.get_resource_mut::<ShaderError>().unwrap();
                        errors.0.insert(Shader::Raytracer, error_msg);
                    }
                }
            }

            // Run schedule
            self.schedule.run(&mut world);

            // Get window size for render target
            let window_size = world.get_resource::<WindowSize>().unwrap();
            let width = window_size.width;
            let height = window_size.height;

            // Check if scene exists and get the spheres/lights buffers
            let scene_buffers = {
                let mut scene_query = world.query::<&GpuRaytracerScene>();
                scene_query
                    .iter(&world)
                    .next()
                    .map(|scene| (scene.spheres_buffer.clone(), scene.lights_buffer.clone()))
            };

            let Some((spheres_buffer, lights_buffer)) = scene_buffers else {
                log::warn!("No GpuRaytracerScene found - scene not yet initialized");
                return Ok(());
            };

            // Query for environment map, or use cached default
            let (env_view, env_sampler) = {
                let mut env_query = world.query::<&GpuEnvironmentMap>();
                if let Some(env) = env_query.iter(&world).next() {
                    log::trace!("Using loaded environment map");
                    (env.view.clone(), env.sampler.clone())
                } else {
                    // Create default if not cached
                    if self.default_env_map.is_none() {
                        let default_texture =
                            self.device.create_texture(&wgpu::TextureDescriptor {
                                label: Some("Default Environment Map"),
                                size: wgpu::Extent3d {
                                    width: 1,
                                    height: 1,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba32Float,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::COPY_DST,
                                view_formats: &[],
                            });

                        self.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &default_texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            &[0.1f32, 0.1, 0.1, 1.0]
                                .iter()
                                .flat_map(|f| f.to_ne_bytes())
                                .collect::<Vec<u8>>(),
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(16),
                                rows_per_image: Some(1),
                            },
                            wgpu::Extent3d {
                                width: 1,
                                height: 1,
                                depth_or_array_layers: 1,
                            },
                        );

                        let default_view =
                            default_texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let default_sampler =
                            self.device.create_sampler(&wgpu::SamplerDescriptor {
                                address_mode_u: wgpu::AddressMode::Repeat,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                address_mode_w: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Nearest,
                                ..Default::default()
                            });

                        self.default_env_map = Some((default_view, default_sampler));
                    }

                    let (view, sampler) = self.default_env_map.as_ref().unwrap();
                    (view.clone(), sampler.clone())
                }
            };

            (
                width,
                height,
                spheres_buffer,
                lights_buffer,
                env_view,
                env_sampler,
            )
        }; // World lock is dropped here

        // Check if we need to recreate texture/bind groups
        let needs_recreation = if let Some(texture) = &self.output_texture {
            let size = texture.size();
            size.width != width || size.height != height || self.compute_bind_group.is_none()
        } else {
            true
        };

        if needs_recreation {
            log::info!(
                "Creating/recreating raytracer output texture and bind groups ({}x{})",
                width,
                height
            );

            // Create output texture - use Rgba8Unorm for storage writes
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Raytracer Output Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Create ping-pong accumulation textures - use Rgba32Float for high precision accumulation
            let accumulation_texture_a = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Raytracer Accumulation Texture A"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING // Storage for writing from compute
                    | wgpu::TextureUsages::TEXTURE_BINDING // For reading in next frame
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let accumulation_view_a =
                accumulation_texture_a.create_view(&wgpu::TextureViewDescriptor::default());

            let accumulation_texture_b = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Raytracer Accumulation Texture B"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let accumulation_view_b =
                accumulation_texture_b.create_view(&wgpu::TextureViewDescriptor::default());

            // Find the main camera entity and update its render target
            {
                let mut world = context.world.lock().unwrap();
                let camera_entity = {
                    let mut camera_query = world.query::<(Entity, &Camera)>();
                    camera_query
                        .iter(&world)
                        .find(|(_, camera)| camera.is_main)
                        .map(|(entity, _)| entity)
                };

                if let Some(entity) = camera_entity {
                    if let Some(mut render_target) = world.get_mut::<GpuRenderTarget>(entity) {
                        render_target.texture = texture.clone();
                        log::info!("Updated GpuRenderTarget on camera");
                    } else {
                        world.entity_mut(entity).insert(GpuRenderTarget {
                            texture: texture.clone(),
                        });
                        log::info!("Added GpuRenderTarget to camera");
                    }
                }
            }

            // Create display bind group
            let display_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Raytracer Display Bind Group"),
                layout: &self.display_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            self.output_texture = Some(texture);
            self.output_view = Some(view);
            self.accumulation_texture_a = Some(accumulation_texture_a);
            self.accumulation_view_a = Some(accumulation_view_a);
            self.accumulation_texture_b = Some(accumulation_texture_b);
            self.accumulation_view_b = Some(accumulation_view_b);
            self.display_bind_group = Some(display_bind_group);
        }

        // Recreate compute bind group every frame for ping-pong accumulation buffers
        // This must happen outside needs_recreation to swap read/write buffers each frame
        if let (Some(view), Some(view_a), Some(view_b)) = (
            &self.output_view,
            &self.accumulation_view_a,
            &self.accumulation_view_b,
        ) {
            // Read from current, write to next
            let (read_accum_view, write_accum_view) = if self.current_accumulation_index {
                (view_b, view_a)
            } else {
                (view_a, view_b)
            };

            let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Raytracer Compute Bind Group"),
                layout: &self.compute_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: spheres_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&env_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&env_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.frame_count_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(read_accum_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(&self.accumulation_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(write_accum_view),
                    },
                ],
            });

            self.compute_bind_group = Some(compute_bind_group);
        }

        // Now run the compute shader to raytrace directly into the output texture
        if let Some(_output_view) = &self.output_view {
            // Check if camera has moved - if so, reset frame count
            let camera_moved = {
                let mut world = context.world.lock().unwrap();
                let mut camera_query = world.query::<(&Camera, &Transform)>();
                if let Some((camera, transform)) =
                    camera_query.iter(&world).find(|(cam, _)| cam.is_main)
                {
                    let current_pos = Vector3::new(
                        transform.position.x,
                        transform.position.y,
                        transform.position.z,
                    );
                    let current_target =
                        Vector3::new(camera.target.x, camera.target.y, camera.target.z);

                    let moved = self.last_camera_position.map_or(true, |last_pos| {
                        (current_pos - last_pos).magnitude() > 0.001
                    }) || self.last_camera_target.map_or(true, |last_target| {
                        (current_target - last_target).magnitude() > 0.001
                    });

                    self.last_camera_position = Some(current_pos);
                    self.last_camera_target = Some(current_target);
                    moved
                } else {
                    false
                }
            };

            if camera_moved {
                self.frame_count = 0;
                self.current_accumulation_index = false; // Reset to A
            }

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Raytracer Encoder"),
                });

            // If there's a shader error, clear to black instead of running compute shader
            if self.shader_error.is_none() {
                // Update frame count
                self.frame_count = self.frame_count.wrapping_add(1);
                self.queue.write_buffer(
                    &self.frame_count_buffer,
                    0,
                    &self.frame_count.to_ne_bytes(),
                );

                // Run compute shader to raytrace the scene
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Raytracer Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(&self.compute_pipeline);
                if let Some(bind_group) = &self.compute_bind_group {
                    compute_pass.set_bind_group(0, bind_group, &[]);

                    // Dispatch compute shader (8x8 workgroups)
                    let workgroup_count_x = (width + 7) / 8;
                    let workgroup_count_y = (height + 7) / 8;
                    compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
                }

                drop(compute_pass);

                // Flip accumulation buffer for next frame
                self.current_accumulation_index = !self.current_accumulation_index;

                // No need to copy - shader writes directly to accumulation_output (binding 9)
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}
}
