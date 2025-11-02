
use crate::prelude::*;
use trialogue_engine::prelude::*;

impl Inspectable for Camera {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.is_main, "Is Main Camera");

        ui.horizontal(|ui| {
            ui.label("Target:");
            ui.add(
                egui::DragValue::new(&mut self.target.x)
                    .prefix("x: ")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.target.y)
                    .prefix("y: ")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.target.z)
                    .prefix("z: ")
                    .speed(0.1),
            );
        });

        ui.horizontal(|ui| {
            ui.label("FOV Y:");
            ui.add(egui::DragValue::new(&mut self.fovy).speed(0.01));
        });

        ui.horizontal(|ui| {
            ui.label("Near:");
            ui.add(egui::DragValue::new(&mut self.znear).speed(0.01));
            ui.label("Far:");
            ui.add(egui::DragValue::new(&mut self.zfar).speed(0.1));
        });

        ui.collapsing("Depth of Field", |ui| {
            let mut dof_enabled = self.aperture > 0.0;
            if ui
                .checkbox(&mut dof_enabled, "Enable Depth of Field")
                .changed()
            {
                if dof_enabled {
                    self.aperture = 0.1;
                } else {
                    self.aperture = 0.0;
                }
            }

            ui.add_enabled(
                dof_enabled,
                egui::DragValue::new(&mut self.aperture)
                    .prefix("Aperture: ")
                    .speed(0.0001),
            );

            ui.add_enabled(
                dof_enabled,
                egui::DragValue::new(&mut self.focus_distance)
                    .prefix("Focus Distance: ")
                    .speed(0.1),
            );
        });
    }
}
