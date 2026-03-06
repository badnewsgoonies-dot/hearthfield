//! Airport service interactions — hotel, car rental, lounge, cargo, customs, briefing, map.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── Service Types ───────────────────────────────────────────────────────

/// All available airport services.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AirportService {
    Hotel,
    CarRental,
    Lounge,
    CargoHandling,
    Customs,
    WeatherBriefing,
    AirportMap,
}

impl AirportService {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Hotel => "Airport Hotel",
            Self::CarRental => "Car Rental",
            Self::Lounge => "VIP Lounge",
            Self::CargoHandling => "Cargo Handling",
            Self::Customs => "Customs & Immigration",
            Self::WeatherBriefing => "Weather Briefing Office",
            Self::AirportMap => "Airport Map",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Hotel => "Rest overnight and restore stamina.",
            Self::CarRental => "Rent a car to explore the city faster.",
            Self::Lounge => "Relax in the premium lounge with food and rest bonuses.",
            Self::CargoHandling => "Professional crew handles cargo loading/unloading.",
            Self::Customs => "Clear customs for international flights.",
            Self::WeatherBriefing => "Get a detailed forecast from the meteorologist.",
            Self::AirportMap => "View the interactive terminal map.",
        }
    }

    /// Base cost in gold. Zero means free.
    pub fn base_cost(&self) -> u32 {
        match self {
            Self::Hotel => 80,
            Self::CarRental => 50,
            Self::Lounge => 120,
            Self::CargoHandling => 30,
            Self::Customs => 0,
            Self::WeatherBriefing => 0,
            Self::AirportMap => 0,
        }
    }

    /// All services available at every airport.
    pub fn all() -> &'static [AirportService] {
        &[
            Self::Hotel,
            Self::CarRental,
            Self::Lounge,
            Self::CargoHandling,
            Self::Customs,
            Self::WeatherBriefing,
            Self::AirportMap,
        ]
    }
}

// ─── Service Availability per Airport ────────────────────────────────────

/// Which services are available at a given airport.
pub fn services_at(airport: AirportId) -> Vec<AirportService> {
    let mut out = vec![AirportService::WeatherBriefing, AirportService::AirportMap];

    match airport {
        AirportId::HomeBase => {
            out.push(AirportService::CargoHandling);
        }
        AirportId::Grandcity => {
            out.push(AirportService::Hotel);
            out.push(AirportService::CarRental);
            out.push(AirportService::Lounge);
            out.push(AirportService::CargoHandling);
            out.push(AirportService::Customs);
        }
        AirportId::Sunhaven => {
            out.push(AirportService::Hotel);
            out.push(AirportService::CarRental);
            out.push(AirportService::Lounge);
            out.push(AirportService::Customs);
        }
        AirportId::Skyreach => {
            out.push(AirportService::Hotel);
            out.push(AirportService::Lounge);
            out.push(AirportService::CargoHandling);
            out.push(AirportService::Customs);
        }
        AirportId::Ironforge => {
            out.push(AirportService::CargoHandling);
            out.push(AirportService::Hotel);
        }
        AirportId::Frostpeak => {
            out.push(AirportService::Hotel);
        }
        AirportId::Windport => {
            out.push(AirportService::Hotel);
            out.push(AirportService::CarRental);
            out.push(AirportService::Customs);
        }
        AirportId::Cloudmere => {
            out.push(AirportService::Hotel);
            out.push(AirportService::Lounge);
        }
        AirportId::Duskhollow => {
            out.push(AirportService::Hotel);
            out.push(AirportService::CarRental);
        }
        AirportId::Stormwatch => {
            out.push(AirportService::Hotel);
            out.push(AirportService::CargoHandling);
        }
    }

    out
}

// ─── Service Effect Results ──────────────────────────────────────────────

/// Result of using a service.
#[derive(Clone, Debug)]
pub struct ServiceResult {
    pub message: String,
    pub gold_cost: u32,
    pub stamina_restored: f32,
    pub speed_bonus: f32,
    pub time_hours: f32,
}

// ─── Cargo Handling Efficiency ───────────────────────────────────────────

/// Cargo loading/unloading efficiency multiplier per airport.
pub fn cargo_efficiency(airport: AirportId) -> f32 {
    match airport {
        AirportId::Grandcity => 1.5,
        AirportId::Ironforge => 1.4,
        AirportId::Skyreach => 1.3,
        AirportId::Stormwatch => 1.1,
        AirportId::HomeBase => 1.0,
        AirportId::Windport | AirportId::Sunhaven => 1.1,
        AirportId::Cloudmere | AirportId::Duskhollow => 0.9,
        AirportId::Frostpeak => 0.8,
    }
}

/// Customs processing time in game-hours.
pub fn customs_processing_time(airport: AirportId) -> f32 {
    match airport {
        AirportId::Grandcity => 1.0,
        AirportId::Skyreach => 0.5,
        AirportId::Sunhaven => 1.5,
        AirportId::Windport => 1.2,
        _ => 2.0,
    }
}

