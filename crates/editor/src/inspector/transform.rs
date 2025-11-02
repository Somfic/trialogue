use crate::prelude::*;
use trialogue_engine::prelude::*;

// Auto-register Transform for inspection
crate::register_inspectable!(Transform, "Transform");

impl Inspectable for Transform {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Position:");
            ui.add(
                egui::DragValue::new(&mut self.position.x)
                    .prefix("x: ")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.position.y)
                    .prefix("y: ")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.position.z)
                    .prefix("z: ")
                    .speed(0.1),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Scale:");
            ui.add(
                egui::DragValue::new(&mut self.scale.x)
                    .prefix("x: ")
                    .speed(0.01),
            );
            ui.add(
                egui::DragValue::new(&mut self.scale.y)
                    .prefix("y: ")
                    .speed(0.01),
            );
            ui.add(
                egui::DragValue::new(&mut self.scale.z)
                    .prefix("z: ")
                    .speed(0.01),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Rotation:");
            let euler = self.rotation.euler_angles();
            let mut pitch = euler.0.to_degrees();
            let mut yaw = euler.1.to_degrees();
            let mut roll = euler.2.to_degrees();

            ui.add(
                egui::DragValue::new(&mut pitch)
                    .prefix("pitch: ")
                    .speed(1.0)
                    .suffix("°"),
            );
            ui.add(
                egui::DragValue::new(&mut yaw)
                    .prefix("yaw: ")
                    .speed(1.0)
                    .suffix("°"),
            );
            ui.add(
                egui::DragValue::new(&mut roll)
                    .prefix("roll: ")
                    .speed(1.0)
                    .suffix("°"),
            );

            self.rotation = UnitQuaternion::from_euler_angles(
                pitch.to_radians(),
                yaw.to_radians(),
                roll.to_radians(),
            );
        });
    }
}
