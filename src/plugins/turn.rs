// plugins/turn.rs
use bevy::prelude::*;
use crate::components::army::{Army, HasActedThisTurn, PendingMove};
use crate::components::country::{Relation, Relations};
use crate::components::province::{Occupied, OwnedBy};
use crate::states::{AppState, GamePhase};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TurnResolutionSet {
    Movement,
    Occupation,
    // PeaceResolution,
    // Later: Combat, Income, PopulationGrowth, PeaceResolution, Cleanup, etc.
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app
            // Define ordering only for Processing state
            .configure_sets(
                Update,
                (
                    TurnResolutionSet::Movement,
                    TurnResolutionSet::Occupation,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame).and(in_state(GamePhase::Processing))),
            )
            .add_systems(
                Update,
                process_turn_moves.in_set(TurnResolutionSet::Movement),
            )
            .add_systems(
                Update,
                apply_occupation.in_set(TurnResolutionSet::Occupation),
            )
            .add_systems(
                Update,
                finish_processing
                    .after(TurnResolutionSet::Occupation)  // or after the last set
                    .run_if(in_state(AppState::InGame).and(in_state(GamePhase::Processing))),
            );
    }
}

fn process_turn_moves(
    mut armies: Query<&mut Army>,
    pending_moves_q: Query<(Entity, &PendingMove)>,
) {
    for (entity, pending) in &pending_moves_q {
        if let Ok(mut army) = armies.get_mut(entity) {
            army.province = pending.target_province;
        }
    }
}

fn apply_occupation(
    mut commands: Commands,
    armies: Query<&Army>,
    pending_moves_q: Query<(Entity, &PendingMove)>,
    provinces: Query<(Entity, &OwnedBy, Option<&Occupied>)>,
    relations: Query<&Relations>,
) {
    for (army_entity, pending) in &pending_moves_q {
        let Ok(army) = armies.get(army_entity) else { continue };

        let target_province_entity = pending.target_province;

        let Ok((prov_entity, owned_by, _occupied)) = provinces.get(target_province_entity) else {
            continue;
        };

        // Only occupy if moving into someone else's province
        if owned_by.owner != army.owner {
            // Check if at war
            if let Ok(invader_rels) = relations.get(army.owner)
                && invader_rels.get(owned_by.owner) == Relation::War {
                    // Apply occupation
                    commands.entity(prov_entity).insert(Occupied {
                        occupier: army.owner,
                    });

                    println!(
                        "Province {:?} occupied by country {:?} (army from {:?})",
                        target_province_entity, army.owner, army_entity
                    );
                }
        }
    }
}

fn finish_processing(
    mut commands: Commands,
    pending_moves_q: Query<Entity, With<PendingMove>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    // Clean up all pending move markers
    for entity in &pending_moves_q {
        commands.entity(entity).remove::<PendingMove>();
        commands.entity(entity).remove::<HasActedThisTurn>();
    }

    // Go back to player turn
    next_state.set(GamePhase::PlayerTurn);
}