// ─── Service state resource ──────────────────────────────────────────────

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AirportServiceState {
    pub car_rental_active: bool,
    pub car_rental_speed_bonus: f32,
    pub lounge_rested: bool,
    pub customs_cleared: bool,
    pub last_briefing_day: u32,
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Use the airport hotel to restore stamina overnight.
pub fn use_hotel(
    input: Res<PlayerInput>,
    location: Res<PlayerLocation>,
    mut pilot: ResMut<PilotState>,
    mut gold: ResMut<Gold>,
    mut toast: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    if !input.interact || location.zone != MapZone::Lounge {
        return;
    }

    let available = services_at(location.airport);
    if !available.contains(&AirportService::Hotel) {
        return;
    }

    let cost = hotel_cost(location.airport);
    if gold.amount < cost {
        toast.send(ToastEvent {
            message: format!("Not enough gold for hotel (need {}G).", cost),
            duration_secs: 3.0,
        });
        return;
    }

    gold.amount -= cost;
    gold_events.send(GoldChangeEvent {
        amount: -(cost as i32),
        reason: "Hotel stay".into(),
    });

    pilot.stamina = pilot.max_stamina;

    toast.send(ToastEvent {
        message: format!(
            "Stayed at {} hotel — stamina fully restored! (-{}G)",
            location.airport.display_name(),
            cost
        ),
        duration_secs: 4.0,
    });
}

fn hotel_cost(airport: AirportId) -> u32 {
    match airport {
        AirportId::HomeBase => 40,
        AirportId::Grandcity | AirportId::Skyreach => 150,
        AirportId::Sunhaven => 120,
        AirportId::Cloudmere | AirportId::Windport => 100,
        _ => 80,
    }
}

/// Rent a car for faster city exploration.
pub fn use_car_rental(
    input: Res<PlayerInput>,
    location: Res<PlayerLocation>,
    mut gold: ResMut<Gold>,
    mut service_state: ResMut<AirportServiceState>,
    mut toast: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    if !input.hotbar_2 || service_state.car_rental_active {
        return;
    }

    let available = services_at(location.airport);
    if !available.contains(&AirportService::CarRental) {
        return;
    }

    let cost = AirportService::CarRental.base_cost();
    if gold.amount < cost {
        toast.send(ToastEvent {
            message: format!("Not enough gold for car rental (need {}G).", cost),
            duration_secs: 3.0,
        });
        return;
    }

    gold.amount -= cost;
    gold_events.send(GoldChangeEvent {
        amount: -(cost as i32),
        reason: "Car rental".into(),
    });

    service_state.car_rental_active = true;
    service_state.car_rental_speed_bonus = 1.5;

    toast.send(ToastEvent {
        message: format!("Car rented — exploration speed increased! (-{}G)", cost),
        duration_secs: 3.0,
    });
}

/// Access the VIP lounge for rest and food bonuses.
pub fn use_lounge(
    input: Res<PlayerInput>,
    location: Res<PlayerLocation>,
    mut pilot: ResMut<PilotState>,
    mut gold: ResMut<Gold>,
    mut service_state: ResMut<AirportServiceState>,
    mut toast: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    if !input.hotbar_3 || service_state.lounge_rested {
        return;
    }

    let available = services_at(location.airport);
    if !available.contains(&AirportService::Lounge) {
        return;
    }

    let cost = AirportService::Lounge.base_cost();
    if gold.amount < cost {
        toast.send(ToastEvent {
            message: format!("Not enough gold for lounge access (need {}G).", cost),
            duration_secs: 3.0,
        });
        return;
    }

    gold.amount -= cost;
    gold_events.send(GoldChangeEvent {
        amount: -(cost as i32),
        reason: "VIP lounge".into(),
    });

    // Lounge restores 40% stamina
    pilot.stamina = (pilot.stamina + pilot.max_stamina * 0.4).min(pilot.max_stamina);
    service_state.lounge_rested = true;

    toast.send(ToastEvent {
        message: format!("VIP lounge — rested and refreshed! (-{}G)", cost),
        duration_secs: 3.0,
    });
}

/// Get a weather briefing from the meteorologist NPC.
pub fn request_weather_briefing(
    input: Res<PlayerInput>,
    location: Res<PlayerLocation>,
    calendar: Res<Calendar>,
    weather: Res<WeatherState>,
    mut service_state: ResMut<AirportServiceState>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !input.hotbar_4 {
        return;
    }

    let today = calendar.total_days();
    if service_state.last_briefing_day == today {
        return; // already briefed today
    }

    service_state.last_briefing_day = today;

    let forecast_text = weather
        .forecast
        .iter()
        .enumerate()
        .map(|(i, w)| format!("Day+{}: {:?}", i + 1, w))
        .collect::<Vec<_>>()
        .join(", ");

    let briefing = format!(
        "BRIEFING — {}: {:?}, wind {:.0}kt/{:.0}°, vis {:.1}nm, ceil {}ft. Forecast: {}",
        location.airport.display_name(),
        weather.current,
        weather.wind_speed_knots,
        weather.wind_direction_deg,
        weather.visibility_nm,
        weather.ceiling_ft,
        if forecast_text.is_empty() {
            "N/A".to_string()
        } else {
            forecast_text
        },
    );

    toast.send(ToastEvent {
        message: briefing,
        duration_secs: 8.0,
    });
}

/// Reset service state when arriving at a new airport.
pub fn reset_services_on_arrival(
    mut arrival_events: EventReader<AirportArrivalEvent>,
    mut service_state: ResMut<AirportServiceState>,
) {
    for _evt in arrival_events.read() {
        service_state.car_rental_active = false;
        service_state.car_rental_speed_bonus = 1.0;
        service_state.lounge_rested = false;
        service_state.customs_cleared = false;
    }
}
