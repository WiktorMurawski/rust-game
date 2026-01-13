use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(SystemParam)]
pub struct MouseAndWindowAndCamera<'w, 's> {
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    pub camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}
