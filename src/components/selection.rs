use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Selection {
    pub selected: Option<SelectedEntity>,
}

pub enum SelectedEntity {
    Province(Entity),
    Army(Entity),
    Country(Entity),
}
