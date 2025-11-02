use bevy_ecs::component::Component;
use std::fmt::Display;

use crate::prelude::Inspectable;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Shader {
    Standard,
    Raytracer,
}

impl Display for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shader::Standard => write!(f, "standard"),
            Shader::Raytracer => write!(f, "raytracer"),
        }
    }
}

/// Material component that references a shader by name
#[derive(Component, Clone)]
pub struct Material {
    /// Name of the shader to use (e.g., "standard", "pbr", "unlit")
    pub shader: Shader,
    // Future material properties can be added here:
    // pub albedo: Color,
    // pub roughness: f32,
    // pub metallic: f32,
    // etc.
}

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

impl Material {
    pub fn new(shader: Shader) -> Self {
        Self { shader }
    }

    /// Create a material using the standard shader
    pub fn standard() -> Self {
        Self::new(Shader::Standard)
    }
}
