// components/buildings.rs
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Clone, Debug, Deserialize, Serialize)]
pub struct Buildings {
    pub built: Vec<BuildingType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum BuildingType {
    Farm,
    Mine,
    Barracks,
}

impl BuildingType {
    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::Farm => "Farm",
            BuildingType::Mine => "Mine",
            BuildingType::Barracks => "Barracks",
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            BuildingType::Farm => 100,
            BuildingType::Mine => 200,
            BuildingType::Barracks => 300,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BuildingType::Farm => "Increases population growth",
            BuildingType::Mine => "Increases province income",
            BuildingType::Barracks => "Allows recruiting troops",
        }
    }

    pub fn income_bonus(&self) -> u32 {
        match self {
            BuildingType::Farm => 0,
            BuildingType::Mine => 10,
            BuildingType::Barracks => 0,
        }
    }

    pub fn growth_bonus(&self) -> f32 {
        match self {
            BuildingType::Farm => 0.01,
            BuildingType::Mine => 0.0,
            BuildingType::Barracks => 0.0,
        }
    }

    pub fn population_bonus(&self) -> u32 {
        match self {
            BuildingType::Farm => 500,
            BuildingType::Mine => 0,
            BuildingType::Barracks => 0,
        }
    }
}

pub const ALL_BUILDINGS: [BuildingType; 3] = [
    BuildingType::Farm,
    BuildingType::Mine,
    BuildingType::Barracks,
];
