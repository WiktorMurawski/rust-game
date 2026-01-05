use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use crate::{plugins::game_systems::MapSetup, resources::MapSize, states::AppState};

pub struct Lighting;

impl Plugin for Lighting {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_lighting.after(MapSetup));
    }
}

fn setup_lighting(mut commands: Commands, map_size: Res<MapSize>) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.9),
            illuminance: 1500.0,
            shadows_enabled: true,
            //shadow_depth_bias: 0.02,
            //shadow_normal_bias: 1.8,
            ..default()
        },
        Transform::from_xyz(map_size.0.x, 100.0, -map_size.0.y)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            minimum_distance: 1.0,
            maximum_distance: 2000.0,
            first_cascade_far_bound: 2000.0,
            overlap_proportion: 0.3,
        }
        .build(),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.7, 0.9),
        brightness: 300.0,
        ..default()
    });

    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
}
