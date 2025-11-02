
use crate::prelude::*;
use trialogue_engine::prelude::*;

use crate::inspector::{ComponentInspector, create_component_inspector};

#[derive(Default)]
pub struct EditorState {
    pub selected_entity: Option<(Entity, Tag)>,
    pub component_inspector: ComponentInspector,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            component_inspector: create_component_inspector(),
        }
    }

    pub fn select_entity(&mut self, entity: Entity, tag: Tag) {
        self.selected_entity = Some((entity, tag));
    }

    pub fn deselect_entity(&mut self) {
        self.selected_entity = None;
    }

    pub fn is_entity_selected(&self, entity: Entity) -> bool {
        self.selected_entity
            .as_ref()
            .is_some_and(|(e, _)| entity == *e)
    }
}
