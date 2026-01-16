// plugins/save_load.rs
use crate::components::army::*;
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
    pub armies: Vec<ArmySaveData>,
    pub occupied_provinces: Vec<OccupiedData>,
    pub player_country_id: Option<u32>,
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
    pub relations: HashMap<u32, Relation>,
}

#[derive(Serialize, Deserialize)]
pub struct ArmySaveData {
    pub owner_id: u32,
    pub province_id: u32,
    pub units: u32,
}

#[derive(Serialize, Deserialize)]
pub struct OccupiedData {
    pub province_id: u32,
    pub occupier_id: u32,
}

#[derive(Resource, Default)]
pub struct SaveLoadError {
    pub message: Option<String>,
}

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
    let country_defs = match load_countries_from_file() {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };

    let mut country_entities = HashMap::new();
    for country_def in &country_defs {
        let flag = country_def
            .flag_path
            .as_ref()
            .map(|path| asset_server.load(path.clone()));

        let mut builder = commands.spawn((
            Country {
                id: country_def.id,
                name: country_def.name.clone(),
                color: country_def.color,
                owned_provinces: country_def.owned_provinces.clone(),
                gold: country_def.gold,
                flag,
                flag_path: country_def.flag_path.clone(),
            },
            Relations::default(),
        ));

        builder.insert(AIControlled);

        let country_entity = builder.id();

        country_entities.insert(country_def.id, country_entity);
    }

    for country_def in &country_defs {
        if let Some(&country_entity) = country_entities.get(&country_def.id) {
            for &province_id in &country_def.owned_provinces {
                if let Some(&province_entity) = province_map.0.get(&province_id) {
                    commands.entity(province_entity).insert(OwnedBy {
                        owner: country_entity,
                    });
                }
            }
        }
    }

    next_state.set(AppState::CountrySelection);
}

