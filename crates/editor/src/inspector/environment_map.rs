
use crate::prelude::*;
use trialogue_engine::prelude::*;

use rfd;

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
