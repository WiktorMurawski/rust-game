use crate::components::country::{Country, CountryDef};
use crate::components::province::{OwnedBy, Province, ProvinceBorder, ProvinceDef};
use crate::components::GameWorldEntity;
use crate::resources::MapSize;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use earcutr::earcut;
use serde::{Deserialize, Serialize};
use voronoice::{Point, Voronoi};

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

fn calculate_neighbors(voronoi_diagram: &Voronoi) -> Vec<HashSet<usize>> {
    let triangulation = voronoi_diagram.triangulation();
    let mut cell_neighbors: Vec<HashSet<usize>> =
        vec![HashSet::new(); voronoi_diagram.sites().len()];
    let tris = &triangulation.triangles;
    for i in (0..tris.len()).step_by(3) {
        let a = tris[i];
        let b = tris[i + 1];
        let c = tris[i + 2];

        // Now add the three neighbor relations
        cell_neighbors[a].insert(b);
        cell_neighbors[a].insert(c);
        cell_neighbors[b].insert(a);
        cell_neighbors[b].insert(c);
        cell_neighbors[c].insert(a);
        cell_neighbors[c].insert(b);
    }

    cell_neighbors
}

pub fn generate_provinces(province_defs: &[ProvinceDef], map_size: Vec2) -> Vec<Province> {
    let province_centers: Vec<Vec2> = province_defs
        .iter()
        .map(|p| Vec2::new(p.center.0, p.center.1))
        .collect();

    let voronoi_diagram =
        build_voronoi(&province_centers, map_size).expect("Failed to build voronoi");
    let polygons = extract_polygons(&voronoi_diagram);

    let cell_neighbors = calculate_neighbors(&voronoi_diagram);

    province_defs
        .iter()
        .zip(polygons)
        .enumerate()
        .map(|(cell_idx, (prov_def, poly))| Province {
            id: prov_def.id,
            center: prov_def.center.into(),
            terrain: prov_def.terrain,
            polygon: poly,
            // Convert cell indices → province IDs
            neighbors: cell_neighbors[cell_idx]
                .iter()
                .map(|&neighbor_cell_idx| province_defs[neighbor_cell_idx].id)
                .collect(),
        })
        .collect()
}

fn add_background_mesh(
    width: f32,
    height: f32,
    scale: f32,
    color: Color,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Rectangle::new(width, height);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
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

fn polygon_to_border_mesh(polygon: &[Vec2], thickness: f32) -> Mesh {
    if polygon.len() < 2 {
        return Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        );
    }

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Create thick lines by building quads for each edge
    for i in 0..polygon.len() {
        let next_i = (i + 1) % polygon.len();
        let p1 = polygon[i];
        let p2 = polygon[next_i];

        // Calculate edge
        let edge = p2 - p1;
        let edge_length = edge.length();

        // Skip degenerate edges (too short)
        if edge_length < 0.01 {
            continue;
        }

        // Now safe to normalize
        let edge_normalized = edge / edge_length;
        // Perpendicular pointing inward (we'll use negative to go inward)
        let perpendicular = Vec2::new(-edge_normalized.y, edge_normalized.x);

        // Four corners of the quad - border entirely on the INSIDE of the polygon
        // This prevents overlapping with neighbor borders
        let v1 = p1; // Edge point
        let v2 = p1 - perpendicular * thickness; // Inward from edge
        let v3 = p2 - perpendicular * thickness; // Inward from edge
        let v4 = p2; // Edge point

        let base = positions.len() as u32;

        // Add positions (elevated above provinces)
        positions.push([v1.x, 0.02, v1.y]);
        positions.push([v2.x, 0.02, v2.y]);
        positions.push([v3.x, 0.02, v3.y]);
        positions.push([v4.x, 0.02, v4.y]);

        // Two triangles for the quad
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

pub fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map_data = load_map_data_from_file();
    let country_defs = load_countries_from_file();

    let map_size: Vec2 = Vec2::from(map_data.map_size);
    let province_defs = map_data.provinces;
    commands.insert_resource(MapSize(map_size));

    let provinces = generate_provinces(&province_defs, map_size);
    println!("PROVINCES GENERATED");
    let province_meshes = provinces_to_meshes(&provinces);
    println!("PROVINCE MESHES GENERATED");

    // Spawn countries first and build a map of country_id -> Entity
    let mut country_entities: HashMap<u32, Entity> = HashMap::new();
    for country_def in &country_defs {
        let country_entity = commands
            .spawn(Country {
                id: country_def.id,
                name: country_def.name.clone(),
                color: country_def.color,
                owned_provinces: country_def.owned_provinces.clone(),
            })
            .id();

        country_entities.insert(country_def.id, country_entity);
        println!(
            "Spawned country: {} (ID: {})",
            country_def.name, country_def.id
        );
    }

    // Build ownership map: province_id -> country_entity
    let mut ownership_map: HashMap<u32, Entity> = HashMap::new();
    for country_def in &country_defs {
        if let Some(&country_entity) = country_entities.get(&country_def.id) {
            for &province_id in &country_def.owned_provinces {
                ownership_map.insert(province_id, country_entity);
            }
        }
    }

    for (province, mesh) in provinces.into_iter().zip(province_meshes) {
        let color = province.terrain.color();

        let material_handle = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            cull_mode: None,
            ..default()
        });

        let border_mesh = polygon_to_border_mesh(&province.polygon, 2.0);
        let province_id = province.id;

        // Spawn province
        let mut province_entity_commands = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material_handle),
            GameWorldEntity,
            province,
        ));
        let province_entity = province_entity_commands.id();

        // Add ownership if this province is owned
        if let Some(&owner_entity) = ownership_map.get(&province_id) {
            province_entity_commands.insert(OwnedBy(owner_entity));
            println!(
                "Province {} owned by country entity {:?}",
                province_id, owner_entity
            );
        }

        // Spawn province border
        let border_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2), // Dark gray
            unlit: true,
            cull_mode: None,
            ..default()
        });
        commands
            .spawn((
                Mesh3d(meshes.add(border_mesh)),
                MeshMaterial3d(border_material.clone()),
                GameWorldEntity,
                ProvinceBorder { province_id },
            ))
            .set_parent_in_place(province_entity);
    }

    println!("MAP SETUP DONE");

    add_background_mesh(
        map_size.x,
        map_size.y,
        10.0,
        Color::srgb_u8(0x4e, 0x62, 0x9d),
        commands,
        meshes,
        materials,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub map_size: (f32, f32),
    pub provinces: Vec<ProvinceDef>,
}

fn load_map_data_from_file() -> MapData {
    let file = std::fs::read_to_string("assets/data/map.ron").expect("Failed to read map.ron");
    ron::from_str(&file).expect("Failed to parse map.ron")
}

pub fn load_countries_from_file() -> Vec<CountryDef> {
    let file =
        std::fs::read_to_string("assets/data/countries.ron").expect("Failed to read countries.ron");
    ron::from_str(&file).expect("Failed to parse countries.ron")
}
