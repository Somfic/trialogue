use crate::prelude::*;

/// Simple WASD + Mouse camera controller
#[derive(Component, Clone)]
pub struct CameraController {
    /// Movement speed in units per second
    pub move_speed: f32,
    /// Mouse look sensitivity (radians per pixel)
    pub look_sensitivity: f32,
    /// Current yaw (rotation around Y axis)
    pub yaw: f32,
    /// Current pitch (rotation around X axis)
    pub pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 100.0,
            look_sensitivity: 0.002,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl CameraController {
    pub fn new(move_speed: f32) -> Self {
        Self {
            move_speed,
            ..Default::default()
        }
    }
}
