//! Screen fade overlay — reads ScreenFadeEvent, animates a full-screen black rect.

use crate::shared::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct FadeOverlay {
    pub fade_in: bool,
    pub duration: f32,
    pub elapsed: f32,
}

pub fn handle_screen_fade(
    mut commands: Commands,
    mut events: EventReader<ScreenFadeEvent>,
    existing: Query<Entity, With<FadeOverlay>>,
) {
    for evt in events.read() {
        // Remove any existing overlay before spawning a new one
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }

        let initial_alpha = if evt.fade_in { 1.0 } else { 0.0 };

        commands.spawn((
            FadeOverlay {
                fade_in: evt.fade_in,
                duration: evt.duration_secs,
                elapsed: 0.0,
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, initial_alpha)),
            GlobalZIndex(999),
        ));
    }
}

pub fn update_screen_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FadeOverlay, &mut BackgroundColor)>,
) {
    for (entity, mut fade, mut bg) in &mut query {
        fade.elapsed += time.delta_secs();
        let t = (fade.elapsed / fade.duration).clamp(0.0, 1.0);

        let alpha = if fade.fade_in { 1.0 - t } else { t };
        bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);

        if t >= 1.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
