//! Gameplay HUD — pilot name, rank, gold, stamina, airport/zone, time, weather.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HudText;

pub fn spawn_hud(mut commands: Commands, font: Res<UiFontHandle>) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                top: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            for _ in 0..7 {
                parent.spawn((
                    HudText,
                    Text::new(""),
                    text_style.clone(),
                    TextColor(Color::WHITE),
                ));
            }
        });
}

pub fn update_hud(
    pilot: Res<PilotState>,
    gold: Res<Gold>,
    location: Res<PlayerLocation>,
    calendar: Res<Calendar>,
    weather: Res<WeatherState>,
    query: Query<Entity, With<HudRoot>>,
    children_q: Query<&Children>,
    mut text_q: Query<&mut Text, With<HudText>>,
) {
    let Ok(root) = query.get_single() else { return };
    let Ok(children) = children_q.get(root) else { return };

    let lines = [
        format!("{} — {}", pilot.name, pilot.rank.display_name()),
        format!("Gold: {}", gold.amount),
        format!(
            "Stamina: {}/{}",
            pilot.stamina as u32, pilot.max_stamina as u32
        ),
        format!(
            "{:?} — {:?}",
            location.airport.display_name(),
            location.zone
        ),
        calendar.formatted_time(),
        calendar.formatted_date(),
        format!("Weather: {:?}", weather.current),
    ];

    for (i, child) in children.iter().enumerate() {
        if i < lines.len() {
            if let Ok(mut text) = text_q.get_mut(*child) {
                **text = lines[i].clone();
            }
        }
    }
}

pub fn despawn_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
