use crate::components::GameWorldEntity;
use crate::plugins::map_generation::setup_map;
use crate::states::AppState;
use bevy::prelude::*;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, menu_input)
            .add_systems(OnEnter(AppState::InGame), setup_map)
            .add_systems(OnExit(AppState::InGame), clear_game_entities);
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
