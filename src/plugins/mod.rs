mod camera_controls;
mod game_systems;
mod main_menu_ui;
mod map_generation;
mod province_info_ui;
mod province_visuals;
mod selection;
mod setup_egui_camera;

pub use camera_controls::GameCamera;
pub use game_systems::GameSystems;
pub use main_menu_ui::MainMenu;
pub use map_generation::setup_map;
pub use province_info_ui::ProvinceInfoUI;
pub use province_visuals::ProvinceVisuals;
pub use selection::SelectionPlugin;
pub use setup_egui_camera::SetupEguiCamera;
