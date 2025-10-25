use bevy_ecs::{
    bundle::Bundle,
    world::{EntityWorldMut, World},
};
use std::sync::{Arc, Mutex};
use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};

pub type Result<T> = anyhow::Result<T>;

pub mod layer;

pub use layer::renderer::RenderLayer;

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
            layer_factories: vec![Box::new(crate::RenderLayerFactory)],
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

pub struct RenderLayerFactory;

impl LayerFactory for RenderLayerFactory {
    fn create(&self, context: &LayerContext) -> Box<dyn Layer> {
        Box::new(RenderLayer::new(context))
    }
}

pub struct Application {
    layer_factories: Vec<Box<dyn LayerFactory>>,
    state: Option<ApplicationState>,
    world: Arc<Mutex<World>>,
}

pub struct ApplicationState {
    window: Arc<Window>,
    layers: Vec<Box<dyn Layer>>,
}

impl Application {
    fn redraw(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let state = match &mut self.state {
            Some(state) => state,
            None => return Ok(()),
        };

        let context = LayerContext {
            window: state.window.clone(),
            world: self.world.clone(),
        };

        for layer in &mut state.layers {
            layer.frame(&context)?;
        }

        Ok(())
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) {
        self.world.lock().unwrap().spawn(bundle);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let world = Arc::new(Mutex::new(World::new()));

        let context = LayerContext {
            window: window.clone(),
            world: world.clone(),
        };

        let layers: Vec<Box<dyn Layer>> = self
            .layer_factories
            .iter()
            .map(|factory| factory.create(&context))
            .collect();

        self.state = Some(ApplicationState { window, layers });
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            let context = LayerContext {
                window: state.window.clone(),
                world: self.world.clone(),
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
            };

            for layer in &mut state.layers {
                layer.event(&context, LayerEvent::WindowEvent(event.clone()));
            }
        }
    }
}
