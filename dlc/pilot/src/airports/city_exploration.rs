//! City exploration — visit zones, collect souvenirs, discover events.

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::shared::*;

pub struct CityExplorationPlugin;

impl Plugin for CityExplorationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityState>()
            .add_systems(
                Update,
                (
                    explore_city,
                    handle_city_events,
                    collect_souvenir,
                    update_city_atmosphere,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CityZone {
    Downtown,
    Park,
    Market,
    Harbor,
    Mountain,
    Beach,
    Museum,
    Restaurant,
}

impl CityZone {
    pub fn display_name(&self) -> &'static str {
        match self {
            CityZone::Downtown => "Downtown",
            CityZone::Park => "City Park",
            CityZone::Market => "Market District",
            CityZone::Harbor => "Harbor",
            CityZone::Mountain => "Mountain Trail",
            CityZone::Beach => "Beach",
            CityZone::Museum => "Aviation Museum",
            CityZone::Restaurant => "Restaurant",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CityDef {
    pub airport: AirportId,
    pub zones: Vec<CityZone>,
    pub souvenir_id: String,
    pub souvenir_name: String,
}

fn city_definitions() -> Vec<CityDef> {
    vec![
        CityDef {
            airport: AirportId::HomeBase,
            zones: vec![CityZone::Downtown, CityZone::Park, CityZone::Restaurant],
            souvenir_id: "souvenir_clearfield".to_string(),
            souvenir_name: "Clearfield Snow Globe".to_string(),
        },
        CityDef {
            airport: AirportId::Windport,
            zones: vec![CityZone::Harbor, CityZone::Market, CityZone::Restaurant, CityZone::Beach],
            souvenir_id: "souvenir_windport".to_string(),
            souvenir_name: "Windport Compass".to_string(),
        },
        CityDef {
            airport: AirportId::Frostpeak,
            zones: vec![CityZone::Mountain, CityZone::Museum, CityZone::Restaurant],
            souvenir_id: "souvenir_frostpeak".to_string(),
            souvenir_name: "Frostpeak Crystal".to_string(),
        },
        CityDef {
            airport: AirportId::Sunhaven,
            zones: vec![CityZone::Beach, CityZone::Market, CityZone::Restaurant, CityZone::Park],
            souvenir_id: "souvenir_sunhaven".to_string(),
            souvenir_name: "Sunhaven Shell Necklace".to_string(),
        },
        CityDef {
            airport: AirportId::Ironforge,
            zones: vec![CityZone::Downtown, CityZone::Museum, CityZone::Market],
            souvenir_id: "souvenir_ironforge".to_string(),
            souvenir_name: "Ironforge Gear".to_string(),
        },
        CityDef {
            airport: AirportId::Cloudmere,
            zones: vec![CityZone::Mountain, CityZone::Park, CityZone::Restaurant],
            souvenir_id: "souvenir_cloudmere".to_string(),
            souvenir_name: "Cloudmere Cloud Jar".to_string(),
        },
        CityDef {
            airport: AirportId::Duskhollow,
            zones: vec![CityZone::Market, CityZone::Museum, CityZone::Restaurant],
            souvenir_id: "souvenir_duskhollow".to_string(),
            souvenir_name: "Duskhollow Sand Rose".to_string(),
        },
        CityDef {
            airport: AirportId::Stormwatch,
            zones: vec![CityZone::Museum, CityZone::Downtown, CityZone::Restaurant],
            souvenir_id: "souvenir_stormwatch".to_string(),
            souvenir_name: "Stormwatch Lightning Stone".to_string(),
        },
        CityDef {
            airport: AirportId::Grandcity,
            zones: vec![CityZone::Downtown, CityZone::Harbor, CityZone::Market, CityZone::Museum],
            souvenir_id: "souvenir_grandcity".to_string(),
            souvenir_name: "Grand City Model Plane".to_string(),
        },
        CityDef {
            airport: AirportId::Skyreach,
            zones: vec![CityZone::Mountain, CityZone::Park, CityZone::Museum, CityZone::Restaurant],
            souvenir_id: "souvenir_skyreach".to_string(),
            souvenir_name: "Skyreach Golden Wing".to_string(),
        },
    ]
}

fn city_for_airport(airport: AirportId) -> Option<CityDef> {
    city_definitions().into_iter().find(|c| c.airport == airport)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CityEvent {
    pub name: String,
    pub description: String,
    pub xp_reward: u32,
    pub friendship_bonus: i32,
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CityState {
    pub current_zone: Option<CityZone>,
    pub souvenirs_collected: Vec<String>,
    pub zones_visited: Vec<(AirportId, CityZone)>,
    pub active_event: Option<CityEvent>,
    pub event_cooldown: f32,
}

impl CityState {
    pub fn has_souvenir(&self, souvenir_id: &str) -> bool {
        self.souvenirs_collected.iter().any(|s| s == souvenir_id)
    }

    pub fn has_visited(&self, airport: AirportId, zone: CityZone) -> bool {
        self.zones_visited.iter().any(|&(a, z)| a == airport && z == zone)
    }

    pub fn total_souvenirs(&self) -> usize {
        self.souvenirs_collected.len()
    }
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn explore_city(
    input: Res<PlayerInput>,
    player_location: Res<PlayerLocation>,
    mut city_state: ResMut<CityState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if player_location.zone != MapZone::CityStreet {
        city_state.current_zone = None;
        return;
    }

    let Some(city) = city_for_airport(player_location.airport) else {
        return;
    };

    // Auto-assign first zone if none selected
    if city_state.current_zone.is_none() {
        if let Some(&first) = city.zones.first() {
            city_state.current_zone = Some(first);
            if !city_state.has_visited(player_location.airport, first) {
                city_state.zones_visited.push((player_location.airport, first));
                toast_events.send(ToastEvent {
                    message: format!("Exploring {} — {}", city.airport.display_name(), first.display_name()),
                    duration_secs: 3.0,
                });
            }
        }
    }

    // Navigate between zones
    if input.tab_next {
        if let Some(current) = city_state.current_zone {
            let idx = city.zones.iter().position(|&z| z == current).unwrap_or(0);
            let next = city.zones[(idx + 1) % city.zones.len()];
            city_state.current_zone = Some(next);
            if !city_state.has_visited(player_location.airport, next) {
                city_state.zones_visited.push((player_location.airport, next));
            }
            toast_events.send(ToastEvent {
                message: format!("Now in: {}", next.display_name()),
                duration_secs: 2.0,
            });
        }
    }
    if input.tab_prev {
        if let Some(current) = city_state.current_zone {
            let idx = city.zones.iter().position(|&z| z == current).unwrap_or(0);
            let prev = if idx == 0 { city.zones.len() - 1 } else { idx - 1 };
            let zone = city.zones[prev];
            city_state.current_zone = Some(zone);
            if !city_state.has_visited(player_location.airport, zone) {
                city_state.zones_visited.push((player_location.airport, zone));
            }
            toast_events.send(ToastEvent {
                message: format!("Now in: {}", zone.display_name()),
                duration_secs: 2.0,
            });
        }
    }
}

pub fn handle_city_events(
    time: Res<Time>,
    calendar: Res<Calendar>,
    player_location: Res<PlayerLocation>,
    mut city_state: ResMut<CityState>,
    mut xp_events: EventWriter<XpGainEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if player_location.zone != MapZone::CityStreet {
        return;
    }

    city_state.event_cooldown -= time.delta_secs();
    if city_state.event_cooldown > 0.0 {
        return;
    }

    let Some(zone) = city_state.current_zone else { return };

    // Seasonal and zone-specific events
    let event = match (zone, calendar.season) {
        (CityZone::Beach, Season::Summer) => Some(CityEvent {
            name: "Beach Festival".to_string(),
            description: "A summer festival with food and music!".to_string(),
            xp_reward: 10,
            friendship_bonus: 5,
        }),
        (CityZone::Mountain, Season::Winter) => Some(CityEvent {
            name: "Snowfall Vista".to_string(),
            description: "A beautiful snowfall covers the mountain.".to_string(),
            xp_reward: 5,
            friendship_bonus: 0,
        }),
        (CityZone::Market, Season::Fall) => Some(CityEvent {
            name: "Harvest Market".to_string(),
            description: "Local farmers sell autumn produce.".to_string(),
            xp_reward: 5,
            friendship_bonus: 3,
        }),
        (CityZone::Park, Season::Spring) => Some(CityEvent {
            name: "Cherry Blossoms".to_string(),
            description: "The park is covered in cherry blossoms.".to_string(),
            xp_reward: 5,
            friendship_bonus: 0,
        }),
        _ => None,
    };

    if let Some(ev) = event {
        city_state.event_cooldown = 300.0; // 5 minutes between events
        xp_events.send(XpGainEvent {
            amount: ev.xp_reward,
            source: format!("City event: {}", ev.name),
        });
        toast_events.send(ToastEvent {
            message: format!("🎉 {} — {}", ev.name, ev.description),
            duration_secs: 5.0,
        });
        city_state.active_event = Some(ev);
    }
}

pub fn collect_souvenir(
    input: Res<PlayerInput>,
    player_location: Res<PlayerLocation>,
    mut city_state: ResMut<CityState>,
    mut inventory: ResMut<Inventory>,
    mut gold: ResMut<Gold>,
    mut toast_events: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    if player_location.zone != MapZone::CityStreet || !input.interact {
        return;
    }

    let Some(city) = city_for_airport(player_location.airport) else {
        return;
    };

    if city_state.has_souvenir(&city.souvenir_id) {
        return;
    }

    // Must be in market zone to buy souvenirs
    if city_state.current_zone != Some(CityZone::Market) {
        return;
    }

    let cost = 50u32;
    if gold.amount < cost {
        toast_events.send(ToastEvent {
            message: "Not enough gold for the souvenir.".to_string(),
            duration_secs: 2.0,
        });
        return;
    }

    gold.amount -= cost;
    inventory.add_item(&city.souvenir_id, 1);
    city_state.souvenirs_collected.push(city.souvenir_id.clone());

    gold_events.send(GoldChangeEvent {
        amount: -(cost as i32),
        reason: "Souvenir".to_string(),
    });
    toast_events.send(ToastEvent {
        message: format!("🎁 Collected: {} (-{cost}g)", city.souvenir_name),
        duration_secs: 4.0,
    });

    if city_state.total_souvenirs() == 10 {
        toast_events.send(ToastEvent {
            message: "🏆 All souvenirs collected!".to_string(),
            duration_secs: 5.0,
        });
    }
}

pub fn update_city_atmosphere(
    calendar: Res<Calendar>,
    player_location: Res<PlayerLocation>,
    mut toast_events: EventWriter<ToastEvent>,
    weather: Res<WeatherState>,
    mut music_events: EventWriter<PlayMusicEvent>,
) {
    if player_location.zone != MapZone::CityStreet {
        return;
    }

    // Time-of-day atmosphere (only on hour change - approximated by checking minute 0)
    if calendar.minute == 0 && calendar.hour == 20 {
        toast_events.send(ToastEvent {
            message: "The city lights come alive as evening falls.".to_string(),
            duration_secs: 3.0,
        });
        music_events.send(PlayMusicEvent {
            track_id: "city_night".to_string(),
            fade_in: true,
        });
    }

    // Weather flavor text when it changes
    let _ = &weather; // Used for atmosphere tracking
}
