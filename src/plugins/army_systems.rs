// plugins/army_systems.rs
use crate::components::army::Army;
use crate::states::AppState;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct ArmySystemsPlugin;
impl Plugin for ArmySystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            merge_armies_in_same_province.run_if(in_state(AppState::InGame)),
        );
    }
}

fn merge_armies_in_same_province(mut commands: Commands, armies: Query<(Entity, &Army)>) {
    let mut province_armies: HashMap<(Entity, Entity), Vec<(Entity, u32)>> = HashMap::new();
    for (entity, army) in &armies {
        province_armies
            .entry((army.owner, army.province))
            .or_default()
            .push((entity, army.units));
    }

    for ((owner, province), mut army_list) in province_armies {
        if army_list.len() > 1 {
            army_list.sort_by_key(|(e, _)| *e);

            let (keep_entity, _) = army_list[0];
            let total_units: u32 = army_list.iter().map(|(_, units)| units).sum();

            if let Ok(mut army) = commands.get_entity(keep_entity) {
                army.insert(Army {
                    owner,
                    province,
                    units: total_units,
                });
            }

            for (entity, _) in army_list.iter().skip(1) {
                commands.entity(*entity).despawn();
            }
        }
    }
}
