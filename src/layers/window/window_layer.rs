use crate::prelude::*;

pub struct WindowLayer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
}

impl WindowLayer {
    pub fn new(context: &LayerContext) -> Self {
        let size = context.window.inner_size();

        // Retrieve everything from world resources (set by DeviceLayer)
        // Take ownership of surface and adapter since they can't be cloned
        let (device, queue, adapter, surface) = {
            let mut world = context.world.lock().unwrap();
            let device = world.get_resource::<GpuDevice>().unwrap().0.clone();
            let queue = world.get_resource::<GpuQueue>().unwrap().0.clone();

            let mut adapter_res = world.get_resource_mut::<GpuAdapter>().unwrap();
            let adapter = adapter_res.0.take().expect("Adapter already taken");

            let mut surface_res = world.get_resource_mut::<GpuSurface>().unwrap();
            let surface = surface_res.0.take().expect("Surface already taken");

            (device, queue, adapter, surface)
        };

        let surface_caps = surface.get_capabilities(&adapter);

        // Try to use Rgba8Unorm to match raytracer output, fallback to sRGB
        let surface_format = if surface_caps
            .formats
            .contains(&wgpu::TextureFormat::Rgba8Unorm)
        {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0])
        };

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

        Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
        }
    }

    fn resize(&mut self, context: &LayerContext, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            // Update window size in world resources
            let mut world = context.world.lock().unwrap();
            world.insert_resource(WindowSize { width, height });
        }
    }
}

impl Layer for WindowLayer {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let mut world = context.world.lock().unwrap();

        // Find the main camera and its render target
        let target = world
            .query::<(&Camera, &GpuRenderTarget)>()
            .iter(&world)
            .find(|(camera, _)| camera.is_main)
            .map(|(_, target)| target);

        let Some(target) = target else {
            // No main camera found, skip rendering
            return Ok(());
        };

        // Get the window surface texture
        let surface_texture = self.surface.get_current_texture()?;
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create an encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Window Blit Encoder"),
            });

        // Check if formats are compatible for direct copy
        let source_format = target.texture.format();
        let dest_format = self.config.format;

        if source_format == dest_format {
            // Direct copy if formats match
            encoder.copy_texture_to_texture(
                target.texture.as_image_copy(),
                surface_texture.texture.as_image_copy(),
                wgpu::Extent3d {
                    width: self.config.width.min(target.texture.width()),
                    height: self.config.height.min(target.texture.height()),
                    depth_or_array_layers: 1,
                },
            );
        } else {
            // Use a render pass to convert formats (e.g., Rgba8Unorm -> Bgra8UnormSrgb)
            // This will handle format conversion automatically through the GPU
            let _target_view = target
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // Create a simple fullscreen blit using a render pass
            // We need a simple shader for this - for now, just clear to a debug color
            // In a production setup, you'd have a proper blit shader
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Window Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            drop(render_pass);

            log::warn!(
                "Format mismatch: source={:?}, dest={:?}. Need blit shader for proper conversion.",
                source_format,
                dest_format
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}

    fn event(&mut self, context: &LayerContext, event: crate::LayerEvent) {
        let crate::LayerEvent::WindowEvent(window_event) = event;
        match *window_event {
            winit::event::WindowEvent::Resized(physical_size) => {
                self.resize(context, physical_size.width, physical_size.height);
            }
            _ => {}
        }
    }
}
