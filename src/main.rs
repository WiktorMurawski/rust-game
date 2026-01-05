use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use rust_game::plugins::GameSystems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(GameSystems)
        .run();
}
