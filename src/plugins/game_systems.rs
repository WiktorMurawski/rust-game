use crate::components::GameWorldEntity;
use crate::map_generation;
use crate::province::ProvinceDef;
use crate::states::AppState;
use crate::terrain_type::TerrainType;
use bevy::prelude::*;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, menu_input)
            .add_systems(OnEnter(AppState::InGame), setup_map)
            .add_systems(OnEnter(AppState::InGame), setup_camera)
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

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 300.0, 400.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        GameWorldEntity,
    ));
}

fn setup_map(mut commands: Commands) {
    let province_defs = vec![
        ProvinceDef {
            id: 101,
            center: Vec2::new(50.0, 50.0),
            terrain: TerrainType::City,
        },
        ProvinceDef {
            id: 102,
            center: Vec2::new(-50.0, 50.0),
            terrain: TerrainType::Plains,
        },
        ProvinceDef {
            id: 103,
            center: Vec2::new(-50.0, -50.0),
            terrain: TerrainType::Forest,
        },
        ProvinceDef {
            id: 104,
            center: Vec2::new(50.0, 50.0),
            terrain: TerrainType::Mountains,
        },
    ];

    let map_width = 200.0;
    let map_height = 200.0;
    let map_size = Vec2::new(map_width, map_height);

    let provinces = map_generation::generate_provinces(&province_defs, map_size);
    let meshes = map_generation::provinces_to_meshes(&provinces);
}
