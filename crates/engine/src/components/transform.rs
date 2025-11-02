use crate::prelude::*;

#[derive(Component)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

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

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Component)]
pub struct GpuTransform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
