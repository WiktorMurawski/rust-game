use bevy::prelude::*;
use RUSTGAME::camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            camera::camera_keyboard_controls,
            camera::camera_scroll_controls,
        ))
        .run();
}

#[derive(Component)]
struct Province {
    id: u32,
    terrain: TerrainType,
}

#[derive(Clone, Copy)]
enum TerrainType {
    Plains,
    Forest,
    Mountains,
    City,
}

impl TerrainType {
    fn color(&self) -> Color {
        match self {
            TerrainType::Plains => Color::srgb(0.4, 0.8, 0.3),
            TerrainType::Forest => Color::srgb(0.2, 0.5, 0.2),
            TerrainType::Mountains => Color::srgb(0.5, 0.5, 0.5),
            TerrainType::City => Color::srgb(0.7, 0.7, 0.8),
        }
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
        Transform::from_xyz(0.0, 300.0, 400.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -0.3, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });

    // Define province centers and types
    let province_data = vec![
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

    // Create provinces as hexagons (simplified Voronoi)
    for (i, (center, terrain)) in province_data.iter().enumerate() {
        // Create hexagonal province
        let radius = 60.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();

        // Center vertex
        vertices.push([center.x, center.y, center.z]);
        normals.push([0.0, 1.0, 0.0]);

        // Hexagon vertices
        for j in 0..6 {
            let angle = (j as f32) * std::f32::consts::PI / 3.0;
            let x = center.x + radius * angle.cos();
            let z = center.z + radius * angle.sin();
            vertices.push([x, center.y, z]);
            normals.push([0.0, 1.0, 0.0]);
        }

        // Create triangles
        for j in 0..6 {
            indices.push(0);
            indices.push(j + 1);
            indices.push(if j == 5 { 1 } else { j + 2 });
        }

        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_indices(bevy::mesh::Indices::U32(indices));

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: terrain.color(),
                ..default()
            })),
            Province {
                id: i as u32,
                terrain: *terrain,
            },
        ));

        // Add border outline
        create_hexagon_border(
            &mut commands,
            &mut meshes,
            &mut materials,
            *center,
            radius,
        );
    }
}

fn create_hexagon_border(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3,
    radius: f32,
) {
    let border_color = Color::srgb(0.1, 0.1, 0.1);
    let border_height = 2.0;
    let border_thickness = 1.0;
    let y_offset = 0.5; // Slightly above ground

    // Create 6 edges of the hexagon
    for i in 0..6 {
        let angle1 = (i as f32) * std::f32::consts::PI / 3.0;
        let angle2 = ((i + 1) as f32) * std::f32::consts::PI / 3.0;

        // Calculate the two endpoints of this edge
        let p1 = Vec3::new(
            center.x + radius * angle1.cos(),
            y_offset,
            center.z + radius * angle1.sin(),
        );
        let p2 = Vec3::new(
            center.x + radius * angle2.cos(),
            y_offset,
            center.z + radius * angle2.sin(),
        );

        // Edge properties
        let edge_center = (p1 + p2) / 2.0;
        let edge_direction = (p2 - p1).normalize();
        let edge_length = p1.distance(p2);

        // Create rotation to align the cuboid along the edge
        // Cuboid is created along X-axis by default, so we need to rotate it
        // to point in the edge_direction
        let rotation = Quat::from_rotation_arc(Vec3::X, edge_direction);

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(edge_length, border_height, border_thickness))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: border_color,
                unlit: true,
                ..default()
            })),
            Transform {
                translation: edge_center,
                rotation,
                scale: Vec3::ONE,
            },
        ));
    }
}

