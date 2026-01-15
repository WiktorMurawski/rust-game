use crate::components::country::{Country, DiplomacyChanged, Relation, Relations};
use crate::components::player::{ControlsCountry, LocalPlayer};
use crate::components::province::{Occupied, OwnedBy};
use crate::plugins::selection::{CurrentSelection, SelectedEntity};
use crate::states::AppState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct DiplomacyPlugin;

impl Plugin for DiplomacyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Messages::<DiplomacyChanged>::default())
            .add_systems(
                EguiPrimaryContextPass,
                diplomacy_window.run_if(in_state(AppState::InGame)),
            )
            .add_observer(on_peace_transfer_occupations);
    }
}

fn on_peace_transfer_occupations(
    trigger: On<DiplomacyChanged>,
    mut commands: Commands,
    mut provinces: Query<(Entity, &mut OwnedBy, &Occupied)>,
    relations: Query<&Relations>,
) {
    let ev = trigger.event();

    if ev.new_relation != Relation::Peace {
        return;
    }

    let peace_parties = [ev.declarer, ev.target];

    for (prov_entity, mut owned_by, occupied) in &mut provinces {
        let occupier = occupied.occupier;

        if !peace_parties.contains(&occupier) {
            continue;
        }

        if let Ok(occupier_rels) = relations.get(occupier)
            && occupier_rels.get(owned_by.owner) == Relation::Peace
        {
            let old_owner = owned_by.owner;
            owned_by.owner = occupier;
            commands.entity(prov_entity).remove::<Occupied>();

            println!(
                "Immediate peace transfer: Province {:?} from {:?} to occupier {:?}",
                prov_entity, old_owner, occupier
            );
        }
    }
}

#[derive(SystemParam)]
struct CommandsAndContexts<'w, 's> {
    commands: Commands<'w, 's>,
    contexts: EguiContexts<'w, 's>,
}

fn diplomacy_window(
    commands_and_contexts: CommandsAndContexts,
    current_selection: Res<CurrentSelection>,
    provinces: Query<&OwnedBy>,
    countries: Query<&Country>,
    mut relations_q: Query<&mut Relations>,
    local_player: Res<LocalPlayer>,
    player_controls: Query<&ControlsCountry>,
) {
    let (mut commands, mut contexts) = (
        commands_and_contexts.commands,
        commands_and_contexts.contexts,
    );

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    // Only when a province is selected
    let SelectedEntity::Province(selected_province) = current_selection
        .entity
        .unwrap_or(SelectedEntity::Province(Entity::PLACEHOLDER))
    else {
        return;
    };

    let Ok(owned_by) = provinces.get(selected_province) else {
        return;
    };
    let selected_country_entity = owned_by.owner;

    let Ok(selected_country) = countries.get(selected_country_entity) else {
        return;
    };

    // Player's controlled country
    let player_entity = local_player.0;
    let Ok(player_control) = player_controls.get(player_entity) else {
        return;
    };
    let player_country_entity = player_control.0;

    if selected_country_entity == player_country_entity {
        return; // don't show for own country
    }

    egui::Window::new(format!("Diplomacy – {}", selected_country.name))
        .anchor(egui::Align2::RIGHT_TOP, [20.0, 60.0])
        .resizable(false)
        .collapsible(false)
        .default_size([280.0, 160.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(&selected_country.name);
                ui.add_space(8.0);

                // Try to get player's relations (mutable)
                if let Ok(mut player_relations) = relations_q.get_mut(player_country_entity) {
                    let current = player_relations.get(selected_country_entity);

                    let status_text = match current {
                        Relation::Peace => {
                            egui::RichText::new("Peace").color(egui::Color32::LIGHT_GREEN)
                        }
                        Relation::War => egui::RichText::new("At War").color(egui::Color32::RED),
                    };

                    ui.label(status_text);
                    ui.add_space(12.0);

                    let button_text = match current {
                        Relation::Peace => "Declare War",
                        Relation::War => "Propose Peace",
                    };

                    let button_color = match current {
                        Relation::Peace => egui::Color32::from_rgb(180, 40, 40),
                        Relation::War => egui::Color32::from_rgb(60, 140, 60),
                    };

                    // Now the button works normally
                    if ui
                        .add(egui::Button::new(button_text).fill(button_color))
                        .clicked()
                    {
                        let new_relation = match current {
                            Relation::Peace => Relation::War,
                            Relation::War => Relation::Peace,
                        };

                        player_relations.set(selected_country_entity, new_relation);

                        // Mirror relation (symmetric diplomacy - recommended)
                        if let Ok(mut target_relations) =
                            relations_q.get_mut(selected_country_entity)
                        {
                            target_relations.set(player_country_entity, new_relation);
                        }

                        // Trigger event
                        commands.trigger(DiplomacyChanged {
                            declarer: player_country_entity,
                            target: selected_country_entity,
                            new_relation,
                        });

                        println!(
                            "Diplomacy changed: {} ↔ {} → {:?}",
                            selected_country.name, player_country_entity, new_relation
                        );
                    }
                } else {
                    ui.colored_label(egui::Color32::RED, "No diplomatic relations data");
                }
            });
        });
}
