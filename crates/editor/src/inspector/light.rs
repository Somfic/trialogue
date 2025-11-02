use crate::prelude::*;
use trialogue_engine::prelude::*;

// Auto-register for inspection
crate::register_inspectable!(Light, "Light");

impl Inspectable for Light {
    fn inspect(&mut self, ui: &mut egui::Ui, _world: &World) {
        ui.label("Note: Position is controlled by Transform component");

        ui.horizontal(|ui| {
            ui.label("Intensity:");
            ui.add(
                egui::DragValue::new(&mut self.intensity)
                    .speed(0.1)
                    .range(0.0..=100.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Color:");
            ui.color_edit_button_rgb(&mut self.color);
        });
    }
}
