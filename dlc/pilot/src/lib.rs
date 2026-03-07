//! Skywarden — Pilot Life Simulator
//!
//! Built on Bevy 0.15, using the same plugin-per-domain architecture as Hearthfield.

#![allow(dead_code, unused_imports, clippy::upper_case_acronyms)]

pub mod aircraft;
pub mod airports;
pub mod crew;
pub mod data;
pub mod economy;
pub mod flight;
pub mod input;
pub mod missions;
pub mod player;
pub mod save;
mod shared;
pub mod ui;
pub mod weather;
pub mod world;

pub use shared::*;
