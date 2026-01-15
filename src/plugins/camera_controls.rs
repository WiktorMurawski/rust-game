use crate::resources::MapSize;
use crate::states::AppState;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CameraSetup;

pub struct GameCamera;

impl Plugin for GameCamera {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_camera.in_set(CameraSetup))
            .add_systems(OnEnter(AppState::InGame), change_background_color)
            .add_systems(
                Update,
                (camera_keyboard_controls, camera_scroll_controls)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn change_background_color(mut clear_color: ResMut<ClearColor>) {
    const BACKGROUND_COLOR: Color = Color::srgb_u8(69, 199, 255);
    clear_color.0 = BACKGROUND_COLOR;
}

fn setup_camera(mut commands: Commands) {
    let mut transform =
        Transform::from_xyz(0.0, 150.0, -200.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);

    transform.rotate_around(
        Vec3::new(0.0, 0.0, 0.0),
        Quat::from_axis_angle(Vec3::Y, 90.0f32.to_radians()),
    );

    commands.spawn((Camera3d::default(), transform));
}

fn camera_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    _map_size: Res<MapSize>,
) {
    let Ok(mut camera) = camera_q.single_mut() else {
        return;
    };

    let mut speed = 300.0 * time.delta_secs();

    let forward = camera.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize();
    let right_xz = Vec3::new(forward.z, 0.0, -forward.x).normalize();

    if keyboard.pressed(KeyCode::ShiftLeft) {
        speed *= 2.0;
    }

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

    const SENS: f32 = 15.0;
    const MIN_Y: f32 = 50.0;
    const MAX_Y: f32 = 500.0;
    const MIN_ANGLE: f32 = 20.0;
    const MAX_ANGLE: f32 = 70.0;
    for event in scroll_events.read() {
        let zoom_amount = event.y * SENS;

        let current_height = camera.translation.y;
        let new_height = (current_height - zoom_amount).clamp(MIN_Y, MAX_Y);
        camera.translation.y = new_height;

        let zoom_factor = (new_height - MIN_Y) / (MAX_Y - MIN_Y);

        let rotation_degrees = MIN_ANGLE + (zoom_factor * (MAX_ANGLE - MIN_ANGLE));
        let rotation_radians = rotation_degrees.to_radians();

        let desired_pitch = -rotation_radians;
        let (yaw, _current_pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, desired_pitch, roll);
    }
}
