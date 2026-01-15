use crate::states::{AppState, GamePhase};
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct EndTurnUI;

impl Plugin for EndTurnUI {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            end_turn_ui.run_if(in_state(AppState::InGame)),
        );
    }
}

fn end_turn_ui(
    mut contexts: EguiContexts,
    game_phase: Res<State<GamePhase>>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::Window::new("Turn Controls")
        .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])  // padding from right/bottom edges
        .resizable(false)
        .collapsible(false)
        .title_bar(true)
        .default_size([180.0, 80.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let phase_text = match game_phase.get() {
                    GamePhase::PlayerTurn => egui::RichText::new("Your Turn").color(egui::Color32::LIGHT_GREEN),
                    GamePhase::Processing => egui::RichText::new("Processing...").color(egui::Color32::LIGHT_BLUE),
                };
                ui.label(phase_text);

                ui.add_space(8.0);

                if *game_phase.get() == GamePhase::PlayerTurn {
                    let button_response = ui.button(egui::RichText::new("End Turn").size(18.0));

                    if button_response.clicked() {
                        next_phase.set(GamePhase::Processing);
                    }
                } else {
                    ui.label("Waiting...");
                }
            });
        });
}