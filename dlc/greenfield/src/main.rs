//! Greenfield DLC — Hearthfield sibling crate.
//!
//! Tower-defense farming survival arena. Tend crops, defend them
//! from critters that drift in to eat them.
//!
//! Build & run:
//!   cargo run -p greenfield_dlc

use bevy::prelude::*;

mod game;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.10, 0.16, 0.10)))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Greenfield — Hearthfield DLC".to_string(),
                    resolution: (960.0, 540.0).into(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(game::GreenfieldPlugin)
        .run();
}
