use std::sync::{Arc, Mutex};
use trialogue_engine::{layers::raytracer::ShaderError, prelude::*};

use super::editor_state::EditorState;

pub fn draw_ui(
    context: &LayerContext,
    ctx: &egui::Context,
    world: &Arc<Mutex<World>>,
    viewport_texture_id: Option<egui::TextureId>,
    editor_state: &mut EditorState,
) {
    let mut world = world.lock().unwrap();

    // Scene
    egui::SidePanel::left("scene")
        .default_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Scene");
            ui.separator();
            world
                .query::<(Entity, &Tag)>()
                .iter(&world)
                .for_each(|(entity, tag)| {
                    if ui.button(format!("{}", tag.label)).clicked() {
                        editor_state.select_entity(entity, tag.clone());
                    }
                });
        });

    // Get viewport size from world
    let viewport_size = *world.get_resource::<WindowSize>().unwrap();

    // Entity Inspector
    egui::SidePanel::right("Entity")
        .default_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();
            if let Some((entity, tag)) = &editor_state.selected_entity {
                ui.label(format!("{}", tag.label));
                ui.separator();

                // Use the component inspector from editor state
                editor_state
                    .component_inspector
                    .inspect_entity(&mut world, *entity, ui);
            } else {
                ui.label("No entity selected");
            }
        });

    // Console panel at the bottom for shader errors
    egui::TopBottomPanel::bottom("console")
        .default_height(150.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Console");
            ui.separator();

            // Check for shader errors
            if let Some(shader_error_res) = world.get_resource::<ShaderError>() {
                if !shader_error_res.0.is_empty() {
                    for (shader_name, error) in &shader_error_res.0 {
                        ui.colored_label(egui::Color32::RED, format!("❌ {} Compilation Error:", shader_name));
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .id_salt(shader_name)
                            .max_height(100.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut error.as_str())
                                        .code_editor()
                                        .desired_width(f32::INFINITY),
                                );
                            });
                        ui.add_space(10.0);
                    }
                } else {
                    ui.colored_label(egui::Color32::GREEN, "✓ All shaders compiled successfully");
                }
            } else {
                ui.label("No shader status available");
            }
        });

    // Viewport
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            // This is where the viewport will be rendered
            // We'll use the available rect to determine viewport size
            let viewport_rect = ui.available_rect_before_wrap();

            // Update viewport size in world resource
            let new_width = viewport_rect.width() as u32;
            let new_height = viewport_rect.height() as u32;

            if new_width > 0 && new_height > 0 {
                let mut window_size = world.get_resource_mut::<WindowSize>().unwrap();
                if window_size.width != new_width || window_size.height != new_height {
                    window_size.width = new_width;
                    window_size.height = new_height;
                }
            }

            // Display the viewport texture if available
            if let Some(texture_id) = viewport_texture_id {
                // Use the actual texture size for 1:1 pixel mapping
                let size = [viewport_size.width as f32, viewport_size.height as f32];
                ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture_id, size))
                        .fit_to_exact_size(egui::vec2(size[0], size[1])),
                );
            } else {
                // Paint a placeholder background for the viewport area
                ui.painter()
                    .rect_filled(viewport_rect, 0.0, egui::Color32::from_rgb(0, 0, 0));
            }
        });

    // floating panel for stats
    egui::Window::new("Stats")
        .default_pos(egui::pos2(20.0, 20.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Rendering Stats:");
            ui.separator();
            let dt = context.delta_time.as_millis();
            ui.label(format!("Frame Time: {} ms", dt));
        });
}
