use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::states::AppState;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass, // ← Changed from Update!
            main_menu_ui.run_if(in_state(AppState::InMainMenu)),
        );
    }
}

fn main_menu_ui(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) -> Result {
    // ← Must return Result!
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        // ← Use ?
        ui.vertical_centered(|ui| {
            ui.heading("Grand Strategy Game");
            ui.add_space(20.0);

            if ui.button("New Game").clicked() {
                next_state.set(AppState::InGame);
            }

            if ui.button("Load Game").clicked() {
                todo!();
            }

            if ui.button("Settings").clicked() {
                todo!();
            }

            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    });
    Ok(()) // ← Return Ok!
}
