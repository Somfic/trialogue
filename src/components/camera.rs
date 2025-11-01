use crate::prelude::*;

#[derive(Component)]
pub struct Camera {
    pub is_main: bool,
    pub target: Point3<f32>,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aperture: f32,
    pub focus_distance: f32,
}

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

#[derive(Component)]
pub struct GpuCamera {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub aspect: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: Matrix4<f32>,
}

#[derive(Component)]
pub struct RenderTarget {}

#[derive(Component)]
pub struct GpuRenderTarget {
    pub texture: wgpu::Texture,
}
