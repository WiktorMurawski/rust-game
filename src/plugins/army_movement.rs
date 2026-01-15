use bevy::prelude::*;

// use crate::states::AppState;
// use crate::components::army::{Army, ArmySelected, MovementOrder};
// use crate::components::province::{OwnedBy, Province};
// use crate::misc::*;
// use crate::plugins::selection::{CurrentSelection, SelectedEntity};
// use bevy::ecs::system::SystemParam;
// use bevy::window::PrimaryWindow;
// use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
// use crate::components::player::{ControlsCountry, LocalPlayer};

pub struct ArmyMovementPlugin;

impl Plugin for ArmyMovementPlugin {
    fn build(&self, app: &mut App) {
        // app
            // .add_systems(Update, move_army.run_if(in_state(AppState::InGame)))
            // .add_systems(EguiPrimaryContextPass, army_movement_ui.run_if(in_state(AppState::InGame)));
    }
}

// fn army_movement_ui(
//     mut commands: Commands,
//     mut contexts: EguiContexts,
//     selection: Res<CurrentSelection>,
//     armies: Query<(&Army, Option<&MovementOrder>)>,
//     provinces: Query<&Province>,
//     local_player: Option<Res<LocalPlayer>>,
//     player_query: Query<&ControlsCountry>,
// ) {
//     let Ok(ctx) = contexts.ctx_mut() else { return; };
//
//     if let Some(SelectedEntity::Army(army_entity)) = selection.entity {
//         if let Ok((army, movement_order)) = armies.get(army_entity) {
//             let player_country_entity = local_player
//                 .and_then(|lp| player_query.get(lp.0).ok())
//                 .map(|controls| controls.0);
//
//             if Some(army.owner) != player_country_entity {
//                 return;
//             }
//
//             let Ok(current_province) = provinces.get(army.province) else { return; };
//
//             egui::Window::new("Army Orders")
//                 .show(ctx, |ui| {
//                     ui.label(format!("Army: {} units", army.units));
//                     ui.label(format!("Location: Province {}", current_province.id));
//
//                     // Show current order if any
//                     if let Some(order) = movement_order {
//                         if let Ok(target_prov) = provinces.get(order.target_province) {
//                             ui.colored_label(
//                                 egui::Color32::YELLOW,
//                                 format!("→ Moving to Province {}", target_prov.id)
//                             );
//                             if ui.button("Cancel Order").clicked() {
//                                 commands.entity(army_entity).remove::<MovementOrder>();
//                             }
//                         }
//                     } else {
//                         ui.separator();
//                         ui.label("Move to neighbor:");
//
//                         for &neighbor_id in &current_province.neighbors {
//                             if ui.button(format!("Province {}", neighbor_id)).clicked() {
//                                 // Find neighbor entity
//                                 if let Some((neighbor_entity, _)) = provinces.iter()
//                                     .find(|(_, p)| p.id == neighbor_id)
//                                 {
//                                     // Add movement order
//                                     commands.entity(army_entity).insert(MovementOrder {
//                                         target_province: neighbor_entity,
//                                     });
//                                 }
//                             }
//                         }
//                     }
//                 });
//         }
//     }
// }

// fn move_army(
//     current_selection: Res<CurrentSelection>,
//     mut armies: Query<&mut Army, With<ArmySelected>>,
//     provinces: Query<&Province>,
//     owned_by: Query<&OwnedBy>,
//     mouse_and_window_and_camera: MouseAndWindowAndCamera,
//     mut contexts: EguiContexts,
// ) {
//     if contexts.ctx_mut().unwrap().wants_pointer_input() {
//         return;
//     }
//
//     let mouse_buttons = mouse_and_window_and_camera.mouse;
//
//     // if mouse_buttons.just_pressed(MouseButton::Right) {
//     //     if let Some(SelectedEntity::Army(army_entity)) = current_selection.entity {
//     //         let mut army = armies.get_mut(army_entity).unwrap();
//     //         if let Some(mouse_pos) = mouse_to_world_coords(...) {  // Your fn
//     //             // Find closest province (like your selection)
//     //             let target_province = // ... logic to get target Entity
//     //             let current_prov = provinces.get(army.province).unwrap();
//     //             if current_prov.neighbors.contains(&target_prov.id) {
//     //                 // Optional: Check if owned or neutral/enemy
//     //                 army.province = target_province;
//     //                 // current_selection.entity = None;
//     //             }
//     //         }
//     //     }
//     // }
// }
