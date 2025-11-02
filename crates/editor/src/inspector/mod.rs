mod camera;
mod environment_map;
mod light;
mod material;
mod sphere;
mod transform;
mod mesh;

use crate::prelude::*;
use bevy_ecs::component::Mutable;

pub trait Inspectable {
    fn inspect(&mut self, ui: &mut egui::Ui, world: &World);
}

/// Trait for components that can be inspected in read-only mode (no Clone/PartialEq needed)
pub trait InspectableReadOnly {
    fn inspect_readonly(&self, ui: &mut egui::Ui);
}

// Registry entry for auto-registration of inspectable components
pub struct InspectableRegistration {
    pub name: &'static str,
    pub register_fn: fn(&mut ComponentInspector),
}

// Collect all inspectable registrations at link-time
inventory::collect!(InspectableRegistration);

// Macro to simplify registering inspectable components
#[macro_export]
macro_rules! register_inspectable {
    ($type:ty, $name:expr) => {
        inventory::submit! {
            $crate::inspector::InspectableRegistration {
                name: $name,
                register_fn: |inspector| {
                    inspector.register::<$type>($name);
                },
            }
        }
    };
}

// Macro to simplify registering read-only inspectable components
#[macro_export]
macro_rules! register_inspectable_readonly {
    ($type:ty, $name:expr) => {
        inventory::submit! {
            $crate::inspector::InspectableRegistration {
                name: $name,
                register_fn: |inspector| {
                    inspector.register_readonly::<$type>($name);
                },
            }
        }
    };
}

// Create and configure the component inspector with all registered components
pub fn create_component_inspector() -> ComponentInspector {
    let mut inspector = ComponentInspector::new();

    // Auto-register all components that used the register_inspectable! macro
    let mut count = 0;
    for registration in inventory::iter::<InspectableRegistration> {
        log::info!("Auto-registering inspector for: {}", registration.name);
        (registration.register_fn)(&mut inspector);
        count += 1;
    }
    log::info!("Total inspectable components registered: {}", count);

    inspector
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
        T: Component<Mutability = Mutable> + Inspectable + Clone + PartialEq,
    {
        let inspect_fn: InspectFn = Box::new(move |world, entity, ui| {
            // First, extract the component data we need
            let component_data = if let Ok(entity_ref) = world.get_entity(entity) {
                entity_ref.get::<T>().map(|c| c.clone())
            } else {
                None
            };

            if let Some(mut component) = component_data {
                let mut changed = false;
                ui.collapsing(name, |ui| {
                    // Clone the component before inspection
                    let before = component.clone();

                    // Call inspect (may modify the component)
                    component.inspect(ui, world);

                    // Compare before and after - only mark as changed if different
                    if before != component {
                        changed = true;
                    }
                });

                // If changed, write back to the entity
                if changed {
                    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                        if let Some(mut comp) = entity_mut.get_mut::<T>() {
                            *comp = component;
                        }
                    }
                }
            }
        });
        self.inspectors.push((name, inspect_fn));
    }

    /// Register a component type for read-only inspection (no Clone/PartialEq needed)
    pub fn register_readonly<T>(&mut self, name: &'static str)
    where
        T: Component + InspectableReadOnly,
    {
        let inspect_fn: InspectFn = Box::new(move |world, entity, ui| {
            if let Ok(entity_ref) = world.get_entity(entity) {
                if let Some(component) = entity_ref.get::<T>() {
                    ui.collapsing(name, |ui| {
                        component.inspect_readonly(ui);
                    });
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
