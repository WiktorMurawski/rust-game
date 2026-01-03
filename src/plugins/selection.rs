use std::cmp::Ordering;

use bevy::prelude::Vec2;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{components::province::Province, states::AppState};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentSelection::default())
            .add_systems(
                Update,
                (update_selection).run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Resource, Default)]
struct CurrentSelection {
    selected: Option<SelectedEntity>,
}

enum SelectedEntity {
    Province(Entity),
    Army(Entity),
    Country(Entity),
}

fn update_selection(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    provinces: Query<&Province>,
    mut current_selection: ResMut<CurrentSelection>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        println!("Mouse button pressed");
        if let Some(mouse_pos) = mouse_to_world_coords(window_query, camera_query) {
            println!("{:?}", mouse_pos);

            // Find the province whose center is closest to the mouse world position
            let closest = provinces.iter().min_by(|a, b| {
                squared_distance(a.center, mouse_pos)
                    .partial_cmp(&squared_distance(b.center, mouse_pos))
                    .unwrap_or(Ordering::Equal)
            });

            match closest {
                Some(province) => {
                    println!(
                        "Closest province: id = {}, center = {:?}",
                        province.id, province.center
                    );

                    //current_selection.selected = Some(SelectedEntity::Province());
                    // → here you would normally select/highlight it, e.g.
                    // commands.entity(province_entity).insert(Selected);
                }
                None => println!("No provinces found"),
            }
        }
    }
}

fn squared_distance(a: Vec2, b: Vec2) -> f32 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}

fn mouse_to_world_coords(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    let (camera, cam_tf) = camera_query.single().ok()?;

    let cursor_pos = window.cursor_position()?;

    // Get ray (returns Result in recent Bevy versions)
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else {
        return None;
    };

    // Plane: y = 0.0, normal = +Y (or -Y — usually +Y works)
    // Use InfinitePlane3d (cheaper) or Plane3d if you want origin offset
    let plane_origin = Vec3::ZERO;
    let plane_normal = Vec3::Y; // pointing "up"

    // Returns distance along ray, or None if parallel / no intersection
    let Some(distance) = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))
    else {
        return None;
    };

    // Optional: ignore hits behind the camera
    if distance <= 0.0 {
        return None;
    }

    let point = ray.get_point(distance);

    // For y=0 plane → return (x, z)
    Some(point.xz())
}
