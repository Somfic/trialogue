mod camera;
mod inspector;
mod label;
mod material;
mod mesh;
mod raytracer;
mod resources;
mod texture;
mod transform;

use bevy_ecs::{component::Mutable, prelude::*};
pub use camera::*;
pub use label::*;
pub use material::*;
pub use mesh::*;
pub use raytracer::*;
pub use resources::*;
pub use texture::*;
pub use transform::*;

/// Create and configure the component inspector with all registered components
pub fn create_component_inspector() -> ComponentInspector {
    let mut inspector = ComponentInspector::new();

    // Register inspectable components
    inspector.register::<Transform>("Transform");
    inspector.register::<Camera>("Camera");
    inspector.register::<Sphere>("Sphere");
    inspector.register::<Light>("Light");
    inspector.register::<EnvironmentMap>("Environment Map");
    inspector.register::<Material>("Material");

    inspector
}

pub trait Inspectable {
    fn inspect(&mut self, ui: &mut egui::Ui);
}

// Store inspection logic that can be called per component
type InspectFn = Box<dyn Fn(&mut World, Entity, &mut egui::Ui) + Send + Sync>;

pub struct ComponentInspector {
    inspectors: Vec<(&'static str, InspectFn)>,
}

impl ComponentInspector {
    pub fn new() -> Self {
        Self {
            inspectors: Vec::new(),
        }
    }

    /// Register a component type with its inspector function
    pub fn register<T>(&mut self, name: &'static str)
    where
        T: Component<Mutability = Mutable> + Inspectable,
    {
        let inspect_fn: InspectFn = Box::new(move |world, entity, ui| {
            // Get mutable entity reference
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut component) = entity_mut.get_mut::<T>() {
                    ui.collapsing(name, |ui| {
                        // Dereference Mut<T> to get &mut T
                        (&mut *component).inspect(ui);
                    });
                }
            }
        });
        self.inspectors.push((name, inspect_fn));
    }

    /// Register a component type without inspection (read-only display)
    pub fn register_readonly<T: Component>(&mut self, name: &'static str) {
        let inspect_fn: InspectFn = Box::new(move |world, entity, ui| {
            if let Ok(entity_ref) = world.get_entity(entity) {
                if entity_ref.contains::<T>() {
                    ui.label(format!("• {}", name));
                }
            }
        });
        self.inspectors.push((name, inspect_fn));
    }

    /// Inspect all registered components for an entity
    pub fn inspect_entity(&self, world: &mut World, entity: Entity, ui: &mut egui::Ui) {
        for (_name, inspect_fn) in &self.inspectors {
            inspect_fn(world, entity, ui);
        }
    }
}

impl Default for ComponentInspector {
    fn default() -> Self {
        Self::new()
    }
}
