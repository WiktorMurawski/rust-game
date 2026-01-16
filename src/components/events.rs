// components/events.rs
use bevy::prelude::*;
use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    ArmyDesertion,
    TaxRevolt,
}

#[derive(Debug, Clone)]
pub struct EventOption {
    pub description: String,
    pub effect: EventEffect,
}

#[derive(Debug, Clone)]
pub enum EventEffect {
    PayGold(u32),
    LoseArmyUnits(f32),
    LoseGold(u32),
    GainGold(u32),
    LosePopulation(f32),
}

#[derive(Debug, Clone)]
pub struct GameEvent {
    pub event_type: EventType,
    pub title: String,
    pub description: String,
    pub options: Vec<EventOption>,
}

impl GameEvent {
    pub fn generate_random() -> Self {
        let mut rng = rand::rng();
        let event_type = if rng.random_bool(0.5) {
            EventType::ArmyDesertion
        } else {
            EventType::TaxRevolt
        };

        match event_type {
            EventType::ArmyDesertion => Self {
                event_type,
                title: "Army Unrest".to_string(),
                description: "Your armies grow restless without pay. The soldiers threaten to desert if their demands are not met.".to_string(),
                options: vec![
                    EventOption {
                        description: "Pay them 2000 gold to maintain morale".to_string(),
                        effect: EventEffect::PayGold(2000),
                    },
                    EventOption {
                        description: "Refuse their demands (desertion of 10% of all units)".to_string(),
                        effect: EventEffect::LoseArmyUnits(0.1),
                    },
                ],
            },
            EventType::TaxRevolt => Self {
                event_type,
                title: "Tax Revolt".to_string(),
                description: "The people are angry about high taxes. Protests break out in your provinces.".to_string(),
                options: vec![
                    EventOption {
                        description: "Lower taxes and compensate (-1500 gold)".to_string(),
                        effect: EventEffect::LoseGold(1500),
                    },
                    EventOption {
                        description: "Crack down on protests (lose 5% population)".to_string(),
                        effect: EventEffect::LosePopulation(0.05),
                    },
                ],
            },
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PendingEvent {
    pub event: GameEvent,
}

impl PendingEvent {
    pub fn new(event: GameEvent) -> Self {
        Self { event }
    }
}
