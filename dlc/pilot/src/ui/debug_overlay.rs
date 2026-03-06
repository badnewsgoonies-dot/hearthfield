//! Debug overlay — F3 toggle, shows FPS, position, airport/zone, weather, flight state.

use crate::shared::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct DebugOverlayRoot;

#[derive(Component)]
pub struct DebugText;

#[allow(clippy::too_many_arguments)]
pub fn toggle_debug_overlay(
    mut commands: Commands,
    input: Res<PlayerInput>,
    mut overlay: ResMut<DebugOverlay>,
    font: Res<UiFontHandle>,
    existing: Query<Entity, With<DebugOverlayRoot>>,
    grid_pos: Res<GridPosition>,
    location: Res<PlayerLocation>,
    weather: Res<WeatherState>,
    flight: Res<FlightState>,
    time: Res<Time>,
) {
    if !input.debug_overlay {
        // Update text when overlay is visible
        if overlay.visible {
            update_debug_text(
                &existing, &commands, &grid_pos, &location, &weather, &flight, &time,
            );
        }
        return;
    }

    overlay.visible = !overlay.visible;

    if overlay.visible {
        spawn_debug_overlay(&mut commands, &font);
    } else {
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_debug_overlay(commands: &mut Commands, font: &Res<UiFontHandle>) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 12.0,
        ..default()
    };

    commands
        .spawn((
            DebugOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(8.0),
                bottom: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(1.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            for _ in 0..6 {
                parent.spawn((
                    DebugText,
                    Text::new(""),
                    text_style.clone(),
                    TextColor(Color::srgb(0.5, 1.0, 0.5)),
                ));
            }
        });
}

fn update_debug_text(
    root_q: &Query<Entity, With<DebugOverlayRoot>>,
    commands: &Commands,
    grid_pos: &Res<GridPosition>,
    location: &Res<PlayerLocation>,
    weather: &Res<WeatherState>,
    flight: &Res<FlightState>,
    time: &Res<Time>,
) {
    // Text update is handled in a separate system to avoid borrow conflicts.
    // The actual refresh happens via update_debug_overlay_text below.
    let _ = (root_q, commands, grid_pos, location, weather, flight, time);
}

/// Standalone system that refreshes debug overlay text each frame when visible.
#[allow(clippy::too_many_arguments)]
pub fn update_debug_overlay_text(
    overlay: Res<DebugOverlay>,
    root_q: Query<Entity, With<DebugOverlayRoot>>,
    children_q: Query<&Children>,
    mut text_q: Query<&mut Text, With<DebugText>>,
    grid_pos: Res<GridPosition>,
    location: Res<PlayerLocation>,
    weather: Res<WeatherState>,
    flight: Res<FlightState>,
    time: Res<Time>,
) {
    if !overlay.visible {
        return;
    }
    let Ok(root) = root_q.get_single() else {
        return;
    };
    let Ok(children) = children_q.get(root) else {
        return;
    };

    let fps = (1.0 / time.delta_secs()).round() as u32;

    let lines = [
        format!("FPS: {fps}"),
        format!("Grid: ({}, {})", grid_pos.x, grid_pos.y),
        format!("Airport: {}", location.airport.display_name()),
        format!("Zone: {:?}", location.zone),
        format!(
            "Weather: {:?} | Wind: {:.0}kts",
            weather.current, weather.wind_speed_knots
        ),
        if flight.phase != FlightPhase::Idle {
            format!(
                "Flight: {} ALT:{:.0}",
                flight.phase.display_name(),
                flight.altitude_ft
            )
        } else {
            "Flight: On Ground".to_string()
        },
    ];

    for (i, child) in children.iter().enumerate() {
        if i < lines.len() {
            if let Ok(mut text) = text_q.get_mut(*child) {
                **text = lines[i].clone();
            }
        }
    }
}
