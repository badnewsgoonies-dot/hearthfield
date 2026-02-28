//! Hearthfield library crate — re-exports all modules for integration testing.
//!
//! The binary crate (`main.rs`) is the actual game entry point.
//! This library crate exposes the same modules so that `tests/` integration
//! tests can import game types, systems, and resources without needing a
//! window or GPU.

pub mod shared;
pub mod input;
pub mod calendar;
pub mod player;
pub mod farming;
pub mod animals;
pub mod world;
pub mod npcs;
pub mod economy;
pub mod crafting;
pub mod fishing;
pub mod mining;
pub mod ui;
pub mod save;
pub mod data;
