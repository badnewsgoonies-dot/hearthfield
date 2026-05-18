use bevy::prelude::*;
use crate::game::components::{PlayerMarker, HudRoot, HudTimer};
use crate::game::resources::{GameScore, LevelProgress, PlayerHealth};

pub fn update_hud_system(mut commands: Commands, mut spawned: Local<bool>, score: Res<GameScore>, level: Res<LevelProgress>, health: Res<PlayerHealth>, mut hud_query: Query<&mut Text, With<HudTimer>>) {
    if !*spawned {
        *spawned = true;
        commands.spawn(Camera2d::default());
        // 20x15 checkerboard background grid (40px tiles)
        for tx in -10..10 {
            for ty in -7..8 {
                let is_dark = (tx + ty) & 1 == 0;
                let shade = if is_dark { 0.08 } else { 0.14 };
                commands.spawn((
                    Sprite {
                        color: Color::srgb(shade, shade + 0.04, shade),
                        custom_size: Some(Vec2::splat(40.0)),
                        ..default()
                    },
                    Transform::from_xyz(tx as f32 * 40.0, ty as f32 * 40.0, -1.0),
                ));
            }
        }
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.8, 0.3),
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            },
            Transform::default(),
            PlayerMarker,
        ));
        commands.spawn((
            HudRoot,
            HudTimer,
            Text::new(""),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.95, 0.95, 0.8)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
        ));
    }
    for mut text in &mut hud_query {
        *text = Text::new(format!(
            "hp {:.0}/{:.0}  score {}/{}  level {} ({}xp)",
            health.hp, health.max_hp,
            score.total, score.high,
            level.level, level.xp,
        ));
    }
}
