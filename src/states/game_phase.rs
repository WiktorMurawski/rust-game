use bevy::prelude::*;

// Global game phase
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GamePhase {
    #[default]
    PlayerTurn,     // Player can select & give orders
    Processing,     // Execute movements, resolve combats, etc. (for now just moves)
    // Later: EnemyTurn, EndOfTurn, etc.
}

// Per-army "has this army acted this turn?"
#[derive(Component, Default)]
pub struct HasActedThisTurn;

// Resource to track pending moves
#[derive(Resource, Default)]
pub struct PendingMoves {
    pub moves: Vec<(Entity, Entity)>, // (army, target_province)
}