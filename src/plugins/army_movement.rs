use std::cmp::Ordering;
use bevy::prelude::*;
use crate::components::army::{Army, HasActedThisTurn, PendingMove};
use crate::components::province::Province;
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
    province_query: Query<(Entity, &Province)>,
    pending_moves: Query<&PendingMove>,
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

    let Some(mouse_pos) = mouse_to_world_coords(window_query,camera_query) else {
        return;
    };

    // Only if we have exactly one army selected
    let SelectedEntity::Army(army_entity) = current_selection.entity.unwrap_or(SelectedEntity::Province(Entity::PLACEHOLDER)) else {
        return;
    };

    if armies.get(army_entity).is_err() || pending_moves.get(army_entity).is_ok() {
        // already has pending move or invalid army
        return;
    }

    // Find closest province under cursor
    let Some((target_province_entity, target_province)) = province_query.iter().min_by(|(_, a), (_, b)| {
        squared_distance(a.center, mouse_pos)
            .partial_cmp(&squared_distance(b.center, mouse_pos))
            .unwrap_or(Ordering::Equal)
    })
    else { return; };

    // Very simple validity check for now (later: real adjacency + movement points)
    let Ok(army) = armies.get(army_entity) else { return };
    let Ok((current_province_entity, current_prov)) = province_query.get(army.province) else { return };

    let is_adjacent = current_prov.neighbors.contains(&target_province.id);
    let same_owner  = true;

    if !is_adjacent || !same_owner {
        // optional: play sound / show message "Cannot move there"
        return;
    }

    // Queue the move
    commands.entity(army_entity).insert(PendingMove {
        target_province: target_province_entity,
    });

    // Optional: mark as acted (if you want only one move per turn)
    // commands.entity(army_entity).insert(HasActedThisTurn);

    println!("Move queued: Army {:?} from {:?} to {:?}", army_entity, current_province_entity, target_province_entity);
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
