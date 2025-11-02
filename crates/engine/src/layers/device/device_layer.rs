
use crate::prelude::*;

/// Layer that initializes the GPU device, queue, and surface.
/// This must run before RenderLayer but doesn't need to do anything during frame rendering.
pub struct DeviceLayer;

impl DeviceLayer {
    pub fn new(context: &LayerContext) -> Self {
        let size = context.window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(context.window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        // Check which polygon mode features are supported
        let adapter_features = adapter.features();
        let mut features = wgpu::Features::empty();
        let mut supported_features = SupportedFeatures::default();

        if adapter_features.contains(wgpu::Features::POLYGON_MODE_LINE) {
            features |= wgpu::Features::POLYGON_MODE_LINE;
            supported_features.polygon_mode_line = true;
        }

        if adapter_features.contains(wgpu::Features::POLYGON_MODE_POINT) {
            features |= wgpu::Features::POLYGON_MODE_POINT;
            supported_features.polygon_mode_point = true;
        }

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: features,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        // Store everything in world resources
        let mut world = context.world.lock().unwrap();
        world.insert_resource(GpuDevice(device));
        world.insert_resource(GpuQueue(queue));
        world.insert_resource(GpuAdapter(Some(adapter)));
        world.insert_resource(GpuSurface(Some(surface)));
        world.insert_resource(supported_features);
        world.insert_resource(WindowSize {
            width: size.width,
            height: size.height,
        });

        Self
    }
}

impl Layer for DeviceLayer {
    fn frame(&mut self, _context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError> {
        Ok(())
    }

    fn detach(&mut self, _context: &LayerContext) {}
}
