fn setup_new_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
                center: Vec2 {
                    x: center.x,
                    y: center.z,
                },
                polygon: default(),
            },
            GameWorldEntity,
        ));
    }
}

fn create_hexagon_mesh(center: Vec3, radius: f32, height: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    positions.push([center.x, height, center.z]);

    for j in 0..6 {
        let angle = (j as f32) * std::f32::consts::PI / 3.0;
        let x = center.x + radius * angle.cos();
        let z = center.z + radius * angle.sin();
        positions.push([x, height, z]);
    }

    for j in 0..6 {
        indices.push(0);
        indices.push(if j == 5 { 1 } else { j + 2 });
        indices.push(j + 1);
    }

    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}
