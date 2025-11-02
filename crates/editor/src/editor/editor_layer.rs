use crate::prelude::*;
use trialogue_engine::prelude::*;

use super::{editor_state::EditorState, editor_ui};

pub struct EditorLayer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    is_surface_configured: bool,
    viewport_texture: Option<wgpu::Texture>,
    viewport_texture_id: Option<egui::TextureId>,

    // egui state
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: Option<egui_wgpu::Renderer>,

    // editor state
    editor_state: EditorState,
}

impl EditorLayer {
    pub fn new(context: &LayerContext) -> Self {
        let size = context.window.inner_size();

        // Retrieve everything from world resources (set by DeviceLayer)
        let (device, queue, adapter, surface) = {
            let mut world = context.world.lock().unwrap();
            let device = world.get_resource::<GpuDevice>().unwrap().0.clone();
            let queue = world.get_resource::<GpuQueue>().unwrap().0.clone();

            let mut adapter_res = world.get_resource_mut::<GpuAdapter>().unwrap();
            let adapter = adapter_res.0.take().expect("Adapter already taken");

            let mut surface_res = world.get_resource_mut::<GpuSurface>().unwrap();
            let surface = surface_res.0.take().expect("Surface already taken");

            // Set initial viewport size (will be updated by egui layout)
            world.insert_resource(WindowSize {
                width: (size.width.saturating_sub(200)).max(1), // Subtract sidebar width
                height: size.height.max(1),
            });

            (device, queue, adapter, surface)
        };

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &context.window,
            None,
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );

        Self {
            surface,
            config,
            device,
            queue,
            is_surface_configured: false,
            viewport_texture: None,
            viewport_texture_id: None,
            egui_ctx,
            egui_state,
            egui_renderer: Some(egui_renderer),
            editor_state: EditorState::new(),
        }
    }

    fn resize(&mut self, _context: &LayerContext, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }
}

impl Layer for EditorLayer {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        if !self.is_surface_configured {
            return Ok(());
        }

        // Get surface texture
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Start egui frame
        let raw_input = self.egui_state.take_egui_input(&context.window);

        // Capture values needed in the closure
        let viewport_texture_id = self.viewport_texture_id;
        let world = context.world.clone();

        let egui_output = self.egui_ctx.run(raw_input, |ctx| {
            editor_ui::draw_ui(
                context,
                ctx,
                &world,
                viewport_texture_id,
                &mut self.editor_state,
            );
        });

        // Handle egui output
        self.egui_state
            .handle_platform_output(&context.window, egui_output.platform_output);

        // Get current viewport size from world
        let viewport_size = {
            let world = context.world.lock().unwrap();
            *world.get_resource::<WindowSize>().unwrap()
        };

        // Create/update intermediate texture for viewport if needed
        let mut texture_changed = false;
        if self.viewport_texture.is_none()
            || self.viewport_texture.as_ref().unwrap().width() != viewport_size.width
            || self.viewport_texture.as_ref().unwrap().height() != viewport_size.height
        {
            if viewport_size.width > 0 && viewport_size.height > 0 {
                // Unregister old texture if it exists
                if let Some(old_id) = self.viewport_texture_id.take() {
                    if let Some(renderer) = &mut self.egui_renderer {
                        renderer.free_texture(&old_id);
                    }
                }

                self.viewport_texture =
                    Some(self.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Viewport Texture"),
                        size: wgpu::Extent3d {
                            width: viewport_size.width,
                            height: viewport_size.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        // Use the same format as the camera render target to avoid color channel swapping
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::COPY_SRC,
                        view_formats: &[],
                    }));
                texture_changed = true;
            }
        }

        // Register the camera render target directly with egui
        {
            let mut world = context.world.lock().unwrap();

            if let Some((_, target)) = world
                .query::<(&Camera, &GpuRenderTarget)>()
                .iter(&world)
                .find(|(camera, _)| camera.is_main)
            {
                let camera_texture = &target.texture;
                let view = camera_texture.create_view(&wgpu::TextureViewDescriptor::default());

                if let Some(renderer) = &mut self.egui_renderer {
                    if texture_changed || self.viewport_texture_id.is_none() {
                        // Unregister old texture if it exists
                        if let Some(old_id) = self.viewport_texture_id.take() {
                            renderer.free_texture(&old_id);
                        }

                        // Register the camera texture directly
                        let texture_id = renderer.register_native_texture(
                            &self.device,
                            &view,
                            wgpu::FilterMode::Nearest,
                        );
                        self.viewport_texture_id = Some(texture_id);
                    } else if let Some(texture_id) = self.viewport_texture_id {
                        // Update the texture reference
                        renderer.update_egui_texture_from_wgpu_texture(
                            &self.device,
                            &view,
                            wgpu::FilterMode::Nearest,
                            texture_id,
                        );
                    }
                }
            }
        }

        // Render everything
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Editor Encoder"),
            });

        // Clear the surface with background color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
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
        }

        // Render egui UI (which will display the viewport texture)
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.egui_ctx.pixels_per_point(),
        };

        let primitives = self
            .egui_ctx
            .tessellate(egui_output.shapes, egui_output.pixels_per_point);

        // Take the renderer out temporarily to avoid borrow checker issues
        let mut renderer = self.egui_renderer.take().unwrap();

        for (id, image_delta) in &egui_output.textures_delta.set {
            renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &primitives[..],
            &screen_descriptor,
        );

        // Render egui
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // egui requires a 'static lifetime for the render pass
            // We use forget_lifetime to convert the lifetime
            let mut render_pass_static = render_pass.forget_lifetime();

            // Render egui primitives
            renderer.render(&mut render_pass_static, &primitives, &screen_descriptor);
        }

        for id in &egui_output.textures_delta.free {
            renderer.free_texture(id);
        }

        // Put the renderer back
        self.egui_renderer = Some(renderer);

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));

        surface_texture.present();

        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}

    fn event(&mut self, context: &LayerContext, event: trialogue_engine::LayerEvent) {
        let trialogue_engine::LayerEvent::WindowEvent(window_event) = event;

        // Let egui handle the event first
        let response = self
            .egui_state
            .on_window_event(&context.window, &window_event);

        // Handle window-level events
        match *window_event {
            winit::event::WindowEvent::Resized(physical_size) => {
                self.resize(context, physical_size.width, physical_size.height);
            }
            _ => {}
        }

        // Request repaint if egui consumed the event
        if response.consumed {
            context.window.request_redraw();
        }
    }
}
