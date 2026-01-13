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
            (
                player_country_panel_with_flag.run_if(in_state(AppState::InGame)),
                player_wealth_ui.run_if(in_state(AppState::InGame)),
            ),
        );
    }
}

fn player_wealth_ui(
    mut contexts: EguiContexts,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
    countries: Query<&Country>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let player_country = local_player
        .and_then(|lp| player_query.get(lp.0).ok())
        .and_then(|controls| countries.get(controls.0).ok());

    if let Some(country) = player_country {
        egui::Window::new("player_wealth")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!("Gold: {}", country.gold))
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
            });
    }
}

fn player_country_panel_with_flag(
    mut contexts: EguiContexts,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&ControlsCountry>,
    countries: Query<&Country>,
) {
    let player_country = local_player
        .and_then(|lp| player_query.get(lp.0).ok())
        .and_then(|controls| countries.get(controls.0).ok());

    let mut texture_id_opt = None;

    if let Some(country) = player_country {
        if let Some(flag_handle) = &country.flag {
            texture_id_opt =
                Some(contexts.add_image(bevy_egui::EguiTextureHandle::Strong(flag_handle.clone())));
        }

        let Ok(ctx) = contexts.ctx_mut() else {
            return;
        };

        // Now open the window
        egui::Window::new("player_country")
            .title_bar(false)
            .resizable(false)
            .fixed_pos([10.0, 10.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(texture_id) = texture_id_opt {
                        ui.image(egui::load::SizedTexture::new(
                            texture_id,
                            egui::vec2(40.0, 30.0),
                        ));
                    } else {
                        show_color_square(ui, country.color);
                    }

                    ui.add_space(10.0);

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

fn show_color_square(ui: &mut egui::Ui, color: Color) {
    let color32 = egui::Color32::from_rgb(
        (color.to_srgba().red * 255.0) as u8,
        (color.to_srgba().green * 255.0) as u8,
        (color.to_srgba().blue * 255.0) as u8,
    );

    let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 30.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, color32);
}
