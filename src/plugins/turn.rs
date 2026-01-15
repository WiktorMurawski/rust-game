// plugins/turn.rs
use bevy::prelude::*;
use crate::components::army::{Army, HasActedThisTurn, PendingMove};
use crate::components::province::Province;
use crate::states::{AppState, GamePhase};

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                process_turn_moves
                    .run_if(in_state(AppState::InGame).and(in_state(GamePhase::Processing))),
            );
    }
}

fn process_turn_moves(
    mut commands: Commands,
    mut armies: Query<&mut Army>,
    pending_moves_q: Query<(Entity, &PendingMove)>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    let mut any_moves = false;

    for (entity, pending) in &pending_moves_q {
        any_moves = true;

        if let Ok(mut army) = armies.get_mut(entity) {
            army.province = pending.target_province;

            // Cleanup markers
            commands.entity(entity).remove::<PendingMove>();
            commands.entity(entity).remove::<HasActedThisTurn>();
        }
    }

    if any_moves {
        // Give some feeling of processing
        // (later: delay, animations, combat resolution, etc.)
    }

    // Always go back to player turn after processing
    next_state.set(GamePhase::PlayerTurn);
}