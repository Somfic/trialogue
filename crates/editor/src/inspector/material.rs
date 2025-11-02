
use crate::prelude::*;
use trialogue_engine::prelude::*;

impl Inspectable for Material {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Shader:");
            let mut shader_str = self.shader.to_string();

            // dropdown for shader selection
            egui::ComboBox::from_id_source("shader_combo")
                .selected_text(&shader_str)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut shader_str,
                            Shader::Standard.to_string(),
                            Shader::Standard.to_string(),
                        )
                        .clicked()
                    {
                        self.shader = Shader::Standard;
                    }
                    if ui
                        .selectable_value(
                            &mut shader_str,
                            Shader::Raytracer.to_string(),
                            Shader::Raytracer.to_string(),
                        )
                        .clicked()
                    {
                        self.shader = Shader::Raytracer;
                    }
                });
        });
    }
}
