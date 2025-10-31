use crate::layers::raytracer::{update_raytracer_camera, update_raytracer_scene};
use crate::prelude::*;
use bevy_ecs::schedule::Schedule;
use std::time::SystemTime;
use wgpu::util::DeviceExt;

pub struct RaytracerLayer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    display_pipeline: wgpu::RenderPipeline,
    output_texture: Option<wgpu::Texture>,
    output_view: Option<wgpu::TextureView>,
    sampler: wgpu::Sampler,
    compute_bind_group: Option<wgpu::BindGroup>,
    display_bind_group: Option<wgpu::BindGroup>,
    camera_buffer: wgpu::Buffer,
    schedule: Schedule,
    shader_path: std::path::PathBuf,
    last_shader_modified: Option<SystemTime>,
    compute_bind_group_layout: wgpu::BindGroupLayout,
    display_bind_group_layout: wgpu::BindGroupLayout,
    shader_error: Option<String>,
}

#[derive(Resource, Clone)]
pub struct ShaderError(pub Option<String>);

impl RaytracerLayer {
    pub fn new(context: &LayerContext) -> Self {
        // Retrieve device and queue from world resources
        let (device, queue) = {
            let world = context.world.lock().unwrap();
            let device = world.get_resource::<GpuDevice>().unwrap();
            let queue = world.get_resource::<GpuQueue>().unwrap();
            (device.0.clone(), queue.0.clone())
        };

        // Load shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/shader.wgsl"));

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
        let camera_data = RaytracerCamera {
            position: [0.0, 2.0, 5.0],
            _padding1: 0.0,
            look_at: [0.0, 0.0, 0.0],
            _padding2: 0.0,
            up: [0.0, 1.0, 0.0],
            fov: 60.0,
            aspect_ratio: 16.0 / 9.0,
            _padding3: [0.0; 3],
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Raytracer Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_data]),
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
        }

        // Setup systems
        let mut schedule = Schedule::default();
        schedule.add_systems((update_raytracer_scene, update_raytracer_camera));

        let shader_path = std::path::PathBuf::from("src/shaders/shader.wgsl");
        let last_shader_modified = std::fs::metadata(&shader_path)
            .ok()
            .and_then(|m| m.modified().ok());

        // Initialize shader error resource in world
        {
            let mut world = context.world.lock().unwrap();
            world.insert_resource(ShaderError(None));
        }

        Self {
            device,
            queue,
            compute_pipeline,
            display_pipeline,
            output_texture: None,
            output_view: None,
            sampler,
            compute_bind_group: None,
            display_bind_group: None,
            camera_buffer,
            schedule,
            shader_path,
            last_shader_modified,
            compute_bind_group_layout: compute_bind_group_layout_clone,
            display_bind_group_layout: display_bind_group_layout_clone,
            shader_error: None,
        }
    }

    fn reload_shader(&mut self, world: &mut World) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Reloading shader...");

        // Read shader file
        let shader_source = std::fs::read_to_string(&self.shader_path)?;

        // Create new shader module descriptor
        let shader_descriptor = wgpu::ShaderModuleDescriptor {
            label: Some("Raytracer Shader (Hot Reloaded)"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        };

        // Try to create the shader module - catch panics from validation errors
        let shader = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.device.create_shader_module(shader_descriptor)
        })) {
            Ok(shader) => {
                // Success! Clear any previous error
                self.shader_error = None;
                world.insert_resource(ShaderError(None));
                shader
            }
            Err(panic_info) => {
                // Extract error message from panic
                let error_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Shader compilation failed with unknown error".to_string()
                };

                log::error!("Shader compilation failed: {}", error_msg);
                self.shader_error = Some(error_msg.clone());
                world.insert_resource(ShaderError(Some(error_msg)));
                return Err("Shader compilation failed - check console for errors".into());
            }
        };

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

        self.compute_pipeline = compute_pipeline;
        self.display_pipeline = display_pipeline;

        log::info!("Shader reloaded successfully!");
        Ok(())
    }
}

impl Layer for RaytracerLayer {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut world = context.world.lock().unwrap();

        // Check for shader hot reload
        if let Ok(metadata) = std::fs::metadata(&self.shader_path) {
            if let Ok(modified) = metadata.modified() {
                if self
                    .last_shader_modified
                    .map_or(true, |last| modified > last)
                {
                    self.last_shader_modified = Some(modified);
                    if let Err(e) = self.reload_shader(&mut world) {
                        log::error!("Failed to reload shader: {}", e);
                    }
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

            // Create output texture - use Rgba8Unorm for storage, Bgra8UnormSrgb for display
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
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Find the main camera entity and update its render target
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

            // Create compute bind group
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
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            });

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
            self.compute_bind_group = Some(compute_bind_group);
            self.display_bind_group = Some(display_bind_group);
        }

        // Now run the compute shader to raytrace directly into the output texture
        if let Some(output_view) = &self.output_view {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Raytracer Encoder"),
                });

            // If there's a shader error, clear to black instead of running compute shader
            if self.shader_error.is_none() {
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
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}
}
