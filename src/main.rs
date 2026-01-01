use bevy::prelude::*;
use rust_game::app_state::AppState;
use rust_game::camera::CameraControls;
use rust_game::game_world_entity::GameWorldEntity;
use rust_game::province::Province;
use rust_game::terrain_type::TerrainType;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::InGame), setup_new_map)
        .add_systems(OnExit(AppState::InGame), clear_game_entities)
        .add_plugins(CameraControls)
        .add_systems(Startup, setup)
        .add_systems(Update, menu_input)
        .run();
}

fn menu_input(mut next_state: ResMut<NextState<AppState>>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::InGame);
    }
}

fn setup_new_map() {
    //todo!()
}

fn clear_game_entities(mut commands: Commands, query: Query<Entity, With<GameWorldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera at angle
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 300.0, 400.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    //commands.insert_resource(AmbientLight {
    //    color: Color::WHITE,
    //    brightness: 5000.0,
    //    affects_lightmapped_meshes: true,
    //});

    // Test 1: Simple cube at origin (RED)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(50.0, 50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0), // RED
            ..default()
        })),
        Transform::from_xyz(0.0, 25.0, 0.0),
        GameWorldEntity,
    ));

    // Test 2: Green cube to the right
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(50.0, 50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0), // GREEN
            ..default()
        })),
        Transform::from_xyz(100.0, 25.0, 0.0),
        GameWorldEntity,
    ));

    // Test 3: Blue cube to the left
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(50.0, 50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 1.0), // BLUE
            ..default()
        })),
        Transform::from_xyz(-100.0, 25.0, 0.0),
        GameWorldEntity,
    ));

    // Test 4: Yellow sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(30.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0), // YELLOW
            ..default()
        })),
        Transform::from_xyz(0.0, 30.0, 100.0),
        GameWorldEntity,
    ));

    // Define province centers and types
    let province_data = [
        (Vec3::new(0.0, 0.0, 0.0), TerrainType::City),
        (Vec3::new(100.0, 0.0, 50.0), TerrainType::Plains),
        (Vec3::new(-100.0, 0.0, 50.0), TerrainType::Forest),
        (Vec3::new(50.0, 0.0, 150.0), TerrainType::Mountains),
        (Vec3::new(-50.0, 0.0, 150.0), TerrainType::Plains),
        (Vec3::new(150.0, 0.0, -50.0), TerrainType::Forest),
        (Vec3::new(-150.0, 0.0, -50.0), TerrainType::Plains),
        (Vec3::new(0.0, 0.0, -150.0), TerrainType::Mountains),
        (Vec3::new(100.0, 0.0, -150.0), TerrainType::Plains),
        (Vec3::new(-100.0, 0.0, -150.0), TerrainType::Forest),
    ];

    // Create provinces as hexagons
    for (i, (center, terrain)) in province_data.iter().enumerate() {
        let radius = 60.0;
        let hex_height = 0.5;

        // Create hexagonal mesh
        let mesh = create_hexagon_mesh(*center, radius, hex_height);

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: terrain.color(),
                unlit: true,
                ..default()
            })),
            Province {
                id: i as u32,
                terrain: *terrain,
                x: center.x,
                z: center.z,
            },
            GameWorldEntity,
        ));
    }
}

fn create_hexagon_mesh(center: Vec3, radius: f32, height: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Top face vertices
    // Center vertex
    positions.push([center.x, height, center.z]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    // Hexagon outer vertices (top face)
    for j in 0..6 {
        let angle = (j as f32) * std::f32::consts::PI / 3.0;
        let x = center.x + radius * angle.cos();
        let z = center.z + radius * angle.sin();
        positions.push([x, height, z]);
        normals.push([0.0, 1.0, 0.0]);

        let u = 0.5 + 0.5 * angle.cos();
        let v = 0.5 + 0.5 * angle.sin();
        uvs.push([u, v]);
    }

    // Create triangles for top face (7 vertices: 0 = center, 1-6 = outer)
    for j in 0..6 {
        indices.push(0);
        indices.push(if j == 5 { 1 } else { j + 2 });
        indices.push(j + 1);
    }

    // In Bevy 0.17, use Mesh::new() which returns the mesh directly
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    //.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}
