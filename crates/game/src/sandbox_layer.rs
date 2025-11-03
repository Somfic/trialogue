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
        schedule.add_systems((crate::systems::planet_mesh, apply_async_entity_results));
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
