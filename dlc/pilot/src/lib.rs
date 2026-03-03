//! Skywarden — Pilot Life Simulator
//!
//! Built on Bevy 0.15, using the same plugin-per-domain architecture as Hearthfield.

mod shared;
pub mod input;
pub mod player;
pub mod flight;
pub mod aircraft;
pub mod airports;
pub mod missions;
pub mod crew;
pub mod weather;
pub mod economy;
pub mod ui;
pub mod save;
pub mod data;
pub mod world;

pub use shared::*;
