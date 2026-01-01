use crate::terrain_type::TerrainType;
use bevy::prelude::*;

#[derive(Component)]
pub struct Province {
    pub id: u32,
    pub center: Vec2,
    pub polygon: Vec<Vec2>,
    pub terrain: TerrainType,
}

pub struct ProvinceDef {
    pub id: u32,
    pub center: Vec2,
    pub terrain: TerrainType,
}
