use crate::province::{Province, ProvinceDef};
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