fn load_saved_game(
    mut commands: Commands,
    province_map: Res<ProvinceEntityMap>,
    provinces: Query<&Province>,
    save_file_path: Res<SaveFilePath>,
    mut next_state: ResMut<NextState<AppState>>,
    mut error: ResMut<SaveLoadError>,
    asset_server: Res<AssetServer>,
) {
    match load_and_apply_save(
        &mut commands,
        &province_map,
        &provinces,
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
    provinces: &Query<&Province>,
    asset_server: Res<AssetServer>,
    path: &str,
) -> Result<()> {
    let save_data = load_save_file(path)?;

    // First pass: create all countries
    let mut country_entities = HashMap::new();
    for country_data in &save_data.countries {
        let flag = country_data
            .flag_path
            .as_ref()
            .map(|path| asset_server.load(path.clone()));

        let mut builder = commands.spawn((
            Country {
                id: country_data.id,
                name: country_data.name.clone(),
                color: country_data.color,
                owned_provinces: country_data.owned_provinces.clone(),
                gold: country_data.gold,
                flag,
                flag_path: country_data.flag_path.clone(),
            },
            Relations::default(),
        ));

        if Some(country_data.id) != save_data.player_country_id {
            builder.insert(AIControlled);
        }

        let country_entity = builder.id();
        country_entities.insert(country_data.id, country_entity);
    }

    // Second pass: set up relations between countries
    for country_data in &save_data.countries {
        let country_entity = *country_entities
            .get(&country_data.id)
            .context("Country entity not found")?;

        let mut relations = Relations::default();
        for (&other_id, &relation_data) in &country_data.relations {
            if let Some(&other_entity) = country_entities.get(&other_id) {
                relations.set(other_entity, relation_data);
            }
        }

        commands.entity(country_entity).insert(relations);
    }

    // Third pass: assign provinces to countries
    for country_data in &save_data.countries {
        let country_entity = *country_entities
            .get(&country_data.id)
            .context("Country entity not found")?;

        for &province_id in &country_data.owned_provinces {
            let province_entity = *province_map
                .0
                .get(&province_id)
                .with_context(|| format!("Province {} not found in map", province_id))?;

            commands.entity(province_entity).insert(OwnedBy {
                owner: country_entity,
            });
        }
    }

    // Fourth pass: create armies
    for army_data in &save_data.armies {
        let owner_entity = *country_entities
            .get(&army_data.owner_id)
            .with_context(|| format!("Army owner {} not found", army_data.owner_id))?;

        let province_entity = *province_map
            .0
            .get(&army_data.province_id)
            .with_context(|| format!("Army province {} not found", army_data.province_id))?;

        let province = provinces
            .get(province_entity)
            .with_context(|| "Province component not found for entity".to_string())?;

        commands.spawn((
            Army {
                owner: owner_entity,
                province: province_entity,
                units: army_data.units,
            },
            Transform::from_xyz(province.center.x, 0.0, province.center.y),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
    }

    // Fifth pass: restore occupied provinces
    for occupied_data in &save_data.occupied_provinces {
        let province_entity = *province_map
            .0
            .get(&occupied_data.province_id)
            .with_context(|| format!("Occupied province {} not found", occupied_data.province_id))?;

        let occupier_entity = *country_entities
            .get(&occupied_data.occupier_id)
            .with_context(|| format!("Occupier {} not found", occupied_data.occupier_id))?;

        commands.entity(province_entity).insert(Occupied {
            occupier: occupier_entity,
        });
    }

    // Finally: set up player
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
    countries: Query<(&Country, &Relations)>,
    armies: Query<&Army>,
    provinces: Query<(&Province, Option<&OwnedBy>)>,
    occupied_provinces: Query<(&Province, &Occupied)>,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
) {
    if keyboard.just_pressed(KeyCode::F5)
        && let Err(e) = save_game(
            countries,
            armies,
            provinces,
            occupied_provinces,
            local_player,
            player_query,
            "saves/quicksave.ron",
        ) {
            eprintln!("Failed to save game: {:?}", e);
        }
}


pub fn save_game(
    countries: Query<(&Country, &Relations)>,
    armies: Query<&Army>,
    provinces: Query<(&Province, Option<&OwnedBy>)>,
    occupied_provinces: Query<(&Province, &Occupied)>,
    local_player: Option<Res<crate::components::player::LocalPlayer>>,
    player_query: Query<&crate::components::player::ControlsCountry>,
    path: &str,
) -> Result<()> {
    let mut country_data = Vec::new();

    for (country, relations) in countries.iter() {
        let owned_provinces: Vec<u32> = provinces
            .iter()
            .filter_map(|(province, owner)| {
                owner.and_then(|o| {
                    if countries.get(o.owner).ok()?.0.id == country.id {
                        Some(province.id)
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Convert entity-based relations to ID-based relations
        let mut relation_map = HashMap::new();
        for (other_entity, relation) in &relations.relations {
            if let Ok((other_country, _)) = countries.get(*other_entity) {
                relation_map.insert(other_country.id, *relation);
            }
        }

        country_data.push(CountrySaveData {
            id: country.id,
            name: country.name.clone(),
            color: country.color,
            owned_provinces,
            gold: country.gold,
            flag_path: country.flag_path.clone(),
            relations: relation_map,
        });
    }

    // Save armies
    let mut army_data = Vec::new();
    for army in armies.iter() {
        if let (Ok((owner_country, _)), Ok((province, _))) =
            (countries.get(army.owner), provinces.get(army.province))
        {
            army_data.push(ArmySaveData {
                owner_id: owner_country.id,
                province_id: province.id,
                units: army.units,
            });
        }
    }

    // Save occupied provinces
    let mut occupied_data = Vec::new();
    for (province, occupied) in occupied_provinces.iter() {
        if let Ok((occupier_country, _)) = countries.get(occupied.occupier) {
            occupied_data.push(OccupiedData {
                province_id: province.id,
                occupier_id: occupier_country.id,
            });
        }
    }

    let player_country_id = local_player.and_then(|lp| {
        player_query
            .get(lp.0)
            .ok()
            .and_then(|controls| countries.get(controls.0).ok())
            .map(|(country, _)| country.id)
    });

    let save_data = SaveData {
        countries: country_data,
        armies: army_data,
        occupied_provinces: occupied_data,
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

fn load_save_file(path: &str) -> Result<SaveData> {
    let file = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read save file at '{}'", path))?;

    ron::from_str(&file).context("Save file is corrupted or invalid")
}

fn load_countries_from_file() -> Result<Vec<CountryDef>> {
    let file = std::fs::read_to_string("assets/data/countries.ron")?;

    ron::from_str(&file).context("Failed to parse countries.ron")
}