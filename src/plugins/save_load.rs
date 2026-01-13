// plugins/save_load.rs
use crate::components::country::*;
use crate::components::player::*;
use crate::components::province::*;
use crate::plugins::map_generation::{MapGenerated, ProvinceEntityMap};
use crate::states::AppState;
use anyhow::{Context, Result};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadError>()
            .add_systems(
                OnEnter(AppState::LoadingNewGame),
                initialize_new_game.after(MapGenerated),
            )
            .add_systems(
                OnEnter(AppState::LoadingSavedGame),
                load_saved_game.after(MapGenerated),
            )
            .add_systems(Update, save_game_on_key.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource)]
pub struct SaveFilePath(pub String);

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub countries: Vec<CountrySaveData>,
    pub player_country_id: Option<u32>,
    // Add more: armies, resources, etc.
}

#[derive(Serialize, Deserialize)]
pub struct CountrySaveData {
    pub id: u32,
    pub name: String,
    #[serde(with = "color_serde")]
    pub color: Color,
    pub gold: u32,
    pub owned_provinces: Vec<u32>,
    pub flag_path: Option<String>,
}

#[derive(Resource, Default)]
pub struct SaveLoadError {
    pub message: Option<String>,
}

// Helper module for Color serialization
mod color_serde {
    use bevy::prelude::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct ColorData {
        r: f32,
        g: f32,
        b: f32,
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let srgba = color.to_srgba();
        ColorData {
            r: srgba.red,
            g: srgba.green,
            b: srgba.blue,
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = ColorData::deserialize(deserializer)?;
        Ok(Color::srgb(data.r, data.g, data.b))
    }
}

fn initialize_new_game(
    mut commands: Commands,
    province_map: Res<ProvinceEntityMap>,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    let country_defs = load_countries_from_file();

    let mut country_entities = HashMap::new();
    for country_def in &country_defs {
        let flag = country_def
            .flag_path
            .as_ref()
            .map(|path| asset_server.load(path.clone()));

        let country_entity = commands
            .spawn(Country {
                id: country_def.id,
                name: country_def.name.clone(),
                color: country_def.color,
                owned_provinces: country_def.owned_provinces.clone(),
                gold: country_def.gold,
                flag,
                flag_path: country_def.flag_path.clone(),
            })
            .id();

        country_entities.insert(country_def.id, country_entity);
    }

    for country_def in &country_defs {
        if let Some(&country_entity) = country_entities.get(&country_def.id) {
            for &province_id in &country_def.owned_provinces {
                if let Some(&province_entity) = province_map.0.get(&province_id) {
                    commands
                        .entity(province_entity)
                        .insert(OwnedBy(country_entity));
                }
            }
        }
    }

    // Transition to country selection
    next_state.set(AppState::CountrySelection);
}

fn load_saved_game(
    mut commands: Commands,
    province_map: Res<ProvinceEntityMap>,
    save_file_path: Res<SaveFilePath>,
    mut next_state: ResMut<NextState<AppState>>,
    mut error: ResMut<SaveLoadError>,
    asset_server: Res<AssetServer>,
) {
    // Use match with anyhow's Result
    match load_and_apply_save(
        &mut commands,
        &province_map,
        asset_server,
        &save_file_path.0,
    ) {
        Ok(_) => {
            next_state.set(AppState::InGame);
            println!("Save loaded successfully!");
        }
        Err(e) => {
            error.message = Some(format!("{:#}", e));
            println!("Error loading save: {:#}", e);
            next_state.set(AppState::InMainMenu);
        }
    }
}

fn load_and_apply_save(
    commands: &mut Commands,
    province_map: &ProvinceEntityMap,
    asset_server: Res<AssetServer>,
    path: &str,
) -> Result<()> {
    let save_data = load_save_file(path)?;

    let mut country_entities = HashMap::new();
    for country_data in &save_data.countries {
        let flag = country_data
            .flag_path
            .as_ref()
            .map(|path| asset_server.load(path.clone()));

        let country_entity = commands
            .spawn(Country {
                id: country_data.id,
                name: country_data.name.clone(),
                color: country_data.color,
                owned_provinces: country_data.owned_provinces.clone(),
                gold: country_data.gold,
                flag,
                flag_path: country_data.flag_path.clone(),
            })
            .id();

        country_entities.insert(country_data.id, country_entity);
    }

    for country_data in &save_data.countries {
        let country_entity = *country_entities
            .get(&country_data.id)
            .context("Country entity not found")?;

        for &province_id in &country_data.owned_provinces {
            let province_entity = *province_map
                .0
                .get(&province_id)
                .with_context(|| format!("Province {} not found in map", province_id))?;

            commands
                .entity(province_entity)
                .insert(OwnedBy(country_entity));
        }
    }

    if let Some(saved_country_id) = save_data.player_country_id {
        if let Some(&country_entity) = country_entities.get(&saved_country_id) {
            let player_entity = commands
                .spawn((
                    Player {
                        id: 0,
                        name: "Player 1".to_string(),
                    },
                    ControlsCountry(country_entity),
                ))
                .id();

            commands.insert_resource(LocalPlayer(player_entity));
        } else {
            println!(
                "Warning: saved player country id {} not found in loaded countries",
                saved_country_id
            );
        }
    } else {
        println!("No player country was saved → starting without local player control");
    }

    Ok(())
}

fn save_game_on_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    countries: Query<&Country>,
    provinces: Query<(&Province, Option<&OwnedBy>)>,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
) {
    if keyboard.just_pressed(KeyCode::F5)
        && let Err(e) = save_game(
            countries,
            provinces,
            local_player,
            player_query,
            "saves/quicksave.ron",
        )
    {
        eprintln!("Failed to save game: {:?}", e);
    }
}

