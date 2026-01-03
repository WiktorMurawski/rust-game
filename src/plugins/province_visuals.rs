use crate::components::country::*;
use crate::components::province::*;
use crate::states::AppState;
use bevy::prelude::*;

pub struct ProvinceVisuals;

impl Plugin for ProvinceVisuals {
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
                    //update_selected_province_borders,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Resource, Debug, Default, PartialEq)]
pub enum MapMode {
    #[default]
    Terrain,
    Political,
}

#[derive(Resource, Default)]
pub struct BordersVisible(pub bool);

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
                        Color::srgb(0.5, 0.5, 0.5) // Unowned/neutral
                    }
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
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

        let color = match *map_mode {
            MapMode::Terrain => province.terrain.color(),
            MapMode::Political => {
                if let Some(owner) = owner {
                    if let Ok(country) = countries.get(owner.0) {
                        country.color
                    } else {
                        Color::srgb(0.5, 0.5, 0.5)
                    }
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
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

//fn update_selected_province_borders(
//    provinces: Query<(&Children, Option<&Selected>), (With<Province>, Changed<Selected>)>,
//    mut borders: Query<&MeshMaterial3d<StandardMaterial>, With<ProvinceBorder>>,
//    mut materials: ResMut<Assets<StandardMaterial>>,
//) {
//    for (children, selected) in provinces.iter() {
//        // Find the border child
//        for &child in children.iter() {
//            if let Ok(border_material) = borders.get(child) {
//                if let Some(material) = materials.get_mut(&border_material.0) {
//                    material.base_color = if selected.is_some() {
//                        Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
//                    } else {
//                        Color::srgb(0.2, 0.2, 0.2) // Dark gray for normal
//                    };
//                }
//            }
//        }
//    }
//}
