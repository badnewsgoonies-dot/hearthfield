//! Dynamic world event system — events that affect airports and gameplay globally.
//!
//! Events like air shows, storms, fuel shortages, and strikes create dynamic
//! gameplay opportunities. Event chains allow cascading consequences.

use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ─── World Event Types ───────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicEventKind {
    AirShow,
    MajorStorm,
    Earthquake,
    FuelShortage,
    AirportStrike,
    CelebrityVisit,
    Holiday,
    MilitaryExercise,
    VolcanicAsh,
    PandemicScare,
}

impl DynamicEventKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AirShow => "Air Show",
            Self::MajorStorm => "Major Storm",
            Self::Earthquake => "Earthquake",
            Self::FuelShortage => "Fuel Shortage",
            Self::AirportStrike => "Airport Workers' Strike",
            Self::CelebrityVisit => "Celebrity Visit",
            Self::Holiday => "National Holiday",
            Self::MilitaryExercise => "Military Exercise",
            Self::VolcanicAsh => "Volcanic Ash Cloud",
            Self::PandemicScare => "Pandemic Scare",
        }
    }

    pub fn news_headline(&self) -> &'static str {
        match self {
            Self::AirShow => "EXCITING: Air show draws crowds!",
            Self::MajorStorm => "WARNING: Major storm system approaching!",
            Self::Earthquake => "BREAKING: Earthquake reported near airport!",
            Self::FuelShortage => "ALERT: Fuel shortage affecting operations!",
            Self::AirportStrike => "UPDATE: Airport workers on strike!",
            Self::CelebrityVisit => "BUZZ: Celebrity spotted at the airport!",
            Self::Holiday => "NOTICE: National holiday — expect crowds!",
            Self::MilitaryExercise => "NOTAM: Military exercise — airspace restricted!",
            Self::VolcanicAsh => "DANGER: Volcanic ash cloud — flights grounded!",
            Self::PandemicScare => "HEALTH: Travel restrictions in effect!",
        }
    }

    /// Duration range in days (min, max).
    pub fn duration_range(&self) -> (u32, u32) {
        match self {
            Self::AirShow => (2, 4),
            Self::MajorStorm => (1, 3),
            Self::Earthquake => (3, 7),
            Self::FuelShortage => (2, 5),
            Self::AirportStrike => (1, 4),
            Self::CelebrityVisit => (1, 2),
            Self::Holiday => (1, 3),
            Self::MilitaryExercise => (1, 3),
            Self::VolcanicAsh => (2, 6),
            Self::PandemicScare => (3, 7),
        }
    }

    /// Can this event close an airport?
    pub fn can_close_airport(&self) -> bool {
        matches!(
            self,
            Self::MajorStorm | Self::Earthquake | Self::AirportStrike | Self::VolcanicAsh
        )
    }
}

// ─── Active Dynamic Event ────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct DynamicEvent {
    pub kind: DynamicEventKind,
    pub affected_airports: Vec<AirportId>,
    pub start_day: u32,
    pub end_day: u32,
    pub price_modifier: f32,
    pub bonus_mission_available: bool,
    pub triggered_chain: Option<DynamicEventKind>,
}

// ─── Event Effects ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct EventEffects {
    pub fuel_price_multiplier: f32,
    pub ticket_price_multiplier: f32,
    pub airport_closures: Vec<AirportId>,
    pub bonus_xp_multiplier: f32,
    pub crew_mood_modifier: f32,
}

impl Default for EventEffects {
    fn default() -> Self {
        Self {
            fuel_price_multiplier: 1.0,
            ticket_price_multiplier: 1.0,
            airport_closures: Vec::new(),
            bonus_xp_multiplier: 1.0,
            crew_mood_modifier: 0.0,
        }
    }
}

pub fn compute_effects(event: &DynamicEvent) -> EventEffects {
    let mut fx = EventEffects::default();

    match &event.kind {
        DynamicEventKind::AirShow => {
            fx.ticket_price_multiplier = 1.3;
            fx.bonus_xp_multiplier = 1.5;
            fx.crew_mood_modifier = 10.0;
        }
        DynamicEventKind::MajorStorm => {
            fx.airport_closures = event.affected_airports.clone();
            fx.crew_mood_modifier = -15.0;
        }
        DynamicEventKind::Earthquake => {
            fx.airport_closures = event.affected_airports.clone();
            fx.fuel_price_multiplier = 1.2;
            fx.crew_mood_modifier = -20.0;
        }
        DynamicEventKind::FuelShortage => {
            fx.fuel_price_multiplier = 2.0;
        }
        DynamicEventKind::AirportStrike => {
            fx.airport_closures = event.affected_airports.clone();
            fx.ticket_price_multiplier = 1.5;
        }
        DynamicEventKind::CelebrityVisit => {
            fx.ticket_price_multiplier = 1.2;
            fx.bonus_xp_multiplier = 1.2;
            fx.crew_mood_modifier = 5.0;
        }
        DynamicEventKind::Holiday => {
            fx.ticket_price_multiplier = 1.4;
            fx.crew_mood_modifier = 10.0;
        }
        DynamicEventKind::MilitaryExercise => {
            fx.airport_closures = event.affected_airports.clone();
        }
        DynamicEventKind::VolcanicAsh => {
            fx.airport_closures = event.affected_airports.clone();
            fx.fuel_price_multiplier = 1.5;
            fx.crew_mood_modifier = -25.0;
        }
        DynamicEventKind::PandemicScare => {
            fx.ticket_price_multiplier = 0.6;
            fx.crew_mood_modifier = -10.0;
        }
    }

    fx
}

