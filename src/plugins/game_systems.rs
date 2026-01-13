use crate::plugins::map_generation::{MapGenerated, MapGenerationPlugin};
use crate::plugins::*;
use crate::states::AppState;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(ClearColor(Color::srgb_u8(30, 30, 30)))
            .add_systems(Startup, setup)
            .add_systems(OnEnter(AppState::InGame), clear_cameras)
            .add_systems(
                OnEnter(AppState::InGame),
                terrain_visual_3d::spawn_3d_objects.after(MapGenerated),
            )
            .add_plugins(MapGenerationPlugin)
            .add_plugins(GameCamera)
            .add_plugins(Lighting)
            .add_plugins(ProvinceVisualsPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(SaveLoadPlugin)
            .add_plugins(ArmySystemsPlugin)
            // UI
            .add_plugins(SetupEguiCamera)
            .add_plugins(MainMenu)
            .add_plugins(CountrySelectionUI)
            .add_plugins(ProvinceInfoUI)
            .add_plugins(PlayerCountryUI)
            .add_plugins(BuildingsUI)
            .add_plugins(ArmyRendering);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn clear_entities<T: QueryFilter>(commands: &mut Commands, query: &Query<Entity, T>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

fn clear_cameras(
    mut commands: Commands,
    query2d: Query<Entity, With<Camera2d>>,
    query3d: Query<Entity, With<Camera3d>>,
) {
    clear_entities(&mut commands, &query2d);
    clear_entities(&mut commands, &query3d);
}

// fn clear_all(mut commands: Commands, query: Query<Entity>){
//
//     clear_entities(&mut commands, &query);
// }

// fn clear_game_entities(mut commands: Commands, query: Query<Entity, With<GameWorldEntity>>) {
//     clear_entities(&mut commands, &query);
// }
