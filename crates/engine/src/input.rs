use crate::prelude::*;

use bevy_ecs::prelude::*;
use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Resource that tracks keyboard and mouse input state
#[derive(Resource, Default)]
pub struct InputState {
    /// Currently pressed keys
    pub keys_pressed: HashSet<KeyCode>,
    /// Mouse delta since last frame (x, y)
    pub mouse_delta: (f32, f32),
    /// Mouse position in window coordinates
    pub mouse_position: (f32, f32),
    /// Whether the mouse is captured for camera control
    pub mouse_captured: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Reset per-frame state (call at start of each frame)
    pub fn reset_frame(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    /// Handle key press
    pub fn press_key(&mut self, key: KeyCode) {
        self.keys_pressed.insert(key);
    }

    /// Handle key release
    pub fn release_key(&mut self, key: KeyCode) {
        self.keys_pressed.remove(&key);
    }

    /// Add mouse delta movement
    pub fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        if self.mouse_captured {
            self.mouse_delta.0 += dx;
            self.mouse_delta.1 += dy;
        }
    }

    /// Update mouse position
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    /// Toggle mouse capture
    pub fn toggle_mouse_capture(&mut self) {
        self.mouse_captured = !self.mouse_captured;
    }
}
