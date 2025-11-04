use crate::prelude::*;
use trialogue_engine::prelude::*;

// Auto-register for inspection
crate::register_inspectable!(Material, "Material");

impl Inspectable for Material {
    fn inspect(&mut self, ui: &mut egui::Ui, world: &World) {
        ui.horizontal(|ui| {
            ui.label("Shader:");
            egui::ComboBox::from_id_source("shader_combo")
                .selected_text(format!("{:?}", self.shader))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.shader, Shader::Standard, "Standard");
                    ui.selectable_value(&mut self.shader, Shader::Instanced, "Instanced");
                    ui.selectable_value(&mut self.shader, Shader::Raytracer, "Raytracer");
                });
        });

        // Get supported features
        let supported_features = world.get_resource::<SupportedFeatures>();

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

                    // Only show Line mode if supported
                    if supported_features
                        .map(|f| f.polygon_mode_line)
                        .unwrap_or(false)
                    {
                        ui.selectable_value(
                            &mut self.render_mode.polygon_mode,
                            wgpu::PolygonMode::Line,
                            "Line",
                        );
                    }

                    // Only show Point mode if supported
                    if supported_features
                        .map(|f| f.polygon_mode_point)
                        .unwrap_or(false)
                    {
                        ui.selectable_value(
                            &mut self.render_mode.polygon_mode,
                            wgpu::PolygonMode::Point,
                            "Point",
                        );
                    }
                });
        });

        // Reset to Fill if current mode is not supported
        if let Some(features) = supported_features {
            match self.render_mode.polygon_mode {
                wgpu::PolygonMode::Line if !features.polygon_mode_line => {
                    self.render_mode.polygon_mode = wgpu::PolygonMode::Fill;
                }
                wgpu::PolygonMode::Point if !features.polygon_mode_point => {
                    self.render_mode.polygon_mode = wgpu::PolygonMode::Fill;
                }
                _ => {}
            }
        }
    }
}
