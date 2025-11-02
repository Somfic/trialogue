pub use bevy_ecs::world::World;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};

use crate::prelude::Shader;
pub type Result<T> = anyhow::Result<T>;

pub mod async_task;
pub mod components;
pub mod gpu_component;
pub mod layers;
pub mod prelude;
pub mod shader;

pub trait Layer: 'static {
    fn frame(&mut self, context: &LayerContext) -> std::result::Result<(), wgpu::SurfaceError>;
    fn detach(&mut self, context: &LayerContext);
    fn event(&mut self, _context: &LayerContext, _event: LayerEvent) {}
}

pub trait LayerFactory: 'static {
    fn create(&self, context: &LayerContext) -> Box<dyn Layer>;
}

pub struct LayerContext {
    pub window: Arc<Window>,
    pub world: Arc<Mutex<World>>,
    pub delta_time: Duration,
}

pub enum LayerEvent {
    WindowEvent(Arc<WindowEvent>),
}

pub struct ApplicationBuilder {
    layer_factories: Vec<Box<dyn LayerFactory>>,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self {
            layer_factories: Vec::new(),
        }
    }

    pub fn add_layer_factory(mut self, factory: impl LayerFactory) -> Self {
        self.layer_factories.push(Box::new(factory));
        self
    }

    pub fn add_layer<F>(mut self, factory_fn: F) -> Self
    where
        F: Fn(&LayerContext) -> Box<dyn Layer> + 'static,
    {
        self.layer_factories
            .push(Box::new(ClosureLayerFactory::new(factory_fn)));
        self
    }

    pub fn build(self) -> Application {
        Application {
            layer_factories: self.layer_factories,
            state: None,
            world: Arc::new(Mutex::new(World::new())),
            shader_registrations: Vec::new(),
        }
    }
}

struct ClosureLayerFactory<F> {
    factory_fn: F,
}

impl<F> ClosureLayerFactory<F> {
    fn new(factory_fn: F) -> Self {
        Self { factory_fn }
    }
}

impl<F> LayerFactory for ClosureLayerFactory<F>
where
    F: Fn(&LayerContext) -> Box<dyn Layer> + 'static,
{
    fn create(&self, context: &LayerContext) -> Box<dyn Layer> {
        (self.factory_fn)(context)
    }
}

pub struct Application {
    layer_factories: Vec<Box<dyn LayerFactory>>,
    state: Option<ApplicationState>,
    world: Arc<Mutex<World>>,
    shader_registrations: Vec<ShaderRegistration>,
}

struct ShaderRegistration {
    path: std::path::PathBuf,
    shader: Shader,
    static_source: &'static str,
}

pub struct ApplicationState {
    window: Arc<Window>,
    layers: Vec<Box<dyn Layer>>,
    last_frame_time: Instant,
}

impl Application {
    fn redraw(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let state = match &mut self.state {
            Some(state) => state,
            None => return Ok(()),
        };

        let now = Instant::now();
        let delta_time = now.duration_since(state.last_frame_time);
        state.last_frame_time = now;

        let context = LayerContext {
            window: state.window.clone(),
            world: self.world.clone(),
            delta_time,
        };

        for layer in &mut state.layers {
            layer.frame(&context)?;
        }

        self.world.lock().unwrap().clear_trackers();

        Ok(())
    }

    pub fn spawn<B: bevy_ecs::bundle::Bundle>(&mut self, label: impl Into<String>, bundle: B) {
        use crate::prelude::*;
        let bundle = (
            Tag {
                label: label.into(),
            },
            bundle,
        );
        self.world.lock().unwrap().spawn(bundle);
    }

    /// Register a shader with the ShaderCache
    ///
    /// In debug builds, this will set up hot-reloading from the specified path.
    /// In release builds, this will use the static source provided via include_str!.
    ///
    /// This method queues the shader for registration - actual registration happens
    /// when the application is resumed (after layers are initialized).
    ///
    /// # Example
    /// ```no_run
    /// app.register_shader(
    ///     "crates/engine/src/layers/renderer/shader.wgsl",
    ///     "standard",
    ///     include_str!("../engine/src/layers/renderer/shader.wgsl"),
    /// );
    /// ```
    pub fn register_shader(
        &mut self,
        path: impl AsRef<std::path::Path>,
        shader: Shader,
        static_source: &'static str,
    ) {
        self.shader_registrations.push(ShaderRegistration {
            path: path.as_ref().to_path_buf(),
            shader: shader,
            static_source,
        });
    }

