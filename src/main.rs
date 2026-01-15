use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};
use rust_game::plugins::GameSystems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(GameSystems)
        .add_plugins(Text3dPlugin {
            default_atlas_dimension: (1024, 1024),
            load_system_fonts: true,
            ..Default::default()
        })
        .insert_resource(LoadFonts {
            font_paths: vec!["assets/fonts/roboto.ttf".to_owned()],
            font_directories: vec!["assets/fonts".to_owned()],
            ..Default::default()
        })
        .run();
}
