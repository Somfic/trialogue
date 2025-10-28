use bevy_ecs::schedule::Schedule;
use trialogue::layer::renderer::{Mesh, Transform};
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

fn rotate(mut query: Query<(&mut Transform, &Mesh)>) {
    for (mut transform, _mesh) in query.iter_mut() {
        let delta = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.01);
        let current = UnitQuaternion::from_quaternion(transform.rotation);
        transform.rotation = (current * delta).into_inner();
    }
}

impl Layer for SandboxLayer {
    fn frame(
        &mut self,
        context: &trialogue::LayerContext,
    ) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut world = context.world.lock().unwrap();
        self.schedule.run(&mut world);
        Ok(())
    }

    fn detach(&mut self, _context: &trialogue::LayerContext) {}
}
