//! World domain — seasonal systems, airport status, world events.

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::shared::*;

use rand::Rng;
use std::collections::HashMap;

pub mod objects;
pub mod seasonal;
pub mod lighting;
pub mod weather_fx;
pub mod events;
pub mod ysort;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AirportStatusMap>()
            .init_resource::<WorldEventQueue>()
            .init_resource::<seasonal::ActiveSeasonalEvent>()
            .init_resource::<lighting::AmbientLighting>()
            .init_resource::<weather_fx::WeatherFxTimer>()
            .init_resource::<weather_fx::StormLightningTimer>()
            .init_resource::<events::DynamicEventQueue>()
            .add_systems(Startup, (
                lighting::spawn_light_overlay,
                weather_fx::spawn_fog_overlay,
            ))
            .add_systems(
                Update,
                (
                    advance_season.run_if(in_state(GameState::Playing)),
                    seasonal_decorations.run_if(in_state(GameState::Playing)),
                    check_airport_status.run_if(in_state(GameState::Playing)),
                    process_world_events.run_if(in_state(GameState::Playing)),
                    objects::spawn_extended_zone_objects.run_if(in_state(GameState::Playing)),
                    objects::interact_extended_object.run_if(in_state(GameState::Playing)),
                    seasonal::update_seasonal_decorations.run_if(in_state(GameState::Playing)),
                    seasonal::check_seasonal_events.run_if(in_state(GameState::Playing)),
                    seasonal::apply_seasonal_tile_tints.run_if(in_state(GameState::Playing)),
                    lighting::update_ambient_lighting.run_if(in_state(GameState::Playing)),
                    lighting::animate_runway_lights.run_if(in_state(GameState::Playing)),
                    lighting::spawn_runway_lights.run_if(in_state(GameState::Playing)),
                    weather_fx::update_fog.run_if(in_state(GameState::Playing)),
                    weather_fx::spawn_weather_particles.run_if(in_state(GameState::Playing)),
                    weather_fx::update_weather_particles.run_if(in_state(GameState::Playing)),
                    weather_fx::update_storm_lightning.run_if(in_state(GameState::Playing)),
                    weather_fx::cleanup_weather_particles.run_if(in_state(GameState::Playing)),
                    events::update_dynamic_events.run_if(in_state(GameState::Playing)),
                    events::display_news_ticker.run_if(in_state(GameState::Playing)),
                    ysort::ysort_update.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

// ─── Airport Status ──────────────────────────────────────────────────────

/// Why an airport might be temporarily unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClosureReason {
    Weather,
    Maintenance,
    AirShow,
    Emergency,
}

/// Current operational status of an airport.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AirportStatus {
    pub open: bool,
    pub closure_reason: Option<ClosureReason>,
    pub reopen_day: Option<u32>,
}

impl Default for AirportStatus {
    fn default() -> Self {
        Self {
            open: true,
            closure_reason: None,
            reopen_day: None,
        }
    }
}

/// Maps every airport to its current status.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AirportStatusMap {
    pub statuses: HashMap<AirportId, AirportStatus>,
}

impl AirportStatusMap {
    pub fn is_open(&self, id: &AirportId) -> bool {
        self.statuses.get(id).is_none_or(|s| s.open)
    }
}

// ─── World Events ────────────────────────────────────────────────────────

/// Global events that affect all airports simultaneously.
#[derive(Clone, Debug)]
pub enum WorldEvent {
    AirShow { airport: AirportId, duration_days: u32 },
    Holiday { name: String, duration_days: u32 },
    WeatherEmergency { affected: Vec<AirportId>, duration_days: u32 },
}

#[derive(Resource, Default)]
pub struct WorldEventQueue {
    pub active_events: Vec<(WorldEvent, u32)>, // (event, end_day)
}

// ─── Seasonal Decoration Tag ─────────────────────────────────────────────

