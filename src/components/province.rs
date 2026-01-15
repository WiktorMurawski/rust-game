// components/province.rs
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

    pub population: u32,
    pub base_growth: f32,
    pub base_income: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProvinceDef {
    pub id: u32,
    pub center: (f32, f32),
    pub terrain: TerrainType,

    pub population: u32,
    pub base_growth: f32,
    pub base_income: u32,
}

#[derive(Component)]
pub struct ProvinceBorder {
    pub province_id: u32,
}

#[derive(Component)]
pub struct OwnedBy {
    pub owner: Entity,
}

impl OwnedBy {
    pub fn owner(&self) -> Entity {
        self.owner
    }
}

#[derive(Component)]
pub struct Occupied {
    pub occupier: Entity,
}

impl Occupied {
    pub fn occupier(&self) -> Entity {
        self.occupier
    }
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
