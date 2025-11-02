use crate::prelude::*;
use egui::DragValue;

#[derive(Component, Clone, PartialEq)]
pub struct Planet {
    pub seed: String,
    pub subdivisions: u32,
}

// Auto-register Planet for inspection
trialogue_editor::register_inspectable!(Planet, "Planet");

impl Inspectable for Planet {
    fn inspect(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Seed:");
            ui.text_edit_singleline(&mut self.seed);
        });

        ui.horizontal(|ui| {
            ui.label("Subdivisions:");
            ui.add(DragValue::new(&mut self.subdivisions).range(1..=30));
        });
    }
}
