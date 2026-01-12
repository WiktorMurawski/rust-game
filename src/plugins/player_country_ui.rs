// plugins/ui/player_country_ui.rs
use crate::components::country::Country;
use crate::components::player::{ControlsCountry, LocalPlayer};
use crate::states::AppState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct PlayerCountryUI;

impl Plugin for PlayerCountryUI {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            player_country_panel.run_if(in_state(AppState::InGame)),
        );
    }
}

fn player_country_panel(
    mut contexts: EguiContexts,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
    countries: Query<&Country>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Get the player's country
    let player_country = local_player
        .and_then(|lp| player_query.get(lp.0).ok())
        .and_then(|controls| countries.get(controls.0).ok());

    println!("Player country: {:?}", player_country);

    if let Some(country) = player_country {
        // Create a panel in the upper left
        egui::Window::new("player_country")
            .title_bar(false)
            .resizable(false)
            .fixed_pos([10.0, 10.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Country flag/color indicator
                    let color = country.color;
                    let color32 = egui::Color32::from_rgb(
                        (color.to_srgba().red * 255.0) as u8,
                        (color.to_srgba().green * 255.0) as u8,
                        (color.to_srgba().blue * 255.0) as u8,
                    );

                    // Large colored square
                    let (rect, _response) =
                        ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color32);

                    ui.add_space(10.0);

                    // Country name
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&country.name).heading().strong());
                        ui.label(
                            egui::RichText::new(format!(
                                "{} provinces",
                                country.owned_provinces.len()
                            ))
                            .small()
                            .color(egui::Color32::GRAY),
                        );
                    });
                });
            });
    }
}
