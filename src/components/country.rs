use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Component)]
pub struct Country {
    pub id: u32,
    pub name: String,
    pub color: Color,
    pub owned_provinces: Vec<u32>,
    pub gold: u32,
    pub flag: Option<Handle<Image>>,
    pub flag_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryDef {
    pub id: u32,
    pub name: String,
    #[serde(with = "color_def")]
    pub color: Color,
    pub gold: u32,
    pub owned_provinces: Vec<u32>,
    pub flag_path: Option<String>,
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

#[derive(Component, Default, Clone, Serialize, Deserialize)]
pub struct Relations {
    pub relations: HashMap<Entity, Relation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Relation {
    Peace,
    War,
}

impl Relations {
    pub fn get(&self, country: Entity) -> Relation {
        *self.relations.get(&country).unwrap_or(&Relation::Peace)
    }

    pub fn set(&mut self, country: Entity, relation: Relation) {
        self.relations.insert(country, relation);
    }
}

#[derive(Event, Message)]
pub struct DiplomacyChanged {
    pub declarer: Entity,
    pub target: Entity,
    pub new_relation: Relation,
}

#[derive(Component)]
pub struct AIControlled;
