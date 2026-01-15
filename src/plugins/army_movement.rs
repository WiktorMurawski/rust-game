use std::cmp::Ordering;
use bevy::prelude::*;
use crate::components::army::{Army, PendingMove};
use crate::components::country::{Relation, Relations};
use crate::components::province::{OwnedBy, Province, TerrainType};
use crate::misc::{mouse_to_world_coords, squared_distance, MouseAndWindowAndCamera};
use crate::plugins::selection::{CurrentSelection, SelectedEntity};
use crate::states::{AppState, GamePhase};

pub struct ArmyMovementPlugin;

impl Plugin for ArmyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, queue_army_move.run_if(in_state(AppState::InGame)))
            .add_systems(Update, draw_pending_move_arrows.run_if(in_state(AppState::InGame)));
    }
}

fn queue_army_move(
    mut commands: Commands,
    current_selection: Res<CurrentSelection>,
    game_phase: Res<State<GamePhase>>,
    armies: Query<&Army>,
    province_query: Query<(Entity, &Province, &OwnedBy)>,
    pending_moves: Query<&PendingMove>,
    relations: Query<&Relations>,
    mouse_and_window_and_cam: MouseAndWindowAndCamera,
) {
    if *game_phase.get() != GamePhase::PlayerTurn {
        return;
    }

    let mouse_buttons = mouse_and_window_and_cam.mouse;
    let window_query = mouse_and_window_and_cam.window;
    let camera_query = mouse_and_window_and_cam.camera;

    if !mouse_buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(mouse_pos) = mouse_to_world_coords(window_query, camera_query) else {
        return;
    };

    // Only if we have exactly one army selected
    let SelectedEntity::Army(army_entity) = current_selection.entity.unwrap_or(SelectedEntity::Province(Entity::PLACEHOLDER)) else {
        return;
    };

    let Ok(army) = armies.get(army_entity) else { return };
    if pending_moves.get(army_entity).is_ok() {
        return; // already has pending move
    }

    // Find closest province under cursor
    let Some((target_province_entity, target_province, target_owned_by)) =
        province_query.iter().min_by(|(_, a, _), (_, b, _)| {
            squared_distance(a.center, mouse_pos)
                .partial_cmp(&squared_distance(b.center, mouse_pos))
                .unwrap_or(Ordering::Equal)
        }) else {
        return;
    };

    // Get current province info
    let Ok((current_province_entity, current_prov, current_owned_by)) =
        province_query.get(army.province) else {
        return;
    };

    let is_adjacent = current_prov.neighbors.contains(&target_province.id);
    let is_land = target_province.terrain != TerrainType::Water;

    // Diplomacy check
    let can_enter = if current_owned_by.0 == target_owned_by.0 {
        // Same owner → always allowed
        true
    } else {
        // Different owner → check if at war
        relations
            .get(army.owner)
            .ok()
            .is_some_and(|rels| rels.get(target_owned_by.0) == Relation::War)
    };

    if !is_adjacent || !is_land || !can_enter {
        println!(
            "Invalid move attempt: adjacent={}, land={}, can_enter={}",
            is_adjacent, is_land, can_enter
        );
        return;
    }

    // Queue the move
    commands.entity(army_entity).insert(PendingMove {
        target_province: target_province_entity,
    });

    println!(
        "Move queued: Army {:?} from province {:?} to province {:?}",
        army_entity, current_province_entity, target_province_entity
    );
}

fn draw_pending_move_arrows(
    mut gizmos: Gizmos,
    pending_armies: Query<(&PendingMove, &GlobalTransform), With<Army>>,
    provinces: Query<&Province>,
) {
    for (pending, transform) in &pending_armies {
        if let Ok(target) = provinces.get(pending.target_province) {
            let start = transform.translation();
            let end   = Vec3::new(target.center.x, 0.15, target.center.y); // slightly above ground

            gizmos.arrow(
                start,
                end,
                Color::srgba(0.1, 0.9, 0.4, 0.85), // player green
            );
        }
    }
}
