use crate::terrain_type::TerrainType;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Province {
    pub id: u32,
    pub terrain: TerrainType,
    pub x: f32,
    pub z: f32,
}
