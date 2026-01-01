use crate::app_state::AppState;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub struct CameraControls;

impl Plugin for CameraControls {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (camera_keyboard_controls, camera_scroll_controls).run_if(in_state(AppState::InGame)),
        );
    }
}

fn camera_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera_q.single_mut() else {
        return;
    };

    let speed = 300.0 * time.delta_secs();

    // Calculate forward and right directions based on camera orientation
    // but only in the XZ plane (ignore Y component for movement)
    let forward = camera.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize();
    let right_xz = Vec3::new(forward.z, 0.0, -forward.x).normalize();

    // WASD movement
    if keyboard.pressed(KeyCode::KeyW) {
        camera.translation += forward_xz * speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        camera.translation -= forward_xz * speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        camera.translation += right_xz * speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        camera.translation -= right_xz * speed;
    }
}

fn camera_scroll_controls(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut camera) = camera_q.single_mut() else {
        return;
    };

    const SENS: f32 = 30.0;
    const MIN_Y: f32 = 100.0;
    const MAX_Y: f32 = 800.0;
    const MIN_ANGLE: f32 = 20.0;
    const MAX_ANGLE: f32 = 60.0;
    for event in scroll_events.read() {
        let zoom_amount = event.y * SENS;

        let current_height = camera.translation.y;
        let new_height = (current_height - zoom_amount).clamp(MIN_Y, MAX_Y);
        camera.translation.y = new_height;

        // Calculate rotation based on zoom level (0.0 = closest, 1.0 = furthest)
        let zoom_factor = (new_height - MIN_Y) / (MAX_Y - MIN_Y);

        // Camera Rotation
        let rotation_degrees = MIN_ANGLE + (zoom_factor * (MAX_ANGLE - MIN_ANGLE));
        let rotation_radians = rotation_degrees.to_radians();

        // Apply rotation (rotate around X axis to tilt camera)
        camera.rotation = Quat::from_rotation_x(-rotation_radians);
    }
}
