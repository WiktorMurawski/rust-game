use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use rust_game::plugins::GameSystems;
use rust_game::states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_plugins(GameSystems)
        .run();
}
