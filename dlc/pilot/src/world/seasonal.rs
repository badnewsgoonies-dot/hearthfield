//! Seasonal decoration system — airport decorations, seasonal events, and mission bonuses.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS & RESOURCES
// ═══════════════════════════════════════════════════════════════════════════

/// Seasonal decoration entity spawned in zones.
#[derive(Component)]
pub struct SeasonalDecor {
    pub season: Season,
    pub decor_kind: DecorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecorKind {
    // Spring
    FlowerPot,
    CherryBlossom,
    SpringBanner,
    // Summer
    SummerBanner,
    PalmDecor,
    BeachPoster,
    // Fall
    FallenLeaves,
    PumpkinDisplay,
    HarvestWreath,
    // Winter
    HolidayLights,
    SnowmanDecor,
    WinterGarland,
}

impl DecorKind {
    pub fn placeholder_color(&self) -> Color {
        match self {
            Self::FlowerPot => Color::srgb(0.9, 0.5, 0.6),
            Self::CherryBlossom => Color::srgb(1.0, 0.7, 0.8),
            Self::SpringBanner => Color::srgb(0.5, 0.9, 0.5),
            Self::SummerBanner => Color::srgb(1.0, 0.8, 0.2),
            Self::PalmDecor => Color::srgb(0.3, 0.7, 0.3),
            Self::BeachPoster => Color::srgb(0.4, 0.7, 0.9),
            Self::FallenLeaves => Color::srgb(0.8, 0.5, 0.2),
            Self::PumpkinDisplay => Color::srgb(0.9, 0.5, 0.1),
            Self::HarvestWreath => Color::srgb(0.7, 0.4, 0.1),
            Self::HolidayLights => Color::srgb(0.9, 0.2, 0.2),
            Self::SnowmanDecor => Color::srgb(0.95, 0.95, 1.0),
            Self::WinterGarland => Color::srgb(0.2, 0.6, 0.2),
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            Self::SpringBanner | Self::SummerBanner => Vec2::new(20.0, 8.0),
            Self::FallenLeaves => Vec2::new(12.0, 6.0),
            Self::HolidayLights => Vec2::new(24.0, 4.0),
            _ => Vec2::new(10.0, 10.0),
        }
    }
}

/// Seasonal event definitions.
#[derive(Clone, Debug)]
pub struct SeasonalEvent {
    pub name: &'static str,
    pub season: Season,
    pub start_day: u32,
    pub duration_days: u32,
    pub description: &'static str,
}

pub const SEASONAL_EVENTS: &[SeasonalEvent] = &[
    SeasonalEvent {
        name: "Spring Air Show",
        season: Season::Spring,
        start_day: 7,
        duration_days: 3,
        description: "Aerobatic displays and aircraft exhibitions at HomeBase!",
    },
    SeasonalEvent {
        name: "Summer Sky Festival",
        season: Season::Summer,
        start_day: 14,
        duration_days: 4,
        description: "Tourism peaks — charter flights pay double!",
    },
    SeasonalEvent {
        name: "Fall Harvest Run",
        season: Season::Fall,
        start_day: 10,
        duration_days: 5,
        description: "Deliver harvest cargo for bonus gold. Cargo missions pay 50% more!",
    },
    SeasonalEvent {
        name: "Winter Holiday Flight",
        season: Season::Winter,
        start_day: 20,
        duration_days: 5,
        description: "Spread holiday cheer! Gift deliveries unlock exclusive rewards.",
    },
];

/// Resource tracking the currently active seasonal event, if any.
#[derive(Resource, Default)]
pub struct ActiveSeasonalEvent {
    pub event_name: Option<&'static str>,
    pub days_remaining: u32,
}

// ═══════════════════════════════════════════════════════════════════════════
// SEASON DECORATIONS DATA
// ═══════════════════════════════════════════════════════════════════════════

struct DecorPlacement {
    kind: DecorKind,
    gx: i32,
    gy: i32,
}

fn spring_decorations() -> Vec<DecorPlacement> {
    vec![
        DecorPlacement {
            kind: DecorKind::FlowerPot,
            gx: 2,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::FlowerPot,
            gx: 18,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::CherryBlossom,
            gx: 10,
            gy: 0,
        },
        DecorPlacement {
            kind: DecorKind::SpringBanner,
            gx: 10,
            gy: 1,
        },
    ]
}

fn summer_decorations() -> Vec<DecorPlacement> {
    vec![
        DecorPlacement {
            kind: DecorKind::SummerBanner,
            gx: 10,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::PalmDecor,
            gx: 2,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::PalmDecor,
            gx: 18,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::BeachPoster,
            gx: 14,
            gy: 2,
        },
    ]
}

fn fall_decorations() -> Vec<DecorPlacement> {
    vec![
        DecorPlacement {
            kind: DecorKind::FallenLeaves,
            gx: 4,
            gy: 9,
        },
        DecorPlacement {
            kind: DecorKind::FallenLeaves,
            gx: 15,
            gy: 9,
        },
        DecorPlacement {
            kind: DecorKind::PumpkinDisplay,
            gx: 2,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::HarvestWreath,
            gx: 10,
            gy: 1,
        },
    ]
}

fn winter_decorations() -> Vec<DecorPlacement> {
    vec![
        DecorPlacement {
            kind: DecorKind::HolidayLights,
            gx: 10,
            gy: 0,
        },
        DecorPlacement {
            kind: DecorKind::SnowmanDecor,
            gx: 3,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::SnowmanDecor,
            gx: 17,
            gy: 1,
        },
        DecorPlacement {
            kind: DecorKind::WinterGarland,
            gx: 10,
            gy: 1,
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Spawn/despawn seasonal decorations when the season changes.
pub fn update_seasonal_decorations(
    mut commands: Commands,
    mut season_events: EventReader<SeasonChangeEvent>,
    existing: Query<Entity, With<SeasonalDecor>>,
) {
    for evt in season_events.read() {
        // Despawn previous decorations
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }

        let placements = match evt.new_season {
            Season::Spring => spring_decorations(),
            Season::Summer => summer_decorations(),
            Season::Fall => fall_decorations(),
            Season::Winter => winter_decorations(),
        };

        for p in placements {
            let pos = grid_to_world_center(p.gx, p.gy);
            commands.spawn((
                SeasonalDecor {
                    season: evt.new_season,
                    decor_kind: p.kind,
                },
                Sprite::from_color(p.kind.placeholder_color(), p.kind.size()),
                Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
            ));
        }
    }
}

/// Check for seasonal events at the start of each day.
pub fn check_seasonal_events(
    calendar: Res<Calendar>,
    mut active_event: ResMut<ActiveSeasonalEvent>,
    mut toast: EventWriter<ToastEvent>,
    mut day_events: EventReader<DayEndEvent>,
) {
    for _ev in day_events.read() {
        // Tick active event
        if active_event.event_name.is_some() {
            if active_event.days_remaining > 0 {
                active_event.days_remaining -= 1;
            }
            if active_event.days_remaining == 0 {
                if let Some(name) = active_event.event_name.take() {
                    toast.send(ToastEvent {
                        message: format!("{name} has ended!"),
                        duration_secs: 3.5,
                    });
                }
            }
        }

        // Check if a new event should start
        if active_event.event_name.is_none() {
            for evt in SEASONAL_EVENTS {
                if evt.season == calendar.season && calendar.day == evt.start_day {
                    active_event.event_name = Some(evt.name);
                    active_event.days_remaining = evt.duration_days;
                    toast.send(ToastEvent {
                        message: format!("🎉 {} begins! {}", evt.name, evt.description),
                        duration_secs: 5.0,
                    });
                }
            }
        }
    }
}

/// Seasonal mission bonus multiplier — cargo is more profitable in winter,
/// tourism/charter in summer, etc.
pub fn seasonal_mission_bonus(season: &Season, mission_type: &MissionType) -> f32 {
    match (season, mission_type) {
        (Season::Winter, MissionType::Cargo) => 1.3,
        (Season::Winter, MissionType::Delivery) => 1.25,
        (Season::Summer, MissionType::Charter) => 1.5,
        (Season::Summer, MissionType::VIP) => 1.4,
        (Season::Summer, MissionType::Passenger) => 1.2,
        (Season::Spring, MissionType::AirShow) => 1.5,
        (Season::Spring, MissionType::Survey) => 1.2,
        (Season::Fall, MissionType::Cargo) => 1.5,
        (Season::Fall, MissionType::Delivery) => 1.3,
        _ => 1.0,
    }
}

/// Apply a tint to outdoor tiles based on the current season.
pub fn apply_seasonal_tile_tints(
    calendar: Res<Calendar>,
    mut tiles: Query<(&MapTile, &mut Sprite)>,
) {
    let tint = match calendar.season {
        Season::Spring => Color::srgba(0.9, 1.0, 0.9, 1.0),
        Season::Summer => Color::srgba(1.0, 1.0, 0.95, 1.0),
        Season::Fall => Color::srgba(1.0, 0.92, 0.85, 1.0),
        Season::Winter => Color::srgba(0.9, 0.92, 1.0, 1.0),
    };

    for (tile, mut sprite) in &mut tiles {
        if matches!(
            tile.kind,
            TileKind::Grass | TileKind::Sand | TileKind::Tarmac
        ) {
            sprite.color = tint;
        }
    }
}