#[derive(Component)]
pub struct SeasonalDecoration {
    pub season: Season,
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Listen for `SeasonChangeEvent` and apply season-specific effects.
pub fn advance_season(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut weather: ResMut<WeatherState>,
    mut toast: EventWriter<ToastEvent>,
) {
    for evt in season_events.read() {
        // Adjust base weather probability per season
        weather.current = match evt.new_season {
            Season::Spring => Weather::Rain,
            Season::Summer => Weather::Clear,
            Season::Fall => Weather::Cloudy,
            Season::Winter => Weather::Snow,
        };

        toast.send(ToastEvent {
            message: format!("{} has arrived!", evt.new_season),
            duration_secs: 4.0,
        });
    }
}

/// Show/hide decorations that are tagged for a specific season.
pub fn seasonal_decorations(
    calendar: Res<Calendar>,
    mut query: Query<(&SeasonalDecoration, &mut Visibility)>,
) {
    for (deco, mut vis) in &mut query {
        *vis = if deco.season == calendar.season {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Periodically roll random airport closures due to weather or maintenance.
/// Checked once per in-game day (when day ticks to 1, i.e. start of day).
pub fn check_airport_status(
    calendar: Res<Calendar>,
    mut status_map: ResMut<AirportStatusMap>,
    mut toast: EventWriter<ToastEvent>,
) {
    // Only run at the start of each day (minute 0, hour == WAKE_HOUR)
    if calendar.hour != WAKE_HOUR || calendar.minute != 0 {
        return;
    }

    let today = calendar.total_days();
    let mut rng = rand::thread_rng();

    // Re-open airports whose closure has expired
    let to_reopen: Vec<AirportId> = status_map
        .statuses
        .iter()
        .filter(|(_, s)| !s.open && s.reopen_day.is_some_and(|d| d <= today))
        .map(|(id, _)| *id)
        .collect();

    for id in to_reopen {
        if let Some(status) = status_map.statuses.get_mut(&id) {
            status.open = true;
            status.closure_reason = None;
            status.reopen_day = None;
            toast.send(ToastEvent {
                message: format!("{} has reopened!", id.display_name()),
                duration_secs: 3.0,
            });
        }
    }

    // Random chance of a closure (5% per non-home airport per day)
    let airports = [
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

    for &airport in &airports {
        if status_map.is_open(&airport) && rng.gen_bool(0.05) {
            let reason = if rng.gen_bool(0.6) {
                ClosureReason::Weather
            } else {
                ClosureReason::Maintenance
            };
            let duration = rng.gen_range(1..=3);
            status_map.statuses.insert(
                airport,
                AirportStatus {
                    open: false,
                    closure_reason: Some(reason),
                    reopen_day: Some(today + duration),
                },
            );
            toast.send(ToastEvent {
                message: format!(
                    "{} closed ({:?}) — reopens in {} day(s).",
                    airport.display_name(),
                    reason,
                    duration
                ),
                duration_secs: 5.0,
            });
        }
    }
}

/// Tick active world events and fire notifications.
pub fn process_world_events(
    calendar: Res<Calendar>,
    mut event_queue: ResMut<WorldEventQueue>,
    mut toast: EventWriter<ToastEvent>,
) {
    let today = calendar.total_days();

    // Remove expired events
    let expired: Vec<usize> = event_queue
        .active_events
        .iter()
        .enumerate()
        .filter(|(_, (_, end))| *end <= today)
        .map(|(i, _)| i)
        .collect();

    for i in expired.into_iter().rev() {
        let (evt, _) = event_queue.active_events.remove(i);
        match evt {
            WorldEvent::AirShow { airport, .. } => {
                toast.send(ToastEvent {
                    message: format!("The air show at {} has ended.", airport.display_name()),
                    duration_secs: 3.0,
                });
            }
            WorldEvent::Holiday { name, .. } => {
                toast.send(ToastEvent {
                    message: format!("{} celebrations are over.", name),
                    duration_secs: 3.0,
                });
            }
            WorldEvent::WeatherEmergency { .. } => {
                toast.send(ToastEvent {
                    message: "The weather emergency has been lifted.".into(),
                    duration_secs: 3.0,
                });
            }
        }
    }
}
