// plugins/buildings_ui.rs
use crate::components::army::Army;
use crate::components::buildings::{ALL_BUILDINGS, BuildingType, Buildings};
use crate::components::country::*;
use crate::components::player::{ControlsCountry, LocalPlayer};
use crate::components::province::*;
use crate::plugins::selection::CurrentSelection;
use crate::plugins::selection::SelectedEntity;
use crate::states::AppState;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

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
    mut commands: Commands,
    mut contexts: EguiContexts,
    selected: Res<CurrentSelection>,
    mut provinces: Query<(&Province, &OwnedBy, &mut Buildings)>,
    mut countries: Query<&mut Country>,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if let Some(SelectedEntity::Province(province_entity)) = selected.entity {
        let Ok((province, owned_by, mut buildings)) = provinces.get_mut(province_entity) else {
            return;
        };

        let player_country_entity = local_player
            .and_then(|lp| player_query.get(lp.0).ok())
            .map(|controls| controls.0);

        if Some(owned_by.owner) != player_country_entity {
            return;
        }

        let Ok(mut player_country) = countries.get_mut(owned_by.owner) else {
            return;
        };

        egui::Window::new("Build in Province")
            .resizable(false)
            .anchor(egui::Align2::LEFT_BOTTOM, [280.0, -10.0])
            .pivot(egui::Align2::LEFT_BOTTOM)
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

                if buildings.built.contains(&BuildingType::Barracks) {
                    ui.separator();
                    ui.label("Recruitment:");
                    if ui.button("Recruit Army (Cost: 100 gold)").clicked()
                        && player_country.gold >= 100
                    {
                        player_country.gold -= 100;

                        let _army_entity = commands
                            .spawn((
                                Army {
                                    owner: owned_by.owner,
                                    province: province_entity,
                                    units: 100,
                                },
                                Transform::from_xyz(province.center.x, 0.0, province.center.y),
                                GlobalTransform::default(),
                                Visibility::Visible,
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ))
                            .id();
                    }
                }
            });
    }
}
