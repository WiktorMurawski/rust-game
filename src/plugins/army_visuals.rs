// plugins/army_visuals.rs
use crate::components::army::Army;
use crate::components::province::Province;
use crate::states::AppState;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, TextAtlas};
use crate::components::country::Country;

pub struct ArmyRendering;

impl Plugin for ArmyRendering {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                render_armies,
                billboard_text,
                update_army_labels,
                update_army_positions,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component)]
struct ArmyLabel;

#[derive(Component)]
struct ArmyModel;

fn update_army_positions(
    mut armies: Query<(&Army, &mut Transform), Changed<Army>>,
    provinces: Query<&Province>,
) {
    for (army, mut transform) in &mut armies {
        if let Ok(province) = provinces.get(army.province) {
            transform.translation = Vec3::new(province.center.x, 0.0, province.center.y);
        }
    }
}

fn update_army_labels(
    mut labels: Query<(&mut Text3d, &ChildOf), With<ArmyLabel>>,
    armies: Query<&Army, Changed<Army>>,
) {
    for (mut text, child_of) in &mut labels {
        if let Ok(army) = armies.get(child_of.parent()) {
            *text = Text3d::new(format!("{}", army.units));
        }
    }
}

fn billboard_text(
    mut query: Query<&mut Transform, With<Text3d>>,
    camera: Query<&Transform, (With<Camera>, Without<Text3d>)>,
) {
    if let Ok(camera_single) = camera.single() {
        for mut tf in &mut query {
            let dir = tf.translation - camera_single.translation;
            tf.look_at(dir.normalize_or_zero(), Vec3::Y);
        }
    }
}

fn render_armies(
    mut commands: Commands,
    armies: Query<(Entity, &Army), Added<Army>>,
    countries: Query<&Country>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (army_entity, army) in &armies {
        commands.entity(army_entity).with_children(|parent| {

            let country_color = countries
                .get(army.owner)
                .map(|country| country.color)
                .unwrap_or(Color::srgb(0.5, 0.5, 0.5));

            let model_color = match country_color {
                Color::Srgba(s) => Color::srgba(
                    (s.red   * 0.7).clamp(0.0, 1.0),
                    (s.green * 0.7).clamp(0.0, 1.0),
                    (s.blue  * 0.7).clamp(0.0, 1.0),
                    s.alpha,
                ),
                _ => country_color,
            };

            let label_color = match country_color {
                Color::Srgba(s) => Color::srgba(
                    (s.red   * 1.4).clamp(0.0, 1.0),
                    (s.green * 1.4).clamp(0.0, 1.0),
                    (s.blue  * 1.4).clamp(0.0, 1.0),
                    s.alpha,
                ),
                _ => country_color,
            };

            parent.spawn((
                ArmyModel,
                Mesh3d(meshes.add(Cuboid::new(1.0, 5.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: model_color,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_xyz(0.0, 2.5, 0.0).with_scale(Vec3::ONE * 5.0),
            ));

            parent.spawn((
                ArmyLabel,
                Text3d::new(format!("{}", army.units)),
                Mesh3d::default(),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: label_color,
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(-8.0, 8.0, 0.0).with_scale(Vec3::splat(0.6)),
            ));
        });
    }
}
