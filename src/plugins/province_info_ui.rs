use crate::components::buildings::Buildings;
use crate::components::country::*;
use crate::components::province::*;
use crate::plugins::selection::CurrentSelection;
use crate::plugins::selection::SelectedEntity;
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

                    if let Some(owner) = owner_opt {
                        if let Ok(country) = countries.get(owner.owner) {
                            ui.label(format!("Owner: {}", country.name));
                        } else {
                            ui.label("Owner: Unknown");
                        }
                    } else {
                        ui.label("Owner: None (unclaimed)");
                    }

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

                    let mut province_growth = province.base_growth;

                    for &building in &buildings.built {
                        province_growth += building.growth_bonus();
                    }

                    let is_occupied = occupied_opt.is_some();
                    if is_occupied {
                        province_growth = -0.05;
                    }

                    let growth_amount =
                        (province.population as f32 * province_growth).round() as i32;

                    ui.label(format!("Population: {}", province.population));
                    ui.label(format!(
                        "Growth per turn: {}% ({} people)",
                        province_growth * 100.0,
                        growth_amount,
                        //(province.population as f32 * province.base_growth).round() as i32
                    ));

                    if !buildings.built.is_empty() {
                        ui.label("Buildings:");
                        for &b in &buildings.built {
                            ui.label(format!("• {}", b.name()));
                        }
                    } else {
                        ui.label("No buildings");
                    }

                    ui.separator();

                    let mut income = province.base_income as f32;

                    for &building in &buildings.built {
                        income += building.income_bonus() as f32;
                    }

                    income += (province.population / 1000) as f32;

                    if occupied_opt.is_some() {
                        income *= 0.6;
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
