//! World map screen — shows airports, connections, weather, and missions.

use bevy::prelude::*;
use crate::shared::*;

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct MapScreenRoot;

#[derive(Component)]
pub struct AirportNode {
    pub airport: AirportId,
}

#[derive(Component)]
pub struct AirportInfoPopup;

#[derive(Component)]
pub struct ConnectionLine;

/// Screen-space layout positions for each airport icon.
fn airport_screen_pos(airport: AirportId) -> (f32, f32) {
    match airport {
        AirportId::HomeBase    => (400.0, 350.0),
        AirportId::Windport    => (200.0, 450.0),
        AirportId::Frostpeak   => (350.0, 150.0),
        AirportId::Sunhaven    => (600.0, 500.0),
        AirportId::Ironforge   => (550.0, 250.0),
        AirportId::Cloudmere   => (150.0, 200.0),
        AirportId::Duskhollow  => (750.0, 400.0),
        AirportId::Stormwatch  => (850.0, 200.0),
        AirportId::Grandcity   => (700.0, 300.0),
        AirportId::Skyreach    => (950.0, 100.0),
    }
}

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_map_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    pilot: Res<PilotState>,
    play_stats: Res<PlayStats>,
    weather: Res<WeatherState>,
    mission_board: Res<MissionBoard>,
) {
    let root = commands
        .spawn((
            MapScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.05, 0.12, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new("WORLD MAP"),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.9, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(15.0),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // Current location indicator
    let loc_text = format!("Current: {}", pilot.current_airport.display_name());
    let loc_label = commands
        .spawn((
            Text::new(loc_text),
            TextFont {
                font: font.0.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 1.0, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(50.0),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(loc_label);

    // Spawn airport nodes
    let all_airports = [
        AirportId::HomeBase,
        AirportId::Windport,
        AirportId::Frostpeak,
        AirportId::Sunhaven,
        AirportId::Ironforge,
        AirportId::Cloudmere,
        AirportId::Duskhollow,
        AirportId::Stormwatch,
        AirportId::Grandcity,
        AirportId::Skyreach,
    ];

    for airport in &all_airports {
        let (x, y) = airport_screen_pos(*airport);
        let visited = play_stats.airports_visited.contains(airport);
        let is_current = *airport == pilot.current_airport;

        let bg_color = if is_current {
            Color::srgb(0.2, 0.8, 0.2)
        } else if visited {
            Color::srgb(0.3, 0.5, 0.8)
        } else {
            Color::srgb(0.3, 0.3, 0.3)
        };

        let node_entity = commands
            .spawn((
                AirportNode { airport: *airport },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x),
                    top: Val::Px(y),
                    width: Val::Px(14.0),
                    height: Val::Px(14.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(bg_color),
                BorderColor(Color::WHITE),
            ))
            .id();
        commands.entity(root).add_child(node_entity);

        // Airport name label
        let status_icon = if is_current {
            "✈ "
        } else if visited {
            "● "
        } else {
            "○ "
        };

        let missions_here = mission_board
            .available
            .iter()
            .filter(|m| m.origin == *airport || m.destination == *airport)
            .count();
        let mission_badge = if missions_here > 0 {
            format!(" [{}]", missions_here)
        } else {
            String::new()
        };

        let label_text = format!("{}{}{}", status_icon, airport.display_name(), mission_badge);
        let name_label = commands
            .spawn((
                Text::new(label_text),
                TextFont {
                    font: font.0.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x + 18.0),
                    top: Val::Px(y - 2.0),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(name_label);

        // Distance from current
        if !is_current {
            let dist = airport_distance(pilot.current_airport, *airport);
            let rank_ok = pilot.rank >= airport.unlock_rank();
            let info_text = format!(
                "{:.0} nm | {}",
                dist,
                if rank_ok { "Unlocked" } else { "Locked" }
            );
            let info_color = if rank_ok {
                Color::srgb(0.6, 0.7, 0.6)
            } else {
                Color::srgb(0.7, 0.4, 0.4)
            };
            let info_label = commands
                .spawn((
                    Text::new(info_text),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(info_color),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(x + 18.0),
                        top: Val::Px(y + 12.0),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(info_label);
        }
    }

    // Weather legend
    let weather_text = format!("Current weather: {:?} | Wind: {:.0} kts", weather.current, weather.wind_speed_knots);
    let weather_label = commands
        .spawn((
            Text::new(weather_text),
            TextFont {
                font: font.0.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.8)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(weather_label);
}

pub fn despawn_map_screen(
    mut commands: Commands,
    query: Query<Entity, With<MapScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
