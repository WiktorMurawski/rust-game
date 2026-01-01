use bevy::prelude::*;
use rust_game::plugins::CameraControls;
use rust_game::plugins::GameSystems;
use rust_game::states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_plugins(GameSystems)
        .add_plugins(CameraControls)
        .run();
}
