use std::hash::{DefaultHasher, Hash, Hasher};

use crate::prelude::*;
use egui::DragValue;

#[derive(Component, Clone, PartialEq)]
pub struct Planet {
    pub seed: String,
    pub subdivisions: u32,
    pub terrain_config: TerrainConfig,
}

impl Planet {
    pub fn seed(&self) -> u32 {
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        hasher.finish() as u32
    }
}

#[derive(Clone, PartialEq)]
pub struct TerrainConfig {
    pub noise_scale: f32,
    pub noise_strength: f32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub persistence: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            noise_scale: 2.0,
            noise_strength: 0.15,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

// Auto-register Planet for inspection
trialogue_editor::register_inspectable!(Planet, "Planet");

impl Inspectable for Planet {
    fn inspect(&mut self, ui: &mut Ui, _world: &World) {
        ui.horizontal(|ui| {
            ui.label("Seed:");
            ui.text_edit_singleline(&mut self.seed);
        });

        ui.horizontal(|ui| {
            ui.label("Subdivisions:");
            ui.add(DragValue::new(&mut self.subdivisions).range(1..=100));
        });

        ui.horizontal(|ui| {
            ui.label("Noise Scale:");
            ui.add(
                DragValue::new(&mut self.terrain_config.noise_scale)
                    .speed(0.025)
                    .range(0.01..=5.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Noise Strength:");
            ui.add(
                DragValue::new(&mut self.terrain_config.noise_strength)
                    .speed(0.01)
                    .range(0.1..=20.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Octaves:");
            ui.add(DragValue::new(&mut self.terrain_config.octaves).range(1..=10));
        });

        ui.horizontal(|ui| {
            ui.label("Lacunarity:");
            ui.add(
                DragValue::new(&mut self.terrain_config.lacunarity)
                    .speed(0.025)
                    .range(1.5..=4.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Persistence:");
            ui.add(
                DragValue::new(&mut self.terrain_config.persistence)
                    .speed(0.01)
                    .range(0.1..=0.9),
            );
        });
    }
}
