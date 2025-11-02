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

        // Check if this is an emissive sphere (any component > 1.0)
        let is_emissive = self.color[0] > 1.0 || self.color[1] > 1.0 || self.color[2] > 1.0;

        ui.horizontal(|ui| {
            ui.label("Color:");

            if is_emissive {
                // HDR mode - show sliders for values > 1.0
                ui.label("(HDR)");
            } else {
                // Regular mode - show color picker
                ui.color_edit_button_rgb(&mut self.color);
            }
        });

        if is_emissive {
            // HDR sliders for each component
            ui.horizontal(|ui| {
                ui.label("R:");
                ui.add(egui::Slider::new(&mut self.color[0], 0.0..=20.0).logarithmic(true));
            });
            ui.horizontal(|ui| {
                ui.label("G:");
                ui.add(egui::Slider::new(&mut self.color[1], 0.0..=20.0).logarithmic(true));
            });
            ui.horizontal(|ui| {
                ui.label("B:");
                ui.add(egui::Slider::new(&mut self.color[2], 0.0..=20.0).logarithmic(true));
            });
        }

        // Button to toggle between regular and emissive
        if ui
            .button(if is_emissive {
                "Make Non-Emissive"
            } else {
                "Make Emissive"
            })
            .clicked()
        {
            if is_emissive {
                // Clamp to [0, 1] range
                self.color[0] = self.color[0].min(1.0);
                self.color[1] = self.color[1].min(1.0);
                self.color[2] = self.color[2].min(1.0);
            } else {
                // Boost to emissive range (e.g., 5.0)
                self.color[0] *= 5.0;
                self.color[1] *= 5.0;
                self.color[2] *= 5.0;
            }
        }

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

/// User-facing component for environment map
/// Provide either a path or raw bytes to an HDR image
#[derive(Component)]
pub struct EnvironmentMap {
    pub bytes: Vec<u8>,
}

impl Inspectable for EnvironmentMap {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Environment Map ({} bytes)", self.bytes.len()));

        if ui.button("Load HDR file...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("HDR Images", &["hdr", "exr"])
                .add_filter("All Images", &["hdr", "exr", "png", "jpg", "jpeg"])
                .pick_file()
            {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        self.bytes = bytes;
                        log::info!("Loaded environment map: {:?}", path);
                    }
                    Err(e) => {
                        log::error!("Failed to load environment map: {}", e);
                    }
                }
            }
        }
    }
}

/// GPU-side component for environment map texture
#[derive(Component)]
pub struct GpuEnvironmentMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bytes_hash: u64, // Hash of the source bytes to detect actual changes
}
