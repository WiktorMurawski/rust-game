// ui/events_ui.rs
use crate::components::army::Army;
use crate::components::country::Country;
use crate::components::events::{EventEffect, PendingEvent};
use crate::components::player::ControlsCountry;
use crate::components::province::{OwnedBy, Province};
use crate::misc::{CommandsAndContexts, PlayerParams};
use crate::states::GamePhase;
use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, egui};

pub struct EventUIPlugin;

impl Plugin for EventUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            show_event_ui.run_if(in_state(GamePhase::Event)),
        );
    }
}

fn show_event_ui(
    commands_and_contexts: CommandsAndContexts,
    pending_event: Res<PendingEvent>,
    mut next_state: ResMut<NextState<GamePhase>>,
    player_params: PlayerParams,
    mut countries: Query<&mut Country>,
    mut armies: Query<&mut Army>,
    mut provinces: Query<(&mut Province, &OwnedBy)>,
) {
    let mut commands = commands_and_contexts.commands;
    let mut contexts = commands_and_contexts.contexts;

    let local_player = player_params.local_player;
    let player_controls = player_params.player_controls;

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    let event = &pending_event.event;

    let player_entity = local_player.0;

    egui::Window::new(&event.title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(400.0);

            ui.label(egui::RichText::new(&event.description).size(16.0));

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            for (i, option) in event.options.iter().enumerate() {
                if ui.button(&option.description).clicked() {
                    // Apply the effect
                    apply_event_effect(
                        &option.effect,
                        player_entity,
                        &player_controls,
                        &mut countries,
                        &mut armies,
                        &mut provinces,
                    );

                    commands.remove_resource::<PendingEvent>();
                    next_state.set(GamePhase::PlayerTurn);
                }

                if i < event.options.len() - 1 {
                    ui.add_space(5.0);
                }
            }
        });
}

fn apply_event_effect(
    effect: &EventEffect,
    player_entity: Entity,
    player_controls: &Query<&ControlsCountry>,
    countries: &mut Query<&mut Country>,
    armies: &mut Query<&mut Army>,
    provinces: &mut Query<(&mut Province, &OwnedBy)>,
) {
    println!("Applying effect {:?}", effect);

    let Ok(player_control) = player_controls.get(player_entity) else {
        println!("Warning: player control component not found");
        return;
    };
    let player_country_entity = player_control.0;

    match effect {
        EventEffect::PayGold(amount) | EventEffect::LoseGold(amount) => {
            if let Ok(mut country) = countries.get_mut(player_country_entity) {
                country.gold = country.gold.saturating_sub(*amount);
                println!("Lost {} gold", amount);
            } else {
                println!(
                    "Warning: player country entity {:?} not found",
                    player_country_entity
                );
            }
        }

        EventEffect::GainGold(amount) => {
            if let Ok(mut country) = countries.get_mut(player_country_entity) {
                country.gold += *amount;
                println!("Gained {} gold", amount);
            }
        }

        EventEffect::LoseArmyUnits(percentage) => {
            for mut army in armies.iter_mut() {
                if army.owner == player_country_entity {
                    let loss = (army.units as f32 * *percentage) as u32;
                    army.units = army.units.saturating_sub(loss);
                    println!("Army lost {} units", loss);
                }
            }
        }

        EventEffect::LosePopulation(percentage) => {
            for (mut province, owned_by) in provinces.iter_mut() {
                if owned_by.owner == player_country_entity {
                    let loss = (province.population as f32 * *percentage) as u32;
                    province.population = province.population.saturating_sub(loss);
                }
            }
            println!("Lost {:.0}% population", *percentage * 100.0);
        }
    }
}
