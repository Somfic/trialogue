use crate::prelude::*;
use bevy_ecs::prelude::*;

/// User-facing component for spawning spheres in the raytracer scene
/// Position is taken from the Transform component
/// The Transform's scale.x is used as the radius (uniform scaling)
#[derive(Component, Clone, Copy)]
pub struct Sphere {
    pub color: [f32; 3],
    pub material_type: u32, // 0 = lambertian, 1 = metal, 2 = dielectric
}

impl Inspectable for Sphere {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Note: Position and radius are controlled by Transform component");

        ui.horizontal(|ui| {
            ui.label("Color:");
            ui.color_edit_button_rgb(&mut self.color);
        });

        ui.horizontal(|ui| {
            ui.label("Material:");
            let material_names = ["Lambertian", "Metal", "Dielectric"];
            let mut selected = self.material_type as usize;
            egui::ComboBox::from_id_salt("material_type")
                .selected_text(material_names.get(selected).copied().unwrap_or("Unknown"))
                .show_ui(ui, |ui| {
                    for (i, name) in material_names.iter().enumerate() {
                        ui.selectable_value(&mut selected, i, *name);
                    }
                });
            self.material_type = selected as u32;
        });
    }
}

/// User-facing component for spawning lights in the raytracer scene
/// Position is taken from the Transform component
#[derive(Component, Clone, Copy)]
pub struct Light {
    pub intensity: f32,
    pub color: [f32; 3],
}

impl Inspectable for Light {
    fn inspect(&mut self, ui: &mut egui::Ui) {
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

/// GPU-side component that holds the buffer data for the entire raytracer scene
/// This is attached to a single entity that manages the scene
#[derive(Component)]
pub struct GpuRaytracerScene {
    pub spheres_buffer: wgpu::Buffer,
    pub lights_buffer: wgpu::Buffer,
    pub sphere_count: u32,
    pub light_count: u32,
}
