use std::cmp::PartialEq;
use crate::components::country::*;
use crate::components::province::*;
use crate::plugins::selection::Selected;
use crate::states::AppState;
use bevy::prelude::*;

pub struct ProvinceVisualsPlugin;

impl Plugin for ProvinceVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapMode>()
            .init_resource::<BordersVisible>()
            .add_systems(
                Update,
                (
                    toggle_map_mode,
                    toggle_borders,
                    update_province_colors,
                    update_changed_province_colors,
                    update_border_visibility,
                    update_selected_province_borders,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Resource, Debug, Default, PartialEq)]
enum MapMode {
    #[default]
    Political,
    Terrain,
}

#[derive(Resource)]
struct BordersVisible(bool);

impl Default for BordersVisible {
    fn default() -> Self {
        BordersVisible(true)
    }
}

fn toggle_map_mode(keyboard: Res<ButtonInput<KeyCode>>, mut map_mode: ResMut<MapMode>) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        *map_mode = match *map_mode {
            MapMode::Terrain => MapMode::Political,
            MapMode::Political => MapMode::Terrain,
        };
        println!("Map mode switched to: {:?}", *map_mode);
    }
}

type ProvinceQuery<'a> = (
    &'a Province,
    &'a MeshMaterial3d<StandardMaterial>,
    Option<&'a OwnedBy>,
);

// Update all provinces when map mode changes
fn update_province_colors(
    map_mode: Res<MapMode>,
    provinces: Query<ProvinceQuery>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    countries: Query<&Country>,
) {
    if !map_mode.is_changed() {
        return;
    }

    for (province, material_handle, owner) in provinces.iter() {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        let color = match *map_mode {
            MapMode::Terrain => province.terrain.color(),
            MapMode::Political => {
                if let Some(owner) = owner {
                    if let Ok(country) = countries.get(owner.0) {
                        country.color
                    } else {
                        Color::srgb(0.0, 0.0, 1.0)
                    }
                } else {
                    Color::srgb(0.0, 0.0, 1.0)
                }
            }
        };

        material.base_color = color;
    }
}

// Update only provinces that had ownership changes
fn update_changed_province_colors(
    map_mode: Res<MapMode>,
    changed_provinces: Query<ProvinceQuery>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    countries: Query<&Country>,
) {
    for (province, material_handle, owner) in changed_provinces.iter() {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        if province.terrain == TerrainType::Water{
            let color = province.terrain.color();
            material.base_color = color;
            continue;
        }

        let color = match *map_mode {
            MapMode::Terrain => province.terrain.color(),
            MapMode::Political => {
                if let Some(owner) = owner {
                    if let Ok(country) = countries.get(owner.0) {
                        country.color
                    } else {
                        Color::srgb(0.0, 0.0, 1.0)
                    }
                } else {
                    Color::srgb(0.0, 0.0, 1.0)
                }
            }
        };

        material.base_color = color;
    }
}

fn toggle_borders(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut borders_visible: ResMut<BordersVisible>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        borders_visible.0 = !borders_visible.0;
        println!("Borders: {}", if borders_visible.0 { "ON" } else { "OFF" });
    }
}

fn update_border_visibility(
    borders_visible: Res<BordersVisible>,
    mut borders: Query<&mut Visibility, With<ProvinceBorder>>,
) {
    if !borders_visible.is_changed() {
        return;
    }

    let visibility = if borders_visible.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut vis in borders.iter_mut() {
        *vis = visibility;
    }
}

fn update_selected_province_borders(
    // Provinces that just got Selected added
    newly_selected: Query<&Children, (With<Province>, Added<Selected>)>,
    // Provinces that exist and might be selected
    all_provinces: Query<(&Children, Has<Selected>), With<Province>>,
    mut removed: RemovedComponents<Selected>,
    borders: Query<&MeshMaterial3d<StandardMaterial>, With<ProvinceBorder>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Handle newly selected provinces
    for children in newly_selected.iter() {
        update_border_color(children, true, &borders, &mut materials);
    }

    // Handle deselected provinces
    for entity in removed.read() {
        // Entity might have been despawned, so check if it still exists
        if let Ok((children, _)) = all_provinces.get(entity) {
            update_border_color(children, false, &borders, &mut materials);
        }
    }
}

// Helper function to reduce duplication
fn update_border_color(
    children: &Children,
    selected: bool,
    borders: &Query<&MeshMaterial3d<StandardMaterial>, With<ProvinceBorder>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    for &child in children {
        if let Ok(border_material) = borders.get(child)
            && let Some(material) = materials.get_mut(&border_material.0)
        {
            material.base_color = if selected {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
            } else {
                Color::srgb(0.2, 0.2, 0.2) // Dark gray for normal
            };
        }
    }
}
