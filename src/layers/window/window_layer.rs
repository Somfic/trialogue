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

        // Copy the camera's render target to the surface
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Window Blit Encoder"),
            });

        encoder.copy_texture_to_texture(
            target.texture.as_image_copy(),
            surface_texture.texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.config.width.min(target.texture.width()),
                height: self.config.height.min(target.texture.height()),
                depth_or_array_layers: 1,
            },
        );

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
