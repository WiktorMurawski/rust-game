use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use bevy_egui::egui::emath;
use crate::components::country::{Country, Relations, Relation};
use crate::components::player::{ControlsCountry, LocalPlayer};
use crate::plugins::selection::{CurrentSelection, SelectedEntity};
use crate::states::AppState;
use crate::components::province::OwnedBy;

pub struct DiplomacyUIPlugin;

impl Plugin for DiplomacyUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,  // ← usually better than EguiPrimaryContextPass for most cases
            diplomacy_window.run_if(in_state(AppState::InGame)),
        );
    }
}

fn diplomacy_window(
    mut contexts: EguiContexts,
    current_selection: Res<CurrentSelection>,
    provinces: Query<&OwnedBy>,
    countries: Query<&Country>,
    mut relations_q: Query<&mut Relations>,
    local_player: Res<LocalPlayer>,
    player_controls: Query<&ControlsCountry>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    // Only when a province is selected
    let SelectedEntity::Province(selected_province) = current_selection.entity.unwrap_or(SelectedEntity::Province(Entity::PLACEHOLDER)) else {
        return;
    };

    let Ok(owned_by) = provinces.get(selected_province) else { return };
    let selected_country_entity = owned_by.0;

    let Ok(selected_country) = countries.get(selected_country_entity) else { return };

    // Player's controlled country
    let player_entity = local_player.0;
    let Ok(player_control) = player_controls.get(player_entity) else { return };
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
                        Relation::Peace => egui::RichText::new("Peace").color(egui::Color32::LIGHT_GREEN),
                        Relation::War   => egui::RichText::new("At War").color(egui::Color32::RED),
                    };

                    ui.label(status_text);
                    ui.add_space(12.0);

                    let button_text = match current {
                        Relation::Peace => "Declare War",
                        Relation::War   => "Propose Peace",
                    };

                    let button_color = match current {
                        Relation::Peace => egui::Color32::from_rgb(180, 40, 40),
                        Relation::War   => egui::Color32::from_rgb(60, 140, 60),
                    };

                    // Now the button works normally
                    if ui
                        .add(egui::Button::new(button_text).fill(button_color))
                        .clicked()
                    {
                        let new_relation = match current {
                            Relation::Peace => Relation::War,
                            Relation::War   => Relation::Peace,
                        };

                        player_relations.set(selected_country_entity, new_relation);

                        // Optional: mirror the relation (symmetric diplomacy)
                        // if let Ok(mut target_rel) = relations_q.get_mut(selected_country_entity) {
                        //     target_rel.set(player_country_entity, new_relation);
                        // }

                        println!(
                            "Diplomacy changed: {} ↔ {} → {:?}",
                            selected_country.name,
                            player_country_entity,
                            new_relation
                        );
                    }
                } else {
                    ui.colored_label(egui::Color32::RED, "No diplomatic relations data");
                }
            });
        });
}

//
// fn diplomacy_window(
//     mut contexts: EguiContexts,
//     current_selection: Res<CurrentSelection>,
//     provinces: Query<&OwnedBy>,
//     countries: Query<&Country>,
//     mut relations_q: Query<&mut Relations>,
//     local_player: Res<LocalPlayer>,
//     player_controls: Query<&ControlsCountry>,
// ) {
//     let ctx = match contexts.ctx_mut() {
//         Ok(ctx) => ctx,
//         Err(_) => return,
//     };
//
//     // Only show when a province is selected
//     let SelectedEntity::Province(selected_province) = current_selection.entity.unwrap_or(SelectedEntity::Province(Entity::PLACEHOLDER)) else {
//         return;
//     };
//
//     // Get the owner of the selected province
//     let Ok(owned_by) = provinces.get(selected_province) else { return };
//     let selected_country_entity = owned_by.0;
//
//     // Get the Country component of the selected province's owner
//     let Ok(selected_country) = countries.get(selected_country_entity) else { return };
//
//     // Get the player's controlled country
//     let player_country_entity = local_player.0;  // the player entity
//
//     let Ok(player_controls) = player_controls.get(player_country_entity) else { return };
//     let player_country_entity = player_controls.0;  // the country entity the player controls
//
//     // Don't show diplomacy window for own country
//     if selected_country_entity == player_country_entity {
//         return;
//     }
//
//     egui::Window::new(format!("Diplomacy – {}", selected_country.name))
//         .anchor(egui::Align2::RIGHT_TOP, [0.0, 60.0])
//         .resizable(false)
//         .collapsible(false)
//         .default_size([260.0, 140.0])
//         .show(ctx, |ui| {
//             ui.vertical_centered(|ui| {
//                 ui.heading(&selected_country.name);
//                 ui.add_space(8.0);
//
//                 // Current relation status
//                 if let Ok(player_relations) = relations_q.get(player_country_entity) {
//                     let current = player_relations.get(selected_country_entity);
//
//                     let status_text = match current {
//                         Relation::Peace => egui::RichText::new("Peace").color(egui::Color32::LIGHT_GREEN),
//                         Relation::War   => egui::RichText::new("At War").color(egui::Color32::RED),
//                     };
//
//                     ui.label(status_text);
//                     ui.add_space(12.0);
//
//                     // Declare war / make peace button
//                     let button_text = match current {
//                         Relation::Peace => "Declare War",
//                         Relation::War   => "Propose Peace",
//                     };
//
//                     let button_color = match current {
//                         Relation::Peace => egui::Color32::from_rgb(180, 40, 40),
//                         Relation::War   => egui::Color32::from_rgb(60, 140, 60),
//                     };
//
//                     if ui
//                         .add(egui::Button::new(button_text).fill(button_color))
//                         .clicked()
//                         && let Ok(mut mut_relations) = relations_q.get_mut(player_country_entity) {
//                             let new_relation = match current {
//                                 Relation::Peace => Relation::War,
//                                 Relation::War   => Relation::Peace,
//                             };
//
//                             mut_relations.set(selected_country_entity, new_relation);
//
//                             // Optional: also set the reverse relation (symmetric diplomacy)
//                             // if let Ok(mut target_relations) = relations_q.get_mut(selected_country_entity) {
//                             //     target_relations.set(player_country_entity, new_relation);
//                             // }
//
//                             println!(
//                                 "{} → {} changed to {:?}",
//                                 selected_country.name, // debug only
//                                 player_country_entity,
//                                 new_relation
//                             );
//
//                             // You can send an event here later if needed
//                             // commands.add(|world| {
//                             //     world.send_event(DiplomacyChanged { ... });
//                             // });
//                         }
//                 } else {
//                     ui.label("No diplomatic relations");
//                 }
//             });
//         });
// }