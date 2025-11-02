use crate::layers::renderer::systems::{
    initialize_camera_buffers, initialize_mesh_buffers, initialize_render_targets,
    initialize_texture_buffers, initialize_transform_buffers, update_camera_buffers,
    update_render_targets, update_transform_buffers,
};
use crate::prelude::*;

pub struct RenderLayer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    schedule: Schedule,
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
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &transform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            render_pipeline,
            schedule,
        }
    }
}

impl Layer for RenderLayer {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut world = context.world.lock().unwrap();

        // Run the schedule first before any queries
        self.schedule.run(&mut world);

        // Store cameras as a separate QueryState to avoid nested mutable borrows
        let mut camera_query = world.query::<(&GpuCamera, &GpuRenderTarget)>();
        let mut mesh_query = world.query::<(&GpuMesh, &GpuTexture, &GpuTransform)>();

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

                for (mesh, texture, transform) in mesh_query.iter(&world) {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(0, Some(&texture.bind_group), &[]);
                    render_pass.set_bind_group(1, &camera.bind_group, &[]);
                    render_pass.set_bind_group(2, &transform.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), index_format());
                    render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            };

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}
}
