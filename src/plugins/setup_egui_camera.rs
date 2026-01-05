use bevy::prelude::*;
use bevy_egui::PrimaryEguiContext;

use crate::{plugins::camera_controls::CameraSetup, states::AppState};

pub struct SetupEguiCamera;

impl Plugin for SetupEguiCamera {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            setup_egui_camera.after(CameraSetup),
        );
    }
}

fn setup_egui_camera(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera3d>, Without<PrimaryEguiContext>)>,
) {
    for camera_entity in cameras.iter() {
        commands.entity(camera_entity).insert(PrimaryEguiContext);
        println!("Added PrimaryEguiContext to camera");
    }
}
