// plugins/map_generation.rs
use crate::components::buildings::Buildings;
use crate::components::province::*;
use crate::resources::MapSize;
use crate::states::AppState;
use anyhow::Context;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use earcutr::earcut;
use voronoice::{Point, Voronoi};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapGenerated;

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingNewGame),
            load_map_geometry.in_set(MapGenerated),
        )
        .add_systems(
            OnEnter(AppState::LoadingSavedGame),
            load_map_geometry.in_set(MapGenerated),
        );
    }
}

#[derive(Resource)]
pub struct ProvinceEntityMap(pub HashMap<u32, Entity>);

fn load_map_geometry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map_data = match load_map_data_from_file() {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Couldn't load map data from file: {:?}", err);
            return;
        }
    };
    let map_size = Vec2::from(map_data.map_size);

    commands.insert_resource(MapSize(map_size));

    let provinces = generate_provinces(&map_data.provinces, map_size);
    let province_meshes = provinces_to_meshes(&provinces);

    let mut province_entities = HashMap::new();

    for (province, mesh) in provinces.into_iter().zip(province_meshes) {
        let color = province.terrain.color();
        let province_id = province.id;

        let material_handle = materials.add(StandardMaterial {
            base_color: color,
            cull_mode: None,
            perceptual_roughness: 0.8,
            ..default()
        });

        let border_mesh = polygon_to_border_mesh(&province.polygon, 1.0);

        let province_entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material_handle),
                province,
            ))
            .id();

        commands
            .entity(province_entity)
            .insert(Buildings::default());

        province_entities.insert(province_id, province_entity);

        let border_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            unlit: true,
            cull_mode: None,
            ..default()
        });

        commands
            .spawn((
                Mesh3d(meshes.add(border_mesh)),
                MeshMaterial3d(border_material),
                ProvinceBorder { province_id },
            ))
            .set_parent_in_place(province_entity);
    }

    commands.insert_resource(ProvinceEntityMap(province_entities));

    add_background_mesh(
        map_size.x,
        map_size.y,
        10.0,
        Color::srgb_u8(0x4e, 0x62, 0x9d),
        commands,
        meshes,
        materials,
    );
}

fn generate_provinces(province_defs: &[ProvinceDef], map_size: Vec2) -> Vec<Province> {
    let province_centers: Vec<Vec2> = province_defs
        .iter()
        .map(|p| Vec2::new(p.center.0, p.center.1))
        .collect();

    let voronoi_diagram = match build_voronoi(&province_centers, map_size) {
        Some(x) => x,
        None => {
            eprintln!("Couldn't build voronoi diagram");
            return Vec::new();
        }
    };
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
            neighbors: cell_neighbors[cell_idx]
                .iter()
                .map(|&neighbor_cell_idx| province_defs[neighbor_cell_idx].id)
                .collect(),
            population: prov_def.population,
            base_growth: prov_def.base_growth,
            base_income: prov_def.base_income,
        })
        .collect()
}

fn build_voronoi(centers: &[Vec2], map_size: Vec2) -> Option<Voronoi> {
    let sites: Vec<Point> = centers
        .iter()
        .map(|p| Point {
            x: p.x as f64,
            y: p.y as f64,
        })
        .collect();

    const PADDING: f32 = 1.01;

    voronoice::VoronoiBuilder::default()
        .set_sites(sites)
        .set_bounding_box(voronoice::BoundingBox::new(
            Point { x: 0.0, y: 0.0 },
            (map_size.x * PADDING) as f64,
            (map_size.y * PADDING) as f64,
        ))
        .build()
}

fn extract_polygons(diagram: &Voronoi) -> Vec<Vec<Vec2>> {
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

fn polygon_to_mesh(polygon: &[Vec2]) -> Mesh {
    // Flatten vertices for earcut
    let flattened: Vec<f64> = polygon
        .iter()
        .flat_map(|v| [v.x as f64, v.y as f64])
        .collect();

    let indices = earcut(&flattened, &[], 2).unwrap_or_default();

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

fn provinces_to_meshes(provinces: &[Province]) -> Vec<Mesh> {
    provinces
        .iter()
        .map(|prov| polygon_to_mesh(&prov.polygon))
        .collect()
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

    for i in 0..polygon.len() {
        let next_i = (i + 1) % polygon.len();
        let p1 = polygon[i];
        let p2 = polygon[next_i];

        let edge = p2 - p1;
        let edge_length = edge.length();

        if edge_length < 0.01 {
            continue;
        }

        let edge_normalized = edge / edge_length;
        let perpendicular = Vec2::new(-edge_normalized.y, edge_normalized.x);

        let v1 = p1; // Edge point
        let v2 = p1 - perpendicular * thickness; // Inward from edge
        let v3 = p2 - perpendicular * thickness; // Inward from edge
        let v4 = p2; // Edge point

        let base = positions.len() as u32;

        positions.push([v1.x, 0.01, v1.y]);
        positions.push([v2.x, 0.01, v2.y]);
        positions.push([v3.x, 0.01, v3.y]);
        positions.push([v4.x, 0.01, v4.y]);

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
    ));
}

fn load_map_data_from_file() -> anyhow::Result<MapData> {
    let file = std::fs::read_to_string("assets/data/map.ron")?;
    ron::from_str(&file).context("Failed to parse map.ron")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MapData {
    pub map_size: (f32, f32),
    pub provinces: Vec<ProvinceDef>,
}
