#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::needless_pass_by_value)] // because clippy produces false positives for Bevy systems

pub mod components;
pub mod misc;
pub mod plugins;
pub mod resources;
pub mod states;
