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
            BuildingType::Farm => "Increases food production",
            BuildingType::Mine => "Increases mineral output",
            BuildingType::Barracks => "Allows recruiting troops",
        }
    }
}

// All possible buildings (for UI listing)
pub const ALL_BUILDINGS: [BuildingType; 3] = [
    BuildingType::Farm,
    BuildingType::Mine,
    BuildingType::Barracks,
];
