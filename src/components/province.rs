use bevy::{platform::collections::HashSet, prelude::*};
use serde::Deserialize;
use serde::Serialize;

#[derive(Component)]
pub struct Province {
    pub id: u32,
    pub center: Vec2,
    pub terrain: TerrainType,
    pub polygon: Vec<Vec2>,
    pub neighbors: HashSet<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProvinceDef {
    pub id: u32,
    //pub center: Vec2,
    pub center: (f32, f32),
    pub terrain: TerrainType,
}

#[derive(Component)]
pub struct ProvinceBorder {
    pub province_id: u32,
}

#[derive(Component)]
pub struct OwnedBy(pub Entity);

#[derive(Component)]
pub struct Occupied {
    //pub original_owner: Entity,
    pub occupier: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum TerrainType {
    Water,
    Plains,
    Forest,
    Mountains,
    City,
}

impl TerrainType {
    pub fn color(&self) -> Color {
        match self {
            TerrainType::Plains => Color::srgb(0.4, 0.8, 0.3),
            TerrainType::Forest => Color::srgb(0.2, 0.5, 0.2),
            TerrainType::Mountains => Color::srgb(0.5, 0.5, 0.5),
            TerrainType::City => Color::srgb(0.7, 0.7, 0.8),
            TerrainType::Water => Color::srgb(0.0, 0.0, 1.0),
        }
    }
}
