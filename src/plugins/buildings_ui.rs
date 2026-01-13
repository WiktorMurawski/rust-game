use crate::components::country::*;
use crate::components::province::*;
use crate::plugins::selection::SelectedEntity;
use crate::plugins::selection::CurrentSelection;
use crate::states::AppState;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use crate::components::buildings::{Buildings, ALL_BUILDINGS};
use crate::components::player::{ControlsCountry, LocalPlayer};

pub struct BuildingsUI;

impl Plugin for BuildingsUI {
    fn build(&self, app: &mut App) {
        app.add_systems(
                EguiPrimaryContextPass,
                province_building_ui.run_if(in_state(AppState::InGame)),
            );
    }
}

fn province_building_ui(
    mut contexts: EguiContexts,
    selected: Res<CurrentSelection>,
    mut provinces: Query<(&OwnedBy, &mut Buildings)>,
    mut countries: Query<&mut Country>,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
) {
    // println!("province_building_ui");

    let Ok(ctx) = contexts.ctx_mut() else { return; };

    if let Some(SelectedEntity::Province(province_entity)) = selected.entity {
        let Ok((owned_by, mut buildings)) = provinces.get_mut(province_entity) else { return; };

        // Check if player owns this province
        let player_country_entity = local_player
            .and_then(|lp| player_query.get(lp.0).ok())
            .map(|controls| controls.0);

        if Some(owned_by.0) != player_country_entity {
            return;  // Not owned by player
        }

        // Get player's country for gold
        let Ok(mut player_country) = countries.get_mut(owned_by.0) else { return; };

        egui::Window::new("Build in Province")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Available Buildings:");
                for &building_type in &ALL_BUILDINGS {
                    if buildings.built.contains(&building_type) {
                        ui.label(format!("{} (Already Built)", building_type.name()));
                        continue;
                    }

                    let cost = building_type.cost();
                    let can_afford = player_country.gold >= cost;

                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}: {} gold - {}",
                            building_type.name(),
                            cost,
                            building_type.description()
                        ));
                        if ui.button("Build").clicked() && can_afford {
                            player_country.gold -= cost;
                            buildings.built.push(building_type);
                        }
                    });
                }
            });
    }
}