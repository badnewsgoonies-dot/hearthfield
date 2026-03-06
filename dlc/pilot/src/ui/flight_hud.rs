//! In-flight instrument panel — altitude, speed, heading, fuel, throttle,
//! flight phase, distance remaining, passenger happiness.

use crate::shared::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct FlightHudRoot;

#[derive(Component)]
pub struct FlightHudText;

pub fn spawn_flight_hud(mut commands: Commands, font: Res<UiFontHandle>) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    commands
        .spawn((
            FlightHudRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(8.0),
                top: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.08, 0.8)),
        ))
        .with_children(|parent| {
            // 8 instrument lines
            for _ in 0..8 {
                parent.spawn((
                    FlightHudText,
                    Text::new(""),
                    text_style.clone(),
                    TextColor(Color::srgb(0.3, 1.0, 0.3)),
                ));
            }
        });
}

pub fn despawn_flight_hud(mut commands: Commands, query: Query<Entity, With<FlightHudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_flight_hud(
    flight: Res<FlightState>,
    query: Query<Entity, With<FlightHudRoot>>,
    children_q: Query<&Children>,
    mut text_q: Query<&mut Text, With<FlightHudText>>,
) {
    let Ok(root) = query.get_single() else { return };
    let Ok(children) = children_q.get(root) else {
        return;
    };

    let lines = [
        format!("Phase: {}", flight.phase.display_name()),
        format!("ALT: {:.0} ft", flight.altitude_ft),
        format!("SPD: {:.0} kts", flight.speed_knots),
        format!("HDG: {:.0}°", flight.heading_deg),
        format!("FUEL: {:.0}%", flight.fuel_remaining),
        format!("THR: {:.0}%", flight.throttle * 100.0),
        format!("DIST: {:.1} nm", flight.distance_remaining_nm),
        format!("PAX: {:.0}%", flight.passengers_happy),
    ];

    for (i, child) in children.iter().enumerate() {
        if i < lines.len() {
            if let Ok(mut text) = text_q.get_mut(*child) {
                **text = lines[i].clone();
            }
        }
    }
}
