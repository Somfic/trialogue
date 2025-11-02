use crate::prelude::*;

use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use std::sync::{Arc, Mutex};
use trialogue_engine::{Layer, LayerContext};

#[derive(Resource, Clone)]
pub struct WorldHandle(pub Arc<Mutex<World>>);

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
        schedule.add_systems(crate::systems::planet_mesh);
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
        world.insert_resource(WorldHandle(context.world.clone()));

        self.schedule.run(&mut world);

        Ok(())
    }

    fn detach(&mut self, _context: &trialogue_engine::LayerContext) {}
}
