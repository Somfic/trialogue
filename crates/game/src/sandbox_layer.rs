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
        
        // Async task system
        schedule.add_systems(apply_async_entity_results);
        
        // Camera controller (automatic circular motion for now)
        schedule.add_systems(crate::systems::update_camera_controller);
        
        // NEW Instanced LOD systems (1 entity → many instances)
        schedule.add_systems(crate::systems::initialize_instanced_quad_lod);
        schedule.add_systems(crate::systems::update_instanced_quad_lod);
        schedule.add_systems(crate::systems::update_instanced_lod_transforms);
        schedule.add_systems(crate::systems::clear_instanced_lod_dirty_flags);
        
        // OLD Quad LOD systems (entity-per-chunk architecture - disabled for instanced test)
        // schedule.add_systems(crate::systems::initialize_quad_lod);
        // schedule.add_systems(crate::systems::generate_quad_chunk_meshes);
        // schedule.add_systems(crate::systems::split_quad_chunks);
        // schedule.add_systems(crate::systems::collapse_quad_chunks);
        
        // Planet systems (also using old architecture)
        schedule.add_systems(crate::systems::planet_mesh);
        schedule.add_systems(crate::systems::initialize_planet_lod_chunks);
        schedule.add_systems(crate::systems::update_planet_lod_raycast);
        schedule.add_systems(crate::systems::generate_chunk_meshes);
        schedule.add_systems(crate::systems::copy_material_to_children);
        schedule.add_systems(crate::systems::copy_texture_to_children);
        schedule.add_systems(crate::systems::update_children_transforms);
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
