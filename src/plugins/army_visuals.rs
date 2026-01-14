use crate::components::army::Army;
use crate::states::AppState;
use bevy::prelude::*;
use bevy_rich_text3d::Text3d;

pub struct ArmyRendering;

impl Plugin for ArmyRendering {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (render_armies, billboard_text, update_army_labels).run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component)]
struct ArmyLabel;

#[derive(Component)]
struct ArmyModel;

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
    for mut tf in &mut query {
        let dir = tf.translation - camera.single().unwrap().translation;
        tf.look_at(dir.normalize_or_zero(), Vec3::Y);
    }
}

fn render_armies(
    mut commands: Commands,
    armies: Query<(Entity, &Army), Added<Army>>,
    // provinces: Query<&Province>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (army_entity, army) in &armies {
        commands.entity(army_entity).with_children(|parent| {
            parent.spawn((
                ArmyModel,
                Mesh3d(meshes.add(Cuboid::new(1.0, 5.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(70, 100, 40),
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
                    base_color_texture: Some(bevy_rich_text3d::TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(-8.0, 8.0, 0.0).with_scale(Vec3::splat(0.6)),
            ));
        });
    }
}
