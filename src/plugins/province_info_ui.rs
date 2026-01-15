use crate::components::country::*;
use crate::components::province::*;
use crate::plugins::selection::SelectedEntity;
use crate::plugins::selection::{CurrentSelection, Selected};
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
    provinces: Query<(&Province, Option<&OwnedBy>), With<Selected>>,
    countries: Query<&Country>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Province Info")
        .default_pos([10.0, 10.0])
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .resizable(false)
        .default_size([200.0, 300.0])
        .show(ctx, |ui| {
            // Match on the SelectedEntity enum
            if let Some(SelectedEntity::Province(entity)) = selection.entity {
                if let Ok((province, owner)) = provinces.get(entity) {
                    ui.label(format!("ID: {}", province.id));
                    ui.label(format!("Terrain: {:?}", province.terrain));
                    ui.label(format!("Center: {:#}", province.center));

                    if let Some(owner) = owner {
                        if let Ok(country) = countries.get(owner.owner) {
                            ui.label(format!("Owner: {}", country.name));
                        } else {
                            ui.label("Owner: Unknown");
                        }
                    } else {
                        ui.label("Owner: None");
                    }

                    ui.separator();
                    ui.label(format!("Neighbors: {}", province.neighbors.len()));
                } else {
                    ui.label("Invalid province selection");
                }
            } else {
                ui.label("No province selected");
            }
        });
}
