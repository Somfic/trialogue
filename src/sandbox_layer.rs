use bevy_ecs::schedule::Schedule;
use trialogue::prelude::*;
use trialogue::{Layer, LayerContext};

pub struct SandboxLayer {
    schedule: Schedule,
}

impl SandboxLayer {
    pub fn new(_context: &LayerContext) -> Self {
        let mut schedule = Schedule::default();
        schedule.add_systems(orbit_spheres);

        Self { schedule }
    }
}

fn orbit_spheres(time: Res<Time>, mut query: Query<(&mut Transform, &Sphere)>) {
    let dt = time.0.as_secs_f32();

    for (mut transform, _sphere) in query.iter_mut() {
        // Calculate orbit radius in XZ plane from current position (first frame sets the radius)
        let orbit_radius = (transform.position.x.powi(2) + transform.position.z.powi(2)).sqrt();

        // Skip if at origin
        if orbit_radius < 0.01 {
            continue;
        }

        // Keep Y constant
        let y = transform.position.y;

        // Calculate angular velocity (adjust speed to your preference)
        let angular_velocity = 0.5; // radians per second

        // Calculate current angle from position
        let current_angle = transform.position.z.atan2(transform.position.x);

        // Add rotation based on delta time
        let angle = current_angle + angular_velocity * dt;

        // Update position in circular orbit around (0, y, 0)
        transform.position.x = orbit_radius * angle.cos();
        transform.position.y = y;
        transform.position.z = orbit_radius * angle.sin();
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
