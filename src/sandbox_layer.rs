use bevy_ecs::schedule::Schedule;
use trialogue::prelude::*;
use trialogue::{Layer, LayerContext};

pub struct SandboxLayer {
    schedule: Schedule,
}

impl SandboxLayer {
    pub fn new(context: &LayerContext) -> Self {
        let mut schedule = Schedule::default();
        schedule.add_systems(rotate);

        Self { schedule }
    }
}

fn rotate(time: Res<Time>, mut query: Query<(&mut Transform, &Mesh)>) {
    for (mut transform, _mesh) in query.iter_mut() {
        let dt = time.0.as_secs_f32();
        let delta = UnitQuaternion::from_euler_angles(0.0, 0.0, 1.0 * dt);
        transform.rotation = transform.rotation * delta;
    }
}

impl Layer for SandboxLayer {
    fn frame(
        &mut self,
        context: &trialogue::LayerContext,
    ) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut world = context.world.lock().unwrap();
        world.insert_resource(Time(context.delta_time));

        self.schedule.run(&mut world);

        Ok(())
    }

    fn detach(&mut self, _context: &trialogue::LayerContext) {}
}
