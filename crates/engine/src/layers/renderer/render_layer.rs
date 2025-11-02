use crate::layers::renderer::systems::{
    initialize_camera_buffers, initialize_mesh_buffers, initialize_render_targets,
    initialize_texture_buffers, initialize_transform_buffers, update_camera_buffers,
    update_render_targets, update_transform_buffers,
};
use crate::prelude::*;
use crate::shader::{BindGroupRequirement, ShaderCache, ShaderInstance};

pub struct RenderLayer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    schedule: Schedule,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    transform_bind_group_layout: wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
}

impl RenderLayer {
    pub fn new(context: &LayerContext) -> Self {
        // Retrieve device and queue from world resources (set by WindowLayer)
        let (device, queue) = {
            let world = context.world.lock().unwrap();
            let device = world.get_resource::<GpuDevice>().unwrap();
            let queue = world.get_resource::<GpuQueue>().unwrap();
            (device.0.clone(), queue.0.clone())
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("transform_bind_group_layout"),
            });

        // ecs resources
        {
            let mut world = context.world.lock().unwrap();
            world.insert_resource(GpuDevice(device.clone()));
            world.insert_resource(GpuQueue(queue.clone()));
            world.insert_resource(TextureBindGroupLayout(texture_bind_group_layout.clone()));
            world.insert_resource(CameraBindGroupLayout(camera_bind_group_layout.clone()));
            world.insert_resource(TransformBindGroupLayout(
                transform_bind_group_layout.clone(),
            ));
        }

        // shaders - use sRGB format for render targets
        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        // Initialize empty ShaderCache - shaders will be registered by game code
        {
            let mut world = context.world.lock().unwrap();
            world.insert_resource(ShaderCache::new());
        }

        // ecs
        let mut schedule = Schedule::default();
        schedule.add_systems((
            initialize_mesh_buffers,
            initialize_texture_buffers,
            initialize_camera_buffers,
            initialize_render_targets,
            update_render_targets,
            update_camera_buffers,
            initialize_transform_buffers,
            update_transform_buffers,
        ));

        Self {
            device,
            queue,
            schedule,
            texture_bind_group_layout,
            camera_bind_group_layout,
            transform_bind_group_layout,
            surface_format,
        }
    }

    fn reload_shader(
        &mut self,
        shader_type: &Shader,
        shader: wgpu::ShaderModule,
        shader_source: &str,
    ) -> Result<ShaderInstance, Box<dyn std::error::Error>> {
        log::info!("Reloading {} shader...", shader_type);

        // Parse bind group requirements from reloaded shader FIRST
        let bind_group_requirements = BindGroupRequirement::parse_from_shader(shader_source);
        log::info!(
            "Reloading {} shader with bind groups: {:?}",
            shader_type,
            bind_group_requirements
        );

        // Build bind group layouts dynamically based on shader requirements
        let mut layouts = Vec::new();
        for requirement in &bind_group_requirements {
            if let Some(req) = requirement {
                let layout = match req {
                    BindGroupRequirement::Texture => &self.texture_bind_group_layout,
                    BindGroupRequirement::Camera => &self.camera_bind_group_layout,
                    BindGroupRequirement::Transform => &self.transform_bind_group_layout,
                    BindGroupRequirement::Unknown(name) => {
                        return Err(
                            format!("Unknown bind group requirement '{}' in shader", name).into(),
                        );
                    }
                };
                layouts.push(layout);
            }
        }

        // Recreate render pipeline with dynamic layouts
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    buffers: &[Vertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
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

        Ok(ShaderInstance {
            module: shader,
            pipeline: render_pipeline,
            bind_group_requirements,
        })
    }
}

impl Layer for RenderLayer {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut world = context.world.lock().unwrap();

        // Check for shader hot reload in ShaderCache
        let reloaded_shaders = {
            if let Some(mut shader_cache) = world.get_resource_mut::<ShaderCache>() {
                shader_cache.check_hot_reload(&self.device)
            } else {
                Vec::new()
            }
        };

        // Process reloaded shaders
        for (shader, reload_result) in reloaded_shaders {
            use crate::layers::raytracer::ShaderError;
            match reload_result {
                Ok((shader_module, shader_source)) => {
                    // Successfully reloaded - recreate pipeline
                    match self.reload_shader(&shader, shader_module, &shader_source) {
                        Ok(shader_instance) => {
                            // Update shader cache with new instance
                            if let Some(mut shader_cache) = world.get_resource_mut::<ShaderCache>()
                            {
                                shader_cache.update_shader(&shader, shader_instance);
                            }
                            // Clear any previous error
                            if let Some(mut errors) = world.get_resource_mut::<ShaderError>() {
                                errors.0.remove(&shader);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to recreate pipeline for {}: {}", shader, e);
                        }
                    }
                }
                Err(error_msg) => {
                    // Shader reload failed - store error
                    if let Some(mut errors) = world.get_resource_mut::<ShaderError>() {
                        errors.0.insert(shader, error_msg);
                    }
                }
            }
        }

        // Run the schedule first before any queries
        self.schedule.run(&mut world);

        // Store cameras as a separate QueryState to avoid nested mutable borrows
        let mut camera_query = world.query::<(&GpuCamera, &GpuRenderTarget)>();
        let mut mesh_query = world.query::<(&Material, &GpuMesh, &GpuTexture, &GpuTransform)>();

        // Get shader cache for looking up pipelines
        let shader_cache = world.get_resource::<ShaderCache>();

        // Process each camera
        for (camera, target) in camera_query.iter(&world) {
            let view = target
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                for (material, mesh, texture, transform) in mesh_query.iter(&world) {
                    // Look up shader pipeline from cache
                    let shader_instance = shader_cache
                        .as_ref()
                        .and_then(|cache| cache.get_shader(&material.shader));

                    if let Some(shader_instance) = shader_instance {
                        render_pass.set_pipeline(&shader_instance.pipeline);

                        // Set bind groups based on shader requirements
                        for (index, requirement) in
                            shader_instance.bind_group_requirements.iter().enumerate()
                        {
                            if let Some(req) = requirement {
                                match req {
                                    BindGroupRequirement::Texture => {
                                        render_pass.set_bind_group(
                                            index as u32,
                                            Some(&texture.bind_group),
                                            &[],
                                        );
                                    }
                                    BindGroupRequirement::Camera => {
                                        render_pass.set_bind_group(
                                            index as u32,
                                            &camera.bind_group,
                                            &[],
                                        );
                                    }
                                    BindGroupRequirement::Transform => {
                                        render_pass.set_bind_group(
                                            index as u32,
                                            &transform.bind_group,
                                            &[],
                                        );
                                    }
                                    BindGroupRequirement::Unknown(name) => {
                                        log::warn!(
                                            "Unknown bind group requirement '{}' at index {}",
                                            name,
                                            index
                                        );
                                    }
                                }
                            }
                        }

                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(mesh.index_buffer.slice(..), index_format());
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    } else {
                        log::warn!("Shader '{}' not found in cache", material.shader);
                    }
                }
            };

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}
}
