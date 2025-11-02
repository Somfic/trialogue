
use trialogue_editor::prelude::*;
use trialogue_engine::prelude::*;

#[derive(Component)]
pub struct Planet {
    pub seed: String,
}

impl Inspectable for Planet {
    fn inspect(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Seed:");
            ui.text_edit_singleline(&mut self.seed);
        });
    }
}
