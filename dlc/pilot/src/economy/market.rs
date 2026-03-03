//! Dynamic market system — fuel prices, cargo rates, seasonal effects.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

const BASE_FUEL_PRICE: f32 = 10.0; // gold per unit

#[derive(Clone, Debug)]
pub struct AirportMarket {
    pub fuel_price: f32,
    pub fuel_supply: f32,   // 0.0–100.0
    pub cargo_rate_modifier: f32, // multiplier on base cargo rates
}

impl Default for AirportMarket {
    fn default() -> Self {
        Self {
            fuel_price: BASE_FUEL_PRICE,
            fuel_supply: 80.0,
            cargo_rate_modifier: 1.0,
        }
    }
}

/// Dynamic fuel and cargo market across all airports.
#[derive(Resource, Clone, Debug)]
pub struct MarketState {
    pub airports: HashMap<AirportId, AirportMarket>,
    pub global_fuel_trend: f32, // -1.0 to +1.0 shift
}

impl Default for MarketState {
    fn default() -> Self {
        let mut airports = HashMap::new();
        for &airport in AIRPORTS {
            airports.insert(airport, AirportMarket::default());
        }
        Self {
            airports,
            global_fuel_trend: 0.0,
        }
    }
}

impl MarketState {
    pub fn fuel_price(&self, airport: AirportId) -> f32 {
        self.airports
            .get(&airport)
            .map_or(BASE_FUEL_PRICE, |m| m.fuel_price)
    }

    pub fn cargo_rate(&self, airport: AirportId, base_rate: f32) -> f32 {
        let modifier = self
            .airports
            .get(&airport)
            .map_or(1.0, |m| m.cargo_rate_modifier);
        base_rate * modifier
    }

    pub fn bulk_discount(quantity: f32) -> f32 {
        if quantity >= 80.0 {
            0.85 // 15% discount
        } else if quantity >= 50.0 {
            0.92 // 8% discount
        } else {
            1.0
        }
    }
}

const AIRPORTS: &[AirportId] = &[
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

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Daily price fluctuations per airport.
pub fn update_market_prices(
    mut day_end_events: EventReader<DayEndEvent>,
    mut market: ResMut<MarketState>,
) {
    for _ev in day_end_events.read() {
        let mut rng = rand::thread_rng();

        // Global trend shifts slightly each day
        market.global_fuel_trend += rng.gen_range(-0.1..0.1);
        market.global_fuel_trend = market.global_fuel_trend.clamp(-1.0, 1.0);

        let global_trend = market.global_fuel_trend;
        for &airport in AIRPORTS {
            let am = market.airports.entry(airport).or_default();

            // Supply-driven price: low supply = higher price
            let supply_shift = rng.gen_range(-5.0..5.0);
            am.fuel_supply = (am.fuel_supply + supply_shift).clamp(10.0, 100.0);

            let supply_factor = 1.0 + (1.0 - am.fuel_supply / 100.0) * 0.5;
            let trend_factor = 1.0 + global_trend * 0.1;
            let random_factor = rng.gen_range(0.95..1.05);

            am.fuel_price = (BASE_FUEL_PRICE * supply_factor * trend_factor * random_factor)
                .clamp(5.0, 25.0);

            // Cargo rate fluctuation
            am.cargo_rate_modifier = rng.gen_range(0.8..1.3);
        }
    }
}

/// Seasonal effects on market — fuel costs more in winter, tourism routes pay more in summer.
pub fn seasonal_price_effects(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    mut market: ResMut<MarketState>,
) {
    for _ev in day_end_events.read() {
        let season_fuel_multiplier = match calendar.season {
            Season::Spring => 1.0,
            Season::Summer => 0.95,
            Season::Fall => 1.05,
            Season::Winter => 1.2, // fuel more expensive in winter
        };

        let season_cargo_multiplier = match calendar.season {
            Season::Spring => 1.0,
            Season::Summer => 1.2, // tourism routes pay more
            Season::Fall => 1.1,
            Season::Winter => 0.9,
        };

        for am in market.airports.values_mut() {
            am.fuel_price *= season_fuel_multiplier;
            am.fuel_price = am.fuel_price.clamp(5.0, 30.0);
            am.cargo_rate_modifier *= season_cargo_multiplier;
            am.cargo_rate_modifier = am.cargo_rate_modifier.clamp(0.5, 2.0);
        }
    }
}

/// Cargo rates vary by distance, urgency, and type.
pub fn cargo_distance_pricing(
    origin: AirportId,
    destination: AirportId,
    cargo_type_multiplier: f32,
    market: &MarketState,
) -> f32 {
    let distance = airport_distance(origin, destination);
    let base_rate = distance * 0.5; // 0.5g per NM base
    let market_rate = market.cargo_rate(destination, base_rate);
    market_rate * cargo_type_multiplier
}
