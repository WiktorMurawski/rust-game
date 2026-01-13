use std::cmp::Ordering;

use bevy::ecs::system::SystemParam;
use bevy::prelude::Vec2;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use crate::resources::MapSize;
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

// Marker component for selected entities
#[derive(Component)]
pub struct Selected;

// Resource to track selection state
#[derive(Resource, Default, Debug)]
pub struct CurrentSelection {
    pub entity: Option<SelectedEntity>,
    // Or for multi-select:
    // pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectedEntity {
    Province(Entity),
    //Army(Entity),
}

//fn print_selection(
//    current_selection: Res<CurrentSelection>,
//    selected_entities: Query<Entity, With<Selected>>,
//) {
//    println!("Current selection: {:?}", current_selection);
//    selected_entities.iter().for_each(|e| println!("{:?}", e));
//}

#[derive(SystemParam)]
struct MouseAndWindowAndCamera<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}

fn update_selection(
    mut commands: Commands,
    // mouse_buttons: Res<ButtonInput<MouseButton>>,
    //provinces: Query<&Province>,
    province_query: Query<(Entity, &Province)>,
    mut current_selection: ResMut<CurrentSelection>,
    selected_query: Query<Entity, With<Selected>>,
    map_size: Res<MapSize>,
    mouse_and_window_and_camera: MouseAndWindowAndCamera,
    mut contexts: EguiContexts,
    //window_query: Query<&Window, With<PrimaryWindow>>,
    //camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if contexts.ctx_mut().expect("REASON").wants_pointer_input() {
        return;  // UI has priority → don't process game clicks
    }

    let mouse_buttons = mouse_and_window_and_camera.mouse;

    if mouse_buttons.just_pressed(MouseButton::Left) {
        // println!("LMB pressed");

        let provinces = province_query;

        let window_query = mouse_and_window_and_camera.window;
        let camera_query = mouse_and_window_and_camera.camera;

        if let Some(mouse_pos) = mouse_to_world_coords(window_query, camera_query) {
            if (mouse_pos.x).abs() * 2.0 > map_size.0.x || (mouse_pos.y).abs() * 2.0 > map_size.0.y
            {
                for entity in selected_query.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
                current_selection.entity = None;
                return;
            }

            let closest = provinces.iter().min_by(|(_, a), (_, b)| {
                squared_distance(a.center, mouse_pos)
                    .partial_cmp(&squared_distance(b.center, mouse_pos))
                    .unwrap_or(Ordering::Equal)
            });

            if let Some((province_entity, _province)) = closest {
                if current_selection.entity == Some(SelectedEntity::Province(province_entity)) {
                    return;
                }

                for entity in selected_query.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
                current_selection.entity = None;

                commands.entity(province_entity).insert(Selected);
                current_selection.entity = Some(SelectedEntity::Province(province_entity));
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

    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else {
        return None;
    };

    let plane_origin = Vec3::ZERO;
    let plane_normal = Vec3::Y;

    let distance = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))?;

    if distance <= 0.0 {
        return None;
    }

    let point = ray.get_point(distance);

    Some(point.xz())
}
