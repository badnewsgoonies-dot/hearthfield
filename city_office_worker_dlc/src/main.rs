use bevy::prelude::*;

mod game;

fn main() {
    // Key controls:
    // P -> process one inbox item (costs energy, +15 minutes)
    // C -> take a coffee break (restore energy, +20 minutes)
    // N -> wait in place (+10 minutes)
    // I -> interruption (+stress, -focus, +12 minutes)
    // 1 -> resolve interruption calmly (+focus, -stress)
    // 2 -> panic response (-focus, +stress)
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "City Office Worker DLC Prototype".to_string(),
                resolution: (960.0, 540.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(game::CityOfficeWorkerPlugin)
        .run();
}
