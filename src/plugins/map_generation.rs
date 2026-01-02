use crate::components::province::{Province, ProvinceDef, TerrainType};
use crate::components::GameWorldEntity;
use crate::resources::MapSize;
use bevy::prelude::*;
use earcutr::earcut;
use voronoice::Point;

fn build_voronoi(centers: &[Vec2], map_size: Vec2) -> Option<voronoice::Voronoi> {
    let sites: Vec<Point> = centers
        .iter()
        .map(|p| Point {
            x: p.x as f64,
            y: p.y as f64,
        })
        .collect();

    println!("sites generated:");
    for s in &sites {
        println!("{:?}", s);
    }

    voronoice::VoronoiBuilder::default()
        .set_sites(sites)
        .set_bounding_box(voronoice::BoundingBox::new(
            Point { x: 0.0, y: 0.0 },
            map_size.x as f64,
            map_size.y as f64,
        ))
        .build()
}

fn extract_polygons(diagram: &voronoice::Voronoi) -> Vec<Vec<Vec2>> {
    diagram
        .cells()
        .iter()
        .map(|cell| {
            cell.iter()
                .filter_map(|&i| diagram.vertices().get(i))
                .map(|p| Vec2::new(p.x as f32, p.y as f32))
                .collect()
        })
        .collect()
}

fn polygon_to_mesh(polygon: &[Vec2]) -> Mesh {
    // Flatten vertices for earcut
    let flattened: Vec<f64> = polygon
        .iter()
        .flat_map(|v| [v.x as f64, v.y as f64])
        .collect();

    let indices = earcut(&flattened, &[], 2).expect("Triangulation failed");

    let positions: Vec<[f32; 3]> = polygon.iter().map(|v| [v.x, 0.0, v.y]).collect();
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = polygon.iter().map(|v| [v.x, v.y]).collect();

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(
        indices.into_iter().map(|i| i as u32).collect(),
    ));
    mesh
}

//fn polygons_to_meshes(polygons: &[Vec<Vec2>]) -> Vec<Mesh> {
//    polygons.iter().map(|p| polygon_to_mesh(p)).collect()
//}

pub fn provinces_to_meshes(provinces: &[Province]) -> Vec<Mesh> {
    provinces
        .iter()
        .map(|prov| polygon_to_mesh(&prov.polygon))
        .collect()
}

pub fn generate_provinces(province_defs: &[ProvinceDef], map_size: Vec2) -> Vec<Province> {
    let province_centers: Vec<Vec2> = province_defs
        .iter()
        .map(|p| Vec2::new(p.center.x, p.center.y))
        .collect();

    let voronoi_diagram =
        build_voronoi(&province_centers, map_size).expect("Failed to build voronoi");
    let polygons = extract_polygons(&voronoi_diagram);

    province_defs
        .iter()
        .zip(polygons)
        .map(|(prov_def, poly)| Province {
            id: prov_def.id,
            center: prov_def.center,
            terrain: prov_def.terrain,
            polygon: poly,
        })
        .collect()
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

pub fn setup_map(
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

    let provinces = generate_provinces(&province_defs, map_size);
    println!("PROVINCES GENERATED");
    let province_meshes = provinces_to_meshes(&provinces);
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
