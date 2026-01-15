use crate::components::army::{Army, PendingMove};
use crate::components::country::{Relation, Relations};
use crate::components::province::{OwnedBy, Province, TerrainType};
use crate::misc::{MouseAndWindowAndCamera, mouse_to_world_coords, squared_distance};
use crate::plugins::selection::{CurrentSelection, SelectedEntity};
use crate::states::{AppState, GamePhase};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::cmp::Ordering;

pub struct ArmyMovementPlugin;

impl Plugin for ArmyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, queue_army_move.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                draw_pending_move_arrows.run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(SystemParam)]
struct ArmyMoveQueries<'w, 's> {
    armies: Query<'w, 's, &'static Army>,
    pending_moves: Query<'w, 's, &'static PendingMove>,
}

fn queue_army_move(
    mut commands: Commands,
    current_selection: Res<CurrentSelection>,
    game_phase: Res<State<GamePhase>>,
    army_move_queries: ArmyMoveQueries,
    province_query: Query<(Entity, &Province, &OwnedBy)>,
    relations: Query<&Relations>,
    mouse_and_window_and_cam: MouseAndWindowAndCamera,
) {
    if *game_phase.get() != GamePhase::PlayerTurn {
        return;
    }

    let armies = army_move_queries.armies;
    let pending_moves = army_move_queries.pending_moves;
    let mouse_buttons = mouse_and_window_and_cam.mouse;
    let window_query = mouse_and_window_and_cam.window;
    let camera_query = mouse_and_window_and_cam.camera;

    if !mouse_buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(mouse_pos) = mouse_to_world_coords(window_query, camera_query) else {
        return;
    };

    let Some(SelectedEntity::Army(army_entity)) = current_selection.entity else {
        return;
    };

    let Ok(army) = armies.get(army_entity) else {
        return;
    };

    let Some((target_province_entity, target_province, target_owned_by)) =
        province_query.iter().min_by(|(_, a, _), (_, b, _)| {
            squared_distance(a.center, mouse_pos)
                .partial_cmp(&squared_distance(b.center, mouse_pos))
                .unwrap_or(Ordering::Equal)
        })
    else {
        return;
    };

    let Ok((_current_prov_entity, current_prov, _)) = province_query.get(army.province) else {
        return;
    };

    let is_adjacent = current_prov.neighbors.contains(&target_province.id);
    let is_land = target_province.terrain != TerrainType::Water;

    let can_enter = if army.owner == target_owned_by.owner {
        true
    } else {
        relations
            .get(army.owner)
            .ok()
            .is_some_and(|rels| rels.get(target_owned_by.owner) == Relation::War)
    };

    let is_valid_target = is_adjacent && is_land && can_enter;

    if let Ok(pending) = pending_moves.get(army_entity) {
        if !is_valid_target {
            return;
        }

        if pending.target_province == target_province_entity {
            commands.entity(army_entity).remove::<PendingMove>();
            return;
        }

        commands.entity(army_entity).insert(PendingMove {
            target_province: target_province_entity,
        });
        return;
    }

    if !is_valid_target {
        return;
    }

    commands.entity(army_entity).insert(PendingMove {
        target_province: target_province_entity,
    });
}

fn draw_pending_move_arrows(
    mut gizmos: Gizmos,
    pending_armies: Query<(&PendingMove, &GlobalTransform), With<Army>>,
    provinces: Query<&Province>,
) {
    for (pending, transform) in &pending_armies {
        if let Ok(target) = provinces.get(pending.target_province) {
            let start = transform.translation();
            let end = Vec3::new(target.center.x, 0.15, target.center.y);

            gizmos.arrow(start, end, Color::srgba(0.1, 0.9, 0.4, 0.85));
        }
    }
}
