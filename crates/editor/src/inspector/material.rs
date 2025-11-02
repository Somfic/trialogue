use crate::prelude::*;
use trialogue_engine::prelude::*;

// Auto-register for inspection
crate::register_inspectable!(Material, "Material");

impl Inspectable for Material {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Shader:");
            egui::ComboBox::from_id_source("shader_combo")
                .selected_text(format!("{:?}", self.shader))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.shader, Shader::Standard, "Standard");
                    ui.selectable_value(&mut self.shader, Shader::Raytracer, "Raytracer");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Polygon Mode:");
            egui::ComboBox::from_id_source("polygon_mode_combo")
                .selected_text(format!("{:?}", self.render_mode.polygon_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.render_mode.polygon_mode,
                        wgpu::PolygonMode::Fill,
                        "Fill",
                    );
                    ui.selectable_value(
                        &mut self.render_mode.polygon_mode,
                        wgpu::PolygonMode::Line,
                        "Line",
                    );
                    ui.selectable_value(
                        &mut self.render_mode.polygon_mode,
                        wgpu::PolygonMode::Point,
                        "Point",
                    );
                });
        });
    }
}
