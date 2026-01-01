use bevy::prelude::Color;

#[derive(Clone, Copy)]
pub enum TerrainType {
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
        }
    }
}