    /// Internal method to actually perform shader registrations after layers are initialized
    fn perform_shader_registrations(&mut self) -> Result<()> {
        use crate::prelude::*;
        use crate::shader::*;

        let registrations = std::mem::take(&mut self.shader_registrations);

        for registration in registrations {
            let mut world = self.world.lock().unwrap();

            // Get required resources
            let device = world.get_resource::<GpuDevice>()
                .ok_or_else(|| anyhow::anyhow!("GpuDevice resource not found - make sure DeviceLayer is added before registering shaders"))?;
            let texture_layout = world
                .get_resource::<TextureBindGroupLayout>()
                .ok_or_else(|| anyhow::anyhow!("TextureBindGroupLayout resource not found"))?;
            let camera_layout = world
                .get_resource::<CameraBindGroupLayout>()
                .ok_or_else(|| anyhow::anyhow!("CameraBindGroupLayout resource not found"))?;
            let transform_layout = world
                .get_resource::<TransformBindGroupLayout>()
                .ok_or_else(|| anyhow::anyhow!("TransformBindGroupLayout resource not found"))?;
            let supported_features = world
                .get_resource::<SupportedFeatures>()
                .ok_or_else(|| anyhow::anyhow!("SupportedFeatures resource not found"))?;

            // Create shader loader based on build configuration
            #[cfg(debug_assertions)]
            let shader_loader =
                create_shader_loader(&registration.path, registration.shader.to_string())
                    .map_err(|e| anyhow::anyhow!("Failed to create shader loader: {}", e))?;

            #[cfg(not(debug_assertions))]
            let shader_loader =
                create_static_shader_loader(registration.static_source, &registration.name);

            let shader = shader_loader.get_shader(&device.0);
            let shader_source = shader_loader.get_source();

            // Parse bind group requirements
            let bind_group_requirements = BindGroupRequirement::parse_from_shader(&shader_source);
            log::info!(
                "Registered shader '{}' with bind groups: {:?}",
                registration.shader,
                bind_group_requirements
            );

            // Build bind group layouts dynamically
            let mut layouts = Vec::new();
            for requirement in &bind_group_requirements {
                if let Some(req) = requirement {
                    let layout = match req {
                        BindGroupRequirement::Texture => &texture_layout.0,
                        BindGroupRequirement::Camera => &camera_layout.0,
                        BindGroupRequirement::Transform => &transform_layout.0,
                        BindGroupRequirement::Unknown(name) => {
                            return Err(anyhow::anyhow!(
                                "Unknown bind group requirement '{}' in shader",
                                name
                            ));
                        }
                    };
                    layouts.push(layout);
                }
            }

            // Create render pipelines for all render modes
            let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;
            let render_pipeline_layout =
                device
                    .0
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(&format!("{} Pipeline Layout", registration.shader)),
                        bind_group_layouts: &layouts,
                        push_constant_ranges: &[],
                    });

            // Build list of render modes based on supported features
            let mut render_modes = vec![RenderMode::filled()];

            if supported_features.polygon_mode_line {
                render_modes.push(RenderMode::wireframe());
            }

            if supported_features.polygon_mode_point {
                render_modes.push(RenderMode {
                    polygon_mode: wgpu::PolygonMode::Point,
                });
            }

            // Clone bind_group_requirements once for all pipelines
            let bind_group_requirements_clone = bind_group_requirements.clone();

            // Clone device for later use (to avoid borrow issues)
            let device_clone = device.0.clone();

            // Create all shader instances first
            let mut instances: Vec<(RenderMode, ShaderInstance)> = Vec::new();

            for render_mode in &render_modes {
                let render_pipeline =
                    device_clone.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&format!(
                            "{} Pipeline {:?}",
                            registration.shader, render_mode.polygon_mode
                        )),
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
                            polygon_mode: render_mode.polygon_mode,
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

                let shader_instance = ShaderInstance {
                    module: shader.clone(),
                    pipeline: render_pipeline,
                    bind_group_requirements: bind_group_requirements_clone.clone(),
                };

                instances.push((*render_mode, shader_instance));
            }

            // Register all instances with shader cache
            let mut shader_cache = world.get_resource_mut::<ShaderCache>().ok_or_else(|| {
                anyhow::anyhow!(
                    "ShaderCache resource not found - make sure RenderLayer is added before registering shaders"
                )
            })?;

            let mut shader_loader_opt = Some(shader_loader);

            for (i, (render_mode, shader_instance)) in instances.into_iter().enumerate() {
                // Only register shader loader once (for the first render mode)
                let loader: Box<dyn ShaderLoader> = if i == 0 {
                    shader_loader_opt.take().unwrap()
                } else {
                    // For other render modes, use dummy loader - only the first one is actually used for hot reloading
                    Box::new(StaticShaderLoader::new("", ""))
                };

                shader_cache.register_shader(
                    registration.shader.clone(),
                    render_mode,
                    loader,
                    shader_instance,
                );
            }
        }

        Ok(())
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let context = LayerContext {
            window: window.clone(),
            world: self.world.clone(),
            delta_time: Duration::ZERO,
        };

        let layers: Vec<Box<dyn Layer>> = self
            .layer_factories
            .iter()
            .map(|factory| factory.create(&context))
            .collect();

        self.state = Some(ApplicationState {
            window,
            layers,
            last_frame_time: Instant::now(),
        });

        // Perform queued shader registrations now that layers are initialized
        if let Err(e) = self.perform_shader_registrations() {
            log::error!("Failed to register shaders: {}", e);
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            let context = LayerContext {
                window: state.window.clone(),
                world: self.world.clone(),
                delta_time: Duration::ZERO,
            };

            for layer in &mut state.layers {
                layer.detach(&context);
            }
        }
        self.state = None;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let event = Arc::new(event);

        match *event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => match self.redraw() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {}
                Err(e) => {
                    log::error!("Unable to render {}", e);
                }
            },
            _ => {}
        }

        if let Some(state) = &mut self.state {
            let context = LayerContext {
                window: state.window.clone(),
                world: self.world.clone(),
                delta_time: Duration::ZERO,
            };

            for layer in &mut state.layers {
                layer.event(&context, LayerEvent::WindowEvent(event.clone()));
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}