// ─── Event Queue Resource ────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct DynamicEventQueue {
    pub active: Vec<DynamicEvent>,
    pub history: Vec<DynamicEvent>,
    pub news_ticker: Vec<String>,
}

impl DynamicEventQueue {
    pub fn has_event_kind(&self, kind: &DynamicEventKind) -> bool {
        self.active.iter().any(|e| &e.kind == kind)
    }

    pub fn aggregate_effects(&self) -> EventEffects {
        let mut combined = EventEffects::default();
        for event in &self.active {
            let fx = compute_effects(event);
            combined.fuel_price_multiplier *= fx.fuel_price_multiplier;
            combined.ticket_price_multiplier *= fx.ticket_price_multiplier;
            combined.bonus_xp_multiplier *= fx.bonus_xp_multiplier;
            combined.crew_mood_modifier += fx.crew_mood_modifier;
            for ap in fx.airport_closures {
                if !combined.airport_closures.contains(&ap) {
                    combined.airport_closures.push(ap);
                }
            }
        }
        combined
    }
}

// ─── Event Generation ────────────────────────────────────────────────────

/// Randomly generate a new dynamic event.
fn generate_event(day: u32) -> Option<DynamicEvent> {
    let mut rng = rand::thread_rng();

    // 8% chance per day of a new event
    if !rng.gen_bool(0.08) {
        return None;
    }

    let kinds = [
        DynamicEventKind::AirShow,
        DynamicEventKind::MajorStorm,
        DynamicEventKind::FuelShortage,
        DynamicEventKind::AirportStrike,
        DynamicEventKind::CelebrityVisit,
        DynamicEventKind::Holiday,
        DynamicEventKind::MilitaryExercise,
        DynamicEventKind::VolcanicAsh,
        DynamicEventKind::PandemicScare,
    ];

    let kind = kinds[rng.gen_range(0..kinds.len())].clone();
    let (min_dur, max_dur) = kind.duration_range();
    let duration = rng.gen_range(min_dur..=max_dur);

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

    // Pick 1-3 affected airports
    let count = rng.gen_range(1..=3).min(all_airports.len());
    let mut affected = Vec::new();
    while affected.len() < count {
        let ap = all_airports[rng.gen_range(0..all_airports.len())];
        if !affected.contains(&ap) {
            affected.push(ap);
        }
    }

    // Event chains
    let chain = match &kind {
        DynamicEventKind::MajorStorm => Some(DynamicEventKind::Earthquake),
        DynamicEventKind::Earthquake => Some(DynamicEventKind::FuelShortage),
        _ => None,
    };

    let price_mod = match &kind {
        DynamicEventKind::FuelShortage => 2.0,
        DynamicEventKind::AirportStrike => 1.5,
        DynamicEventKind::Holiday => 1.4,
        _ => 1.0,
    };

    Some(DynamicEvent {
        kind,
        affected_airports: affected,
        start_day: day,
        end_day: day + duration,
        price_modifier: price_mod,
        bonus_mission_available: rng.gen_bool(0.4),
        triggered_chain: chain,
    })
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Generate new events and expire old ones each day.
pub fn update_dynamic_events(
    calendar: Res<Calendar>,
    mut event_queue: ResMut<DynamicEventQueue>,
    mut toast: EventWriter<ToastEvent>,
) {
    if calendar.hour != WAKE_HOUR || calendar.minute != 0 {
        return;
    }

    let today = calendar.total_days();

    // Expire finished events
    let mut expired = Vec::new();
    event_queue.active.retain(|e| {
        if e.end_day <= today {
            expired.push(e.clone());
            false
        } else {
            true
        }
    });

    for e in &expired {
        toast.send(ToastEvent {
            message: format!("{} has ended.", e.kind.display_name()),
            duration_secs: 3.0,
        });
        event_queue.history.push(e.clone());

        // Trigger chain events
        if let Some(ref chain_kind) = e.triggered_chain {
            let (min_d, max_d) = chain_kind.duration_range();
            let mut rng = rand::thread_rng();
            let chain_event = DynamicEvent {
                kind: chain_kind.clone(),
                affected_airports: e.affected_airports.clone(),
                start_day: today,
                end_day: today + rng.gen_range(min_d..=max_d),
                price_modifier: 1.0,
                bonus_mission_available: false,
                triggered_chain: None,
            };
            toast.send(ToastEvent {
                message: format!("Chain event triggered: {}!", chain_kind.display_name()),
                duration_secs: 4.0,
            });
            event_queue.active.push(chain_event);
        }
    }

    // Maybe generate a new event
    if let Some(event) = generate_event(today) {
        toast.send(ToastEvent {
            message: event.kind.news_headline().to_string(),
            duration_secs: 5.0,
        });
        event_queue
            .news_ticker
            .push(event.kind.news_headline().to_string());
        event_queue.active.push(event);
    }
}

/// Display active events in a news ticker.
pub fn display_news_ticker(
    event_queue: Res<DynamicEventQueue>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !event_queue.is_changed() {
        return;
    }

    for event in &event_queue.active {
        let airports_str: Vec<&str> = event
            .affected_airports
            .iter()
            .map(|a| a.display_name())
            .collect();

        if event.bonus_mission_available {
            toast.send(ToastEvent {
                message: format!(
                    "📰 {}: {} — Bonus missions available!",
                    event.kind.display_name(),
                    airports_str.join(", ")
                ),
                duration_secs: 4.0,
            });
        }
    }
}
