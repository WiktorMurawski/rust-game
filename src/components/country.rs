use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Country {
    pub id: u32,
    pub name: String,
    pub color: Color,
    pub owned_provinces: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryDef {
    pub id: u32,
    pub name: String,
    #[serde(with = "color_def")]
    pub color: Color,
    pub owned_provinces: Vec<u32>,
}

mod color_def {
    use bevy::prelude::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rgba = color.to_srgba();
        (rgba.red, rgba.green, rgba.blue).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (r, g, b) = <(f32, f32, f32)>::deserialize(deserializer)?;
        Ok(Color::srgb(r, g, b))
    }
}