pub fn save_game(
    countries: Query<&Country>,
    provinces: Query<(&Province, Option<&OwnedBy>)>,
    local_player: Option<Res<crate::components::player::LocalPlayer>>,
    player_query: Query<&crate::components::player::ControlsCountry>,
    path: &str,
) -> Result<()> {
    let mut country_data = Vec::new();

    for country in countries.iter() {
        let owned_provinces: Vec<u32> = provinces
            .iter()
            .filter_map(|(province, owner)| {
                owner.and_then(|o| {
                    if countries.get(o.0).ok()?.id == country.id {
                        Some(province.id)
                    } else {
                        None
                    }
                })
            })
            .collect();

        country_data.push(CountrySaveData {
            id: country.id,
            name: country.name.clone(),
            color: country.color,
            owned_provinces,
            gold: country.gold,
            flag_path: country.flag_path.clone(),
        });
    }

    let player_country_id = local_player.and_then(|lp| {
        player_query
            .get(lp.0)
            .ok()
            .and_then(|controls| countries.get(controls.0).ok())
            .map(|country| country.id)
    });

    let save_data = SaveData {
        countries: country_data,
        player_country_id,
    };

    std::fs::create_dir_all("saves").context("Failed to create saves directory")?;

    let serialized = ron::ser::to_string_pretty(&save_data, Default::default())
        .context("Failed to serialize save data")?;

    std::fs::write(path, serialized)
        .with_context(|| format!("Failed to write save file to '{}'", path))?;

    println!("Game saved to {}", path);
    Ok(())
}

// Much cleaner function signatures!
fn load_save_file(path: &str) -> Result<SaveData> {
    let file = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read save file at '{}'", path))?;

    ron::from_str(&file).context("Save file is corrupted or invalid")
}

fn load_countries_from_file() -> Vec<CountryDef> {
    let file =
        std::fs::read_to_string("assets/data/countries.ron").expect("Failed to read countries.ron");
    ron::from_str(&file).expect("Failed to parse countries.ron")
}
