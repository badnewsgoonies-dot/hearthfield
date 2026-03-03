//! Toast notification system — auto-despawning text popups.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct ToastDisplay {
    pub timer: Timer,
}

pub fn show_toast(
    mut commands: Commands,
    mut events: EventReader<ToastEvent>,
    font: Res<UiFontHandle>,
) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 16.0,
        ..default()
    };

    for evt in events.read() {
        commands.spawn((
            ToastDisplay {
                timer: Timer::from_seconds(evt.duration_secs, TimerMode::Once),
            },
            Text::new(evt.message.clone()),
            text_style.clone(),
            TextColor(Color::srgb(1.0, 1.0, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Percent(50.0),
                ..default()
            },
        ));
    }
}

pub fn update_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ToastDisplay)>,
) {
    for (entity, mut toast) in &mut query {
        toast.timer.tick(time.delta());
        if toast.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
