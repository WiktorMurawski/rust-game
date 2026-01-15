use crate::components::army::Army;
use crate::resources::MapSize;
use crate::{components::province::Province, states::AppState};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use std::cmp::Ordering;
use crate::misc::{mouse_to_world_coords, squared_distance, MouseAndWindowAndCamera};

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
    Army(Entity),
}

// fn print_selection(
//    current_selection: Res<CurrentSelection>,
//    selected_entities: Query<Entity, With<Selected>>,
// ) {
//    println!("Current selection: {:?}", current_selection);
//    selected_entities.iter().for_each(|e| println!("{:?}", e));
// }

#[derive(SystemParam)]
struct SelectionParams<'w, 's> {
    current_selection: ResMut<'w, CurrentSelection>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
}

fn update_selection(
    mut commands: Commands,
    mut contexts: EguiContexts,
    province_query: Query<(Entity, &Province)>,
    army_query: Query<(Entity, &Army, &GlobalTransform)>,
    selection_params: SelectionParams,
    // mut current_selection: ResMut<CurrentSelection>,
    // selected_query: Query<Entity, With<Selected>>,
    map_size: Res<MapSize>,
    mouse_and_window_and_camera: MouseAndWindowAndCamera,
) {
    if contexts
        .ctx_mut()
        .ok()
        .is_some_and(|ctx| ctx.wants_pointer_input())
    {
        return;
    }

    let mouse_buttons = mouse_and_window_and_camera.mouse;
    let window_query = mouse_and_window_and_camera.window;
    let camera_query = mouse_and_window_and_camera.camera;

    let mut current_selection = selection_params.current_selection;
    let selected_query = selection_params.selected_query;

    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(mouse_pos) = mouse_to_world_coords(window_query, camera_query) else {
        return;
    };

    // Check if clicked outside map
    if (mouse_pos.x).abs() * 2.0 > map_size.0.x || (mouse_pos.y).abs() * 2.0 > map_size.0.y {
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        current_selection.entity = None;
        return;
    }

    // Step 1: Check if clicked on an army (simple 2D distance)
    let mut closest_army: Option<(Entity, f32)> = None;

    const RADIUS: f32 = 5.0;

    for (entity, _army, transform) in army_query.iter() {
        let army_pos_2d = transform.translation().xz(); // Get x,z position
        let distance = mouse_pos.distance(army_pos_2d);

        // Click radius of 3 units
        if distance < RADIUS && closest_army.is_none_or(|(_, d)| distance < d) {
            closest_army = Some((entity, distance));
        }
    }

    // If clicked an army, select it
    if let Some((army_entity, _)) = closest_army {
        if current_selection.entity != Some(SelectedEntity::Army(army_entity)) {
            // Deselect all
            for entity in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }

            // Select army
            commands.entity(army_entity).insert(Selected);
            current_selection.entity = Some(SelectedEntity::Army(army_entity));
        }
        return; // Don't check provinces if we clicked an army
    }

    // Step 2: Province selection (your existing code)
    let closest = province_query.iter().min_by(|(_, a), (_, b)| {
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

        commands.entity(province_entity).insert(Selected);
        current_selection.entity = Some(SelectedEntity::Province(province_entity));
    }
}
