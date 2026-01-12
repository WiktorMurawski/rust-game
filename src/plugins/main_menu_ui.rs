use crate::plugins::save_load::{SaveFilePath, SaveLoadError};
use crate::states::AppState;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            main_menu_ui.run_if(in_state(AppState::InMainMenu)),
        );
    }
}

fn main_menu_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut error: ResMut<SaveLoadError>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let error_msg = error.message.clone();

    if let Some(msg) = &error_msg {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(msg);
                ui.add_space(10.0);
                if ui.button("OK").clicked() {
                    error.message = None;
                }
            });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Grand Strategy Game");
            ui.add_space(20.0);

            let new_game_button = ui.button("New Game");
            let load_game_button = ui.button("Load Game");
            let quit_button = ui.button("Quit");

            if new_game_button.clicked() {
                next_state.set(AppState::LoadingNewGame);
            }

            if load_game_button.clicked() {
                commands.insert_resource(SaveFilePath("saves/quicksave.ron".to_string()));
                next_state.set(AppState::LoadingSavedGame);
            }

            if quit_button.clicked() {
                std::process::exit(0);
            }
        });
    });

    Ok(())
}
