use crate::components::buildings::Buildings;
use crate::components::country::*;
use crate::components::province::*;
use crate::plugins::selection::SelectedEntity;
use crate::plugins::selection::{CurrentSelection};
use crate::states::AppState;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct ProvinceInfoUI;

impl Plugin for ProvinceInfoUI {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            province_info_ui.run_if(in_state(AppState::InGame)),
        );
    }
}

fn province_info_ui(
    mut contexts: EguiContexts,
    selection: Res<CurrentSelection>,
    provinces: Query<(&Province, Option<&OwnedBy>, &Buildings, Option<&Occupied>)>,
    countries: Query<&Country>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Province Info")
        .default_pos([10.0, 10.0])
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .resizable(false)
        .default_size([240.0, 380.0])
        .show(ctx, |ui| {
            if let Some(SelectedEntity::Province(entity)) = selection.entity {
                if let Ok((province, owner_opt, buildings, occupied_opt)) = provinces.get(entity) {
                    ui.heading(format!("Province {}", province.id));
                    ui.separator();

                    ui.label(format!("Terrain: {:?}", province.terrain));
                    ui.label(format!(
                        "Center: {:.1}, {:.1}",
                        province.center.x, province.center.y
                    ));

                    // Owner
                    if let Some(owner) = owner_opt {
                        if let Ok(country) = countries.get(owner.owner) {
                            ui.label(format!("Owner: {}", country.name));
                        } else {
                            ui.label("Owner: Unknown");
                        }
                    } else {
                        ui.label("Owner: None (unclaimed)");
                    }

                    // Occupation status
                    if let Some(occupied) = occupied_opt {
                        if let Ok(occ_country) = countries.get(occupied.occupier) {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Occupied by: {}", occ_country.name),
                            );
                        } else {
                            ui.colored_label(egui::Color32::RED, "Occupied");
                        }
                    }

                    ui.separator();

                    // Population & growth
                    ui.label(format!("Population: {}", province.population));
                    ui.label(format!(
                        "Growth per turn: +{:.1}% ({:+} people)",
                        province.base_growth * 100.0,
                        (province.population as f32 * province.base_growth).round() as i32
                    ));

                    // Buildings summary
                    if !buildings.built.is_empty() {
                        ui.label("Buildings:");
                        for &b in &buildings.built {
                            ui.label(format!("• {}", b.name()));
                        }
                    } else {
                        ui.label("No buildings");
                    }

                    ui.separator();

                    // Income calculation (same logic as in your economy system)
                    let mut income = province.base_income as f32;

                    // Building bonuses
                    for &building in &buildings.built {
                        income += building.income_bonus() as f32;
                    }

                    // Simple population contribution (e.g. 1 gold per 1000 people)
                    income += (province.population / 1000) as f32;

                    // Occupation penalty (optional)
                    if occupied_opt.is_some() {
                        income *= 0.6; // e.g. 60% income when occupied
                        ui.label("Occupied: income reduced");
                    }

                    ui.label(format!(
                        "Estimated income this turn: {} gold",
                        income.round() as u32
                    ));

                    ui.separator();

                    ui.label(format!("Neighbors: {}", province.neighbors.len()));
                } else {
                    ui.label("Invalid province data");
                }
            } else {
                ui.label("No province selected");
            }
        });
}

// fn province_info_ui(
//     mut contexts: EguiContexts,
//     selection: Res<CurrentSelection>,
//     provinces: Query<(&Province, Option<&OwnedBy>), With<Selected>>,
//     countries: Query<&Country>,
// ) {
//     let Ok(ctx) = contexts.ctx_mut() else {
//         return;
//     };
//
//     egui::Window::new("Province Info")
//         .default_pos([10.0, 10.0])
//         .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
//         .resizable(false)
//         .default_size([200.0, 300.0])
//         .show(ctx, |ui| {
//             // Match on the SelectedEntity enum
//             if let Some(SelectedEntity::Province(entity)) = selection.entity {
//                 if let Ok((province, owner)) = provinces.get(entity) {
//                     ui.label(format!("ID: {}", province.id));
//                     ui.label(format!("Terrain: {:?}", province.terrain));
//                     ui.label(format!("Center: {:#}", province.center));
//
//                     if let Some(owner) = owner {
//                         if let Ok(country) = countries.get(owner.owner) {
//                             ui.label(format!("Owner: {}", country.name));
//                         } else {
//                             ui.label("Owner: Unknown");
//                         }
//                     } else {
//                         ui.label("Owner: None");
//                     }
//
//                     ui.separator();
//                     ui.label(format!("Neighbors: {}", province.neighbors.len()));
//                 } else {
//                     ui.label("Invalid province selection");
//                 }
//             } else {
//                 ui.label("No province selected");
//             }
//         });
// }
