use crate::prelude::*;

use bevy_ecs::schedule::Schedule;
use trialogue_engine::{Layer, LayerContext};

pub struct SandboxLayer {
    schedule: Schedule,
}

impl SandboxLayer {
    pub fn new(context: &LayerContext) -> Self {
        // Initialize resources
        {
            let mut world = context.world.lock().unwrap();
            world.insert_resource(AsyncTaskTracker::<Entity>::new());
        }

        let mut schedule = Schedule::default();
        // Use chain() to run systems sequentially and avoid query conflicts
        schedule.add_systems((
            apply_async_entity_results,
            crate::systems::planet_mesh,
            crate::systems::initialize_planet_lod_chunks,
            crate::systems::update_planet_lod_raycast,
            crate::systems::generate_chunk_meshes,
            crate::systems::copy_material_to_children,
            crate::systems::copy_texture_to_children,
            crate::systems::update_children_transforms,
        ).chain());
        Self { schedule }
    }
}

impl Layer for SandboxLayer {
    fn frame(
        &mut self,
        context: &trialogue_engine::LayerContext,
    ) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut world = context.world.lock().unwrap();
        world.insert_resource(Time(context.delta_time));

        self.schedule.run(&mut world);

        Ok(())
    }

    fn detach(&mut self, _context: &trialogue_engine::LayerContext) {}
}
