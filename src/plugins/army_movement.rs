use crate::components::army::{Army, ArmySelected};
use crate::components::province::{OwnedBy, Province};
use crate::misc::*;
use crate::plugins::selection::CurrentSelection;
use crate::states::AppState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

pub struct ArmyMovementPlugin;

impl Plugin for ArmyMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_army.run_if(in_state(AppState::InGame)));
    }
}

fn move_army(
    current_selection: Res<CurrentSelection>,
    mut armies: Query<&mut Army, With<ArmySelected>>,
    provinces: Query<&Province>,
    owned_by: Query<&OwnedBy>,
    mouse_and_window_and_camera: MouseAndWindowAndCamera,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().unwrap().wants_pointer_input() {
        return;
    }

    let mouse_buttons = mouse_and_window_and_camera.mouse;

    // if mouse_buttons.just_pressed(MouseButton::Right) {
    //     if let Some(SelectedEntity::Army(army_entity)) = current_selection.entity {
    //         let mut army = armies.get_mut(army_entity).unwrap();
    //         if let Some(mouse_pos) = mouse_to_world_coords(...) {  // Your fn
    //             // Find closest province (like your selection)
    //             let target_province = // ... logic to get target Entity
    //             let current_prov = provinces.get(army.province).unwrap();
    //             if current_prov.neighbors.contains(&target_prov.id) {
    //                 // Optional: Check if owned or neutral/enemy
    //                 army.province = target_province;
    //                 // current_selection.entity = None;
    //             }
    //         }
    //     }
    // }
}
