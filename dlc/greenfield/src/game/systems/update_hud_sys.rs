use bevy::prelude::*;
use crate::game::components::PlayerMarker;

pub fn update_hud_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    *spawned = true;
    commands.spawn(Camera2d::default());
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.3),
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
        Transform::default(),
        PlayerMarker,
    ));
}
