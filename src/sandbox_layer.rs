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
    for (mut transform, mesh) in query.iter_mut() {
        transform.rotation += *UnitQuaternion::from_euler_angles(1.0, 0.0, 0.0);
        println!("Rotation: {:?}", transform.rotation);
    }
}

impl Layer for SandboxLayer {
    fn frame(
        &mut self,
        context: &trialogue::LayerContext,
    ) -> std::result::Result<(), wgpu::SurfaceError> {
        {
            let mut world = context.world.lock().unwrap();
            self.schedule.run(&mut world);
        }

        // Request next frame for continuous animation
        context.window.request_redraw();

        Ok(())
    }

    fn detach(&mut self, context: &trialogue::LayerContext) {}
}
