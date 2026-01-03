use crate::components::province::Province;
use crate::components::GameWorldEntity;
use crate::plugins::map_generation::setup_map;
use crate::plugins::{ProvinceVisuals, SelectionPlugin};
use crate::states::AppState;
use bevy::prelude::*;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, menu_input)
            .add_systems(OnEnter(AppState::InGame), setup_map)
            .add_systems(OnExit(AppState::InGame), clear_game_entities)
            .add_systems(
                Update,
                print_provinces_on_p.run_if(in_state(AppState::InGame)),
            )
            .add_plugins(ProvinceVisuals)
            .add_plugins(SelectionPlugin);
    }
}

fn setup() {}

fn clear_game_entities(mut commands: Commands, query: Query<Entity, With<GameWorldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn menu_input(mut next_state: ResMut<NextState<AppState>>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::InGame);
        println!("SETTING APPSTATE TO INGAME");
    }
}

//pub fn toggle_map_mode(keyboard: Res<ButtonInput<KeyCode>>, mut map_mode: ResMut<MapMode>) {
//    if keyboard.just_pressed(KeyCode::KeyM) {
//        *map_mode = match *map_mode {
//            MapMode::Terrain => MapMode::Political,
//            MapMode::Political => MapMode::Terrain,
//        };
//        println!("Map mode switched to: {:?}", *map_mode);
//    }
//}

fn print_provinces_on_p(keyboard: Res<ButtonInput<KeyCode>>, query: Query<&Province>) {
    // Only react on just-pressed (not held)
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

    println!("\n══════════════════════════════════════════");
    println!("         PROVINCE NEIGHBORS (P pressed)      ");
    println!("══════════════════════════════════════════");

    let mut provinces: Vec<&Province> = query.iter().collect();
    // Optional: sort by ID so output is stable/readable
    provinces.sort_by_key(|p| p.id);

    for province in provinces {
        let mut neighbor_ids: Vec<u32> = province.neighbors.iter().copied().collect();
        neighbor_ids.sort(); // nice-to-have: sorted output

        let neighbor_list = if neighbor_ids.is_empty() {
            "(none)".to_string()
        } else {
            neighbor_ids
                .iter()
                .map(|&id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "Province {:4} → neighbors: [{}]",
            province.id, neighbor_list
        );
    }

    println!("Total provinces: {}\n", query.iter().count());
}
