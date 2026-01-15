use bevy::prelude::*;

use crate::components::province::Province;
use crate::plugins::map_generation::MapGenerated;
use crate::plugins::terrain_visual_3d;
use crate::states::AppState;

pub struct Terrain3DVisualsPlugin;

impl Plugin for Terrain3DVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            terrain_visual_3d::spawn_3d_objects.after(MapGenerated),
        );
    }
}

const SCALE: f32 = 5.0;

pub fn spawn_3d_objects(
    mut commands: Commands,
    query: Query<(Entity, &Province)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // println!("spawn_3d_objects");

    for (entity, province) in query {
        match province.terrain {
            crate::components::province::TerrainType::Water => (),
            crate::components::province::TerrainType::Plains => (),
            crate::components::province::TerrainType::Forest => spawn_forest(
                province,
                &entity,
                &mut commands,
                &mut meshes,
                &mut materials,
            ),
            crate::components::province::TerrainType::Mountains => spawn_mountains(
                province,
                &entity,
                &mut commands,
                &mut meshes,
                &mut materials,
            ),
            crate::components::province::TerrainType::City => spawn_city(
                province,
                &entity,
                &mut commands,
                &mut meshes,
                &mut materials,
            ),
        }
    }
}

fn spawn_city(
    province: &Province,
    entity: &Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let (x, z) = (province.center.x, province.center.y);

    let child = commands
        .spawn((
            // Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
            // Mesh3d(meshes.add(Extrusion::new(Annulus::new(0.5, 0.8), 0.5))),
            Mesh3d(meshes.add(Extrusion::new(Circle::new(0.8), 0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(70, 50, 40),
                cull_mode: None,
                ..default()
            })),
            Transform::from_xyz(x, 0.01, z)
                .with_scale(Vec3::ONE * SCALE)
                .with_rotation(Quat::from_rotation_x(90f32.to_radians())),
        ))
        .id();

    commands.entity(child).insert(ChildOf(*entity));
}

fn spawn_mountains(
    province: &Province,
    entity: &Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let (x, z) = (province.center.x, province.center.y);

    let child = commands
        .spawn((
            Mesh3d(meshes.add(ConicalFrustum {
                radius_top: 0.5,
                radius_bottom: 2.5,
                height: 1.5,
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(48, 48, 48),
                cull_mode: None,
                ..default()
            })),
            Transform::from_xyz(x, 0.01, z).with_scale(Vec3::ONE * SCALE),
        ))
        .id();

    commands.entity(child).insert(ChildOf(*entity));
}

fn spawn_forest(
    province: &Province,
    entity: &Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let (x, z) = (province.center.x, province.center.y);

    let child1 = commands
        .spawn((
            Mesh3d(meshes.add(Cone::new(0.5, 1.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(40, 120, 40),
                cull_mode: None,
                ..default()
            })),
            Transform::from_xyz(x, 1.00 * SCALE, z).with_scale(Vec3::ONE * SCALE),
        ))
        .id();
    let child2 = commands
        .spawn((
            Mesh3d(meshes.add(Extrusion::new(Circle::new(0.2), 0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(70, 50, 40),
                cull_mode: None,
                ..default()
            })),
            Transform::from_xyz(x, 0.01, z)
                .with_scale(Vec3::ONE * SCALE)
                .with_rotation(Quat::from_rotation_x(90f32.to_radians())),
        ))
        .id();

    commands.entity(child1).insert(ChildOf(*entity));
    commands.entity(child2).insert(ChildOf(*entity));
}
