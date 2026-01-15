// components/player.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub id: u32,
    pub name: String,
}

#[derive(Component)]
pub struct ControlsCountry(pub Entity);

#[derive(Resource)]
pub struct LocalPlayer(pub Entity);
