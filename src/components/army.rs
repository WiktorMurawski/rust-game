use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct Army {
    pub owner: Entity,
    pub province: Entity,
    pub units: u32,
}

impl Army {
    pub fn merge(&mut self, other: &Army) {
        self.units += other.units;
    }
}

#[derive(Component)]
pub struct ArmySelected;

#[derive(Component)]
pub struct ArmyUnitLabel {
    pub army: Entity,
}
