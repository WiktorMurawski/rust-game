use bevy::platform::collections::HashMap;
// plugins/turn.rs
use crate::components::army::{Army, HasActedThisTurn, PendingMove};
use crate::components::buildings::{ALL_BUILDINGS, BuildingType, Buildings};
use crate::components::country::{AIControlled, Country, DiplomacyChanged, Relation, Relations};
use crate::components::province::{Occupied, OwnedBy, Province};
use crate::states::{AppState, GamePhase};
use bevy::prelude::*;
use rand::Rng;
use rand::prelude::IndexedRandom;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TurnResolutionSet {
    Movement,
    Combat,
    Occupation,
    Economy,
    AIDecision,
    End,
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                TurnResolutionSet::AIDecision,
                TurnResolutionSet::Movement,
                TurnResolutionSet::Combat,
                TurnResolutionSet::Occupation,
                TurnResolutionSet::Economy,
                TurnResolutionSet::End,
            )
                .chain()
                .run_if(in_state(AppState::InGame).and(in_state(GamePhase::Processing))),
        )
        .add_systems(
            Update,
            (
                ai_build_buildings,
                ai_recruit_armies.after(ai_build_buildings),
                ai_move_armies.after(ai_recruit_armies),
                // ai_declare_war.after(ai_move_armies),
                ai_diplomacy.after(ai_move_armies),
            )
                .in_set(TurnResolutionSet::AIDecision)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            process_turn_moves.in_set(TurnResolutionSet::Movement),
        )
        .add_systems(Update, resolve_combat.in_set(TurnResolutionSet::Combat))
        .add_systems(
            Update,
            resolve_occupation.in_set(TurnResolutionSet::Occupation),
        )
        .add_systems(Update, process_economy.in_set(TurnResolutionSet::Economy))
        .add_systems(
            Update,
            crate::misc::empty_function.in_set(TurnResolutionSet::End),
        )
        .add_systems(
            Update,
            finish_processing
                .after(TurnResolutionSet::End)
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

fn resolve_combat(mut commands: Commands, armies: Query<(Entity, &Army)>) {
    let mut province_armies: HashMap<Entity, Vec<(Entity, Entity, u32)>> = HashMap::new();

    for (army_entity, army) in &armies {
        province_armies.entry(army.province).or_default().push((
            army_entity,
            army.owner,
            army.units,
        ));
    }

    for (province_entity, armies_in_prov) in province_armies {
        if armies_in_prov.len() < 2 {
            continue;
        }

        let mut owner_strength: HashMap<Entity, (u32, Vec<Entity>)> = HashMap::new();

        for (army_entity, owner, units) in armies_in_prov {
            let entry = owner_strength.entry(owner).or_default();
            entry.0 += units;
            entry.1.push(army_entity);
        }

        if owner_strength.len() < 2 {
            continue;
        }

        let mut weakest_owner = None;
        let mut weakest_strength = u32::MAX;

        for (&owner, &(total, _)) in &owner_strength {
            if total < weakest_strength {
                weakest_strength = total;
                weakest_owner = Some(owner);
            }
        }

        if let Some(loser_owner) = weakest_owner
            && let Some((_, losing_armies)) = owner_strength.get(&loser_owner)
        {
            for &army_entity in losing_armies {
                commands.entity(army_entity).despawn();
            }
            println!(
                "Combat in province {:?}: Owner {:?} lost {} units",
                province_entity, loser_owner, weakest_strength
            );
        }
    }
}

fn resolve_occupation(
    mut commands: Commands,
    provinces: Query<(Entity, &OwnedBy, Option<&Occupied>)>,
    armies: Query<(Entity, &Army)>,
) {
    let mut province_to_armies: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (army_entity, army) in &armies {
        province_to_armies
            .entry(army.province)
            .or_default()
            .push(army_entity);
    }

    for (prov_entity, owned_by, occupied_opt) in &provinces {
        let Some(armies_here) = province_to_armies.get(&prov_entity) else {
            continue;
        };

        if armies_here.is_empty() {
            continue;
        }

        let mut present_owners: Vec<Entity> = armies_here
            .iter()
            .filter_map(|&army_entity| armies.get(army_entity).ok().map(|army| army.1.owner))
            .collect();

        present_owners.sort();
        present_owners.dedup();

        if present_owners.is_empty() {
            continue;
        }

        if present_owners.contains(&owned_by.owner) {
            if occupied_opt.is_some() {
                commands.entity(prov_entity).remove::<Occupied>();
            }
            continue;
        }

        let occupier = present_owners[0];

        if occupied_opt.is_none_or(|o| o.occupier != occupier) {
            commands.entity(prov_entity).insert(Occupied { occupier });
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
            province_growth *= 0.0;
        }

        let growth_amount = (province.population as f32 * province_growth).round() as i32;

        province.population = (province.population as i32 + growth_amount).max(0) as u32;

        let mut province_income = province.base_income;

        for &building in &buildings.built {
            province_income += building.income_bonus();
        }

        province_income += province.population / 1000;

        if is_occupied {
            province_income = (province_income as f32 * 0.5) as u32;
        }

        *income_map.entry(owner).or_insert(0) += province_income;
    }

    for (country_entity, mut country) in &mut countries {
        if let Some(income) = income_map.get(&country_entity) {
            country.gold += *income;
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

fn ai_build_buildings(
    mut ai_countries: Query<(Entity, &mut Country), With<AIControlled>>,
    mut provinces: Query<(Entity, &OwnedBy, &mut Buildings)>,
) {
    let mut rng = rand::rng();

    for (country_entity, mut country) in &mut ai_countries {
        let affordable: Vec<BuildingType> = ALL_BUILDINGS
            .into_iter()
            .filter(|&b| country.gold >= b.cost())
            .collect();

        if affordable.is_empty() {
            continue;
        }

        let Some(choice) = affordable.choose(&mut rng).copied() else {
            continue;
        };

        let candidates: Vec<Entity> = provinces
            .iter()
            .filter(|(_, owned_by, buildings)| {
                owned_by.owner == country_entity && !buildings.built.contains(&choice)
            })
            .map(|(e, _, _)| e)
            .collect();

        if candidates.is_empty() {
            continue;
        }

        let chosen_prov = candidates[rng.random_range(0..candidates.len())];

        if let Ok(mut buildings) = provinces.get_mut(chosen_prov).map(|(_, _, b)| b) {
            buildings.built.push(choice);
            country.gold -= choice.cost();
        }
    }
}

fn ai_recruit_armies(
    mut commands: Commands,
    mut ai_countries: Query<(Entity, &mut Country), With<AIControlled>>,
    provinces: Query<(Entity, &Province, &OwnedBy, &Buildings)>, // ← add &Province here
) {
    let mut rng = rand::rng();

    for (country_entity, mut country) in &mut ai_countries {
        if country.gold < 100 {
            continue;
        }

        let barracks_provinces: Vec<(Entity, &Province)> = provinces
            .iter()
            .filter(|(_, _, owned_by, buildings)| {
                owned_by.owner == country_entity
                    && buildings.built.contains(&BuildingType::Barracks)
            })
            .map(|(e, prov, _, _)| (e, prov))
            .collect();

        if barracks_provinces.is_empty() {
            continue;
        }

        let (prov_entity, province) =
            barracks_provinces[rng.random_range(0..barracks_provinces.len())];

        commands.spawn((
            Army {
                owner: country_entity,
                province: prov_entity,
                units: 100,
            },
            Transform::from_xyz(province.center.x, 0.0, province.center.y),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        country.gold -= 100;
    }
}

fn ai_move_armies(
    mut commands: Commands,
    ai_countries: Query<Entity, With<AIControlled>>,
    armies: Query<(Entity, &Army)>,
    provinces: Query<(Entity, &Province, &OwnedBy)>,
    relations: Query<&Relations>,
    pending_moves: Query<&PendingMove>,
) {
    let mut rng = rand::rng();

    for country_entity in &ai_countries {
        let my_armies: Vec<(Entity, &Army)> = armies
            .iter()
            .filter(|(_, army)| army.owner == country_entity)
            .collect();

        for (army_entity, army) in my_armies {
            if !rng.random_bool(0.2) {
                continue;
            }

            let Ok((_, current_prov, _)) = provinces.get(army.province) else {
                continue;
            };

            let all_targets: Vec<(Entity, Entity)> = current_prov
                .neighbors
                .iter()
                .filter_map(|&nid| {
                    provinces
                        .iter()
                        .find(|(_e, p, _o)| p.id == nid)
                        .map(|(e, _, o)| (e, o.owner))
                })
                .collect();

            if all_targets.is_empty() {
                continue;
            }

            let enemy_targets: Vec<Entity> = all_targets
                .iter()
                .filter(|(_, owner)| *owner != country_entity)
                .filter(|(_, owner)| {
                    relations
                        .get(country_entity)
                        .is_ok_and(|r| r.get(*owner) == Relation::War)
                })
                .map(|(e, _)| *e)
                .collect();

            let friendly_targets: Vec<Entity> = all_targets
                .iter()
                .filter(|(_, owner)| *owner == country_entity)
                .map(|(e, _)| *e)
                .collect();

            let target_province = if !enemy_targets.is_empty() {
                if rng.random_bool(0.9) {
                    enemy_targets[rng.random_range(0..enemy_targets.len())]
                } else if !friendly_targets.is_empty() {
                    friendly_targets[rng.random_range(0..friendly_targets.len())]
                } else {
                    continue;
                }
            } else if !friendly_targets.is_empty() {
                friendly_targets[rng.random_range(0..friendly_targets.len())]
            } else {
                continue;
            };

            if pending_moves
                .get(army_entity)
                .is_ok_and(|p| p.target_province == target_province)
            {
                continue;
            }

            commands
                .entity(army_entity)
                .insert(PendingMove { target_province });
        }
    }
}

fn ai_diplomacy(
    commands: Commands,
    ai_countries: Query<(Entity, &Country), With<AIControlled>>,
    relations: Query<&mut Relations>,
    countries: Query<Entity, With<Country>>,
) {
    let r = rand::rng().random_range(0.0..1.0);
    if r < 0.2 {
        ai_declare_war(commands, ai_countries, relations, countries);
    } else if r < 0.4 {
        ai_propose_peace(commands, ai_countries, relations, countries);
    }
}

fn ai_declare_war(
    mut commands: Commands,
    ai_countries: Query<(Entity, &Country), With<AIControlled>>,
    mut relations: Query<&mut Relations>,
    countries: Query<Entity, With<Country>>,
) {
    let mut rng = rand::rng();

    for (country_entity, _) in &ai_countries {
        let possible_targets: Vec<Entity> = countries
            .iter()
            .filter(|&e| e != country_entity)
            .filter(|&e| {
                relations
                    .get(country_entity)
                    .map_or(true, |r| r.get(e) == Relation::Peace)
            })
            .collect();

        if possible_targets.is_empty() {
            continue;
        }

        let target = possible_targets[rng.random_range(0..possible_targets.len())];

        if let Ok(mut my_rels) = relations.get_mut(country_entity) {
            my_rels.set(target, Relation::War);
        }

        if let Ok(mut target_rels) = relations.get_mut(target) {
            target_rels.set(country_entity, Relation::War);
        }

        commands.trigger(DiplomacyChanged {
            declarer: country_entity,
            target,
            new_relation: Relation::War,
        });

        println!(
            "AI country {:?} declared war on {:?}",
            country_entity, target
        );
    }
}

fn ai_propose_peace(
    mut commands: Commands,
    ai_countries: Query<(Entity, &Country), With<AIControlled>>,
    mut relations: Query<&mut Relations>,
    countries: Query<Entity, With<Country>>,
) {
    println!("ai_propose_peace");
    let mut rng = rand::rng();

    for (country_entity, _) in &ai_countries {
        let possible_targets: Vec<Entity> = countries
            .iter()
            .filter(|&e| e != country_entity)
            .filter(|&e| {
                relations
                    .get(country_entity)
                    .is_ok_and(|r| r.get(e) == Relation::War)
            })
            .collect();

        if possible_targets.is_empty() {
            continue;
        }

        let target = possible_targets[rng.random_range(0..possible_targets.len())];

        if let Ok(mut my_rels) = relations.get_mut(country_entity) {
            my_rels.set(target, Relation::Peace);
        }

        if let Ok(mut target_rels) = relations.get_mut(target) {
            target_rels.set(country_entity, Relation::Peace);
        }

        commands.trigger(DiplomacyChanged {
            declarer: country_entity,
            target,
            new_relation: Relation::Peace,
        });

        println!(
            "AI country {:?} made peace with {:?}",
            country_entity, target
        );
    }
}
