// plugins/country_selection_ui.rs
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use crate::components::country::Country;
use crate::components::player::{Player, ControlsCountry, LocalPlayer};
use crate::states::AppState;

pub struct CountrySelectionUI;

impl Plugin for CountrySelectionUI {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::CountrySelection), setup_country_selection)
            .add_systems(OnExit(AppState::CountrySelection), cleanup_countries)
            .add_systems(
                EguiPrimaryContextPass,
                country_selection_ui
                    .run_if(in_state(AppState::CountrySelection))
            );
    }
}

fn setup_country_selection() {}

fn cleanup_countries(
    mut commands: Commands,
    countries: Query<Entity, With<Country>>,
) {
    for entity in countries.iter() {
        commands.entity(entity).despawn();
    }
    println!("Cleaned up {} countries", countries.iter().count());
}

fn country_selection_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    countries: Query<(Entity, &Country)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Select Your Country");
            ui.add_space(20.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (country_entity, country) in countries.iter() {
                    ui.horizontal(|ui| {
                        // Show country color as a small square
                        let color = country.color;
                        let color32 = egui::Color32::from_rgb(
                            (color.to_srgba().red * 255.0) as u8,
                            (color.to_srgba().green * 255.0) as u8,
                            (color.to_srgba().blue * 255.0) as u8,
                        );
                        ui.colored_label(color32, "■");

                        if ui.button(&country.name).clicked() {
                            // Create player entity
                            let player_entity = commands.spawn((
                                Player {
                                    id: 0,
                                    name: "Player 1".to_string(),
                                },
                                ControlsCountry(country_entity),
                            )).id();

                            // Store as local player
                            commands.insert_resource(LocalPlayer(player_entity));

                            println!("Player selected: {}", country.name);
                            next_state.set(AppState::InGame);
                        }
                    });
                }
            });

            ui.add_space(20.0);
            if ui.button("Back").clicked() {
                next_state.set(AppState::InMainMenu);
            }
        });
    });
}