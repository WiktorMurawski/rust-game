use crate::components::province::ProvinceDef;
use crate::components::province::TerrainType;
use crate::components::GameWorldEntity;
use crate::plugins::map_generation;
use crate::resources::MapSize;
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

fn add_background_mesh(
    width: f32,
    height: f32,
    scale: f32,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Rectangle::new(width as f32, height as f32);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x4e, 0x62, 0x9d),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(0.0, -0.01, 0.0),
            rotation: Quat::from_rotation_x(90f32.to_radians()),
            scale: Vec3::ONE * scale,
        },
        GameWorldEntity,
    ));
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let province_defs = vec![
        ProvinceDef {
            id: 101,
            center: Vec2::new(50.0, 50.0),
            terrain: TerrainType::Water,
        },
        ProvinceDef {
            id: 102,
            center: Vec2::new(-50.0, 50.0),
            terrain: TerrainType::Plains,
        },
        ProvinceDef {
            id: 103,
            center: Vec2::new(-50.0, -50.0),
            //terrain: TerrainType::Forest,
            terrain: TerrainType::Plains,
        },
        ProvinceDef {
            id: 104,
            center: Vec2::new(50.0, -50.0),
            //terrain: TerrainType::Mountains,
            terrain: TerrainType::Plains,
        },
        ProvinceDef {
            id: 101,
            center: Vec2::new(90.0, 50.0),
            terrain: TerrainType::City,
        },
    ];

    let map_width = 200.0;
    let map_height = 200.0;
    let map_size = Vec2::new(map_width, map_height);
    commands.insert_resource(MapSize(map_size));

    let provinces = map_generation::generate_provinces(&province_defs, map_size);
    println!("PROVINCES GENERATED");
    let province_meshes = map_generation::provinces_to_meshes(&provinces);
    println!("PROVINCE MESHES GENERATED");

    for (province, mesh) in provinces.into_iter().zip(province_meshes) {
        let color = province.terrain.color();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                cull_mode: None,
                ..default()
            })),
            GameWorldEntity,
            province,
        ));
    }

    println!("MAP SETUP DONE");

    add_background_mesh(map_width, map_height, 10.0, commands, meshes, materials)
}
