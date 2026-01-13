use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct Army {
    pub owner: Entity,      // The country entity that owns this army
    pub province: Entity,   // Current province entity (for position/ownership checks)
    pub units: u32,         // Simple count (e.g., 100 soldiers)
}

// Marker for selected army (we'll extend your SelectedEntity enum)
#[derive(Component)]
pub struct ArmySelected;  // Separate from province Selected, to allow both at once if needed