use bevy_ecs::schedule::Schedule;
use trialogue_engine::prelude::*;
use trialogue_engine::{Layer, LayerContext};

pub struct SandboxLayer {
    schedule: Schedule,
}

impl SandboxLayer {
    pub fn new(_context: &LayerContext) -> Self {
        let schedule = Schedule::default();
        // Add your game systems here
        // schedule.add_systems(your_game_system);

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
