use crate::prelude::*;
use winit::keyboard::KeyCode;

/// Camera controller system for WASD + mouse look
/// Right-click to capture mouse, WASD to move, mouse to look around
pub fn update_camera_controller(
    mut camera_query: Query<(&mut Transform, &mut Camera, &mut CameraController)>,
    input: Res<InputState>,
    time: Res<Time>,
) {
    for (mut transform, mut camera, mut controller) in camera_query.iter_mut() {
        let dt = time.0.as_secs_f32();
        
        // Mouse look (only when captured)
        if input.mouse_captured {
            controller.yaw -= input.mouse_delta.0 * controller.look_sensitivity;
            controller.pitch -= input.mouse_delta.1 * controller.look_sensitivity;
            
            // Clamp pitch to avoid gimbal lock
            controller.pitch = controller.pitch.clamp(-1.5, 1.5);
        }
        
        // Calculate forward/right vectors from yaw/pitch
        let forward = Vector3::new(
            controller.yaw.cos() * controller.pitch.cos(),
            controller.pitch.sin(),
            controller.yaw.sin() * controller.pitch.cos(),
        ).normalize();
        
        let right = Vector3::new(
            -controller.yaw.sin(),
            0.0,
            controller.yaw.cos(),
        ).normalize();
        
        let up = Vector3::new(0.0, 1.0, 0.0);
        
        // WASD movement
        let mut movement = Vector3::zeros();
        
        if input.is_key_pressed(KeyCode::KeyW) {
            movement += forward;
        }
        if input.is_key_pressed(KeyCode::KeyS) {
            movement -= forward;
        }
        if input.is_key_pressed(KeyCode::KeyA) {
            movement -= right;
        }
        if input.is_key_pressed(KeyCode::KeyD) {
            movement += right;
        }
        if input.is_key_pressed(KeyCode::Space) {
            movement += up;
        }
        if input.is_key_pressed(KeyCode::ShiftLeft) {
            movement -= up;
        }
        
        // Normalize and apply speed
        if movement.magnitude() > 0.0 {
            movement = movement.normalize() * controller.move_speed * dt;
            transform.position += movement;
        }
        
        // Update camera target to look in forward direction
        camera.target = transform.position + forward * 10.0;
    }
}
