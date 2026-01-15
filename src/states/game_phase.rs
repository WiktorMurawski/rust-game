use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GamePhase {
    #[default]
    PlayerTurn,
    Processing,
}

#[derive(Resource, Default)]
pub struct PendingMoves {
    pub moves: Vec<(Entity, Entity)>,
}
