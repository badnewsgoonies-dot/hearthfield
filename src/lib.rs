//! Hearthfield library crate — re-exports all modules for integration testing.
//!
//! The binary crate (`main.rs`) is the actual game entry point.
//! This library crate exposes the same modules so that `tests/` integration
//! tests can import game types, systems, and resources without needing a
//! window or GPU.

pub mod animals;
pub mod calendar;
pub mod crafting;
pub mod data;
pub mod economy;
pub mod farming;
pub mod fishing;
pub mod input;
pub mod mining;
pub mod npcs;
pub mod player;
pub mod save;
pub mod shared;
pub mod ui;
pub mod world;
