use bevy::platform::collections::HashMap;
// plugins/turn.rs
use crate::components::army::{Army, HasActedThisTurn, PendingMove};
use crate::components::buildings::Buildings;
use crate::components::country::{Country, Relation, Relations};
use crate::components::province::{Occupied, OwnedBy, Province};
use crate::states::{AppState, GamePhase};
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TurnResolutionSet {
    Movement,
    Occupation,
    Economy,
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                TurnResolutionSet::Movement,
                TurnResolutionSet::Occupation,
                TurnResolutionSet::Economy,
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
        .add_systems(Update, process_economy.in_set(TurnResolutionSet::Economy))
        .add_systems(
            Update,
            finish_processing
                .after(TurnResolutionSet::Occupation)
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
        let Ok(army) = armies.get(army_entity) else {
            continue;
        };

        let target_province_entity = pending.target_province;

        let Ok((prov_entity, owned_by, _occupied)) = provinces.get(target_province_entity) else {
            continue;
        };

        if owned_by.owner != army.owner {
            // Check if at war
            if let Ok(invader_rels) = relations.get(army.owner)
                && invader_rels.get(owned_by.owner) == Relation::War
            {
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

fn process_economy(
    mut provinces: Query<(Entity, &mut Province, &OwnedBy, &Buildings)>,
    occupied: Query<&Occupied>,
    mut countries: Query<(Entity, &mut Country)>,
) {
    let mut income_map: HashMap<Entity, u32> = HashMap::new();

    for (prov_entity, mut province, owned_by, buildings) in &mut provinces {
        let owner = if let Ok(occ) = occupied.get(prov_entity) {
            occ.occupier
        } else {
            owned_by.owner
        };

        let mut province_growth = province.base_growth;

        for &building in &buildings.built {
            province_growth += building.growth_bonus();
        }

        let is_occupied = occupied.get(prov_entity).is_ok();
        if is_occupied {
            province_growth *= 0.7;
        }

        let growth_amount = (province.population as f32 * province_growth).round() as i32;

        province.population = (province.population as i32 + growth_amount).max(0) as u32;

        let mut province_income = province.base_income;

        for &building in &buildings.built {
            province_income += building.income_bonus();
        }

        province_income += province.population / 1000;

        if is_occupied {
            province_income = (province_income as f32 * 0.6) as u32; // 60% when occupied
        }

        *income_map.entry(owner).or_insert(0) += province_income;
    }

    // Apply total income to countries
    for (country_entity, mut country) in &mut countries {
        if let Some(income) = income_map.get(&country_entity) {
            country.gold += *income as u64;
            println!(
                "{} gained {} gold this turn (total: {})",
                country.name, income, country.gold
            );
        }
    }
}

fn finish_processing(
    mut commands: Commands,
    pending_moves_q: Query<Entity, With<PendingMove>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for entity in &pending_moves_q {
        commands.entity(entity).remove::<PendingMove>();
        commands.entity(entity).remove::<HasActedThisTurn>();
    }

    next_state.set(GamePhase::PlayerTurn);
}
