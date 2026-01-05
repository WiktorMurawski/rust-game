use crate::components::GameWorldEntity;
use crate::plugins::map_generation::setup_map;
use crate::plugins::*;
use crate::states::AppState;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(Startup, setup)
            .add_systems(OnExit(AppState::InMainMenu), clear_cameras)
            .add_systems(OnEnter(AppState::InGame), setup_map)
            .add_systems(OnExit(AppState::InGame), clear_game_entities)
            .add_plugins(GameCamera)
            .add_plugins(ProvinceVisuals)
            .add_plugins(SelectionPlugin)
            .add_plugins(MainMenu)
            .add_plugins(SetupEguiCamera)
            .add_plugins(ProvinceInfoUI);
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

fn clear_game_entities(mut commands: Commands, query: Query<Entity, With<GameWorldEntity>>) {
    clear_entities(&mut commands, &query);
}

fn clear_cameras(
    mut commands: Commands,
    query2d: Query<Entity, With<Camera2d>>,
    query3d: Query<Entity, With<Camera3d>>,
) {
    clear_entities(&mut commands, &query2d);
    clear_entities(&mut commands, &query3d);
}
