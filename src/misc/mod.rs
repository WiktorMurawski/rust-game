use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::components::player::{ControlsCountry, LocalPlayer};

#[derive(SystemParam)]
pub struct MouseAndWindowAndCamera<'w, 's> {
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    pub camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}

#[derive(SystemParam)]
pub struct CommandsAndContexts<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub contexts: EguiContexts<'w, 's>,
}

#[derive(SystemParam)]
pub struct PlayerParams<'w, 's> {
    pub local_player: Res<'w, LocalPlayer>,
    pub player_controls: Query<'w, 's, &'static ControlsCountry>,
}

pub fn squared_distance(a: Vec2, b: Vec2) -> f32 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}

pub fn mouse_to_world_coords(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    let (camera, cam_tf) = camera_query.single().ok()?;

    let cursor_pos = window.cursor_position()?;

    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else {
        return None;
    };

    let plane_origin = Vec3::ZERO;
    let plane_normal = Vec3::Y;

    let distance = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))?;

    if distance <= 0.0 {
        return None;
    }

    let point = ray.get_point(distance);

    Some(point.xz())
}

pub fn empty_function() {}
