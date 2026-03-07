//! Weather forecasting — multi-day forecasts with accuracy that improves closer to the day.

use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct AirportForecast {
    pub airport: AirportId,
    pub predictions: Vec<DayForecast>,
}

#[derive(Clone, Debug)]
pub struct DayForecast {
    pub day_offset: u32, // 1 = tomorrow, 2 = day after, 3 = three days out
    pub predicted_weather: Weather,
    pub actual_weather: Option<Weather>,
    pub confidence: f32, // 0.0–1.0 — higher is more reliable
    pub predicted_wind_speed: f32,
    pub predicted_ceiling_ft: u32,
}

/// Forecasting resource — per-airport multi-day predictions.
#[derive(Resource, Clone, Debug, Default)]
pub struct WeatherForecasts {
    pub forecasts: Vec<AirportForecast>,
    pub last_generated_day: u32,
}

impl WeatherForecasts {
    pub fn get_forecast(&self, airport: AirportId, day_offset: u32) -> Option<&DayForecast> {
        self.forecasts
            .iter()
            .find(|f| f.airport == airport)
            .and_then(|f| f.predictions.iter().find(|d| d.day_offset == day_offset))
    }

    pub fn all_for_airport(&self, airport: AirportId) -> Option<&AirportForecast> {
        self.forecasts.iter().find(|f| f.airport == airport)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER
// ═══════════════════════════════════════════════════════════════════════════

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

fn random_weather(rng: &mut impl Rng) -> Weather {
    match rng.gen_range(0..7) {
        0 => Weather::Clear,
        1 => Weather::Cloudy,
        2 => Weather::Rain,
        3 => Weather::Windy,
        4 => Weather::Fog,
        5 => Weather::Snow,
        _ => Weather::Storm,
    }
}

fn maybe_scramble(base: Weather, confidence: f32, rng: &mut impl Rng) -> Weather {
    if rng.gen::<f32>() > confidence {
        random_weather(rng)
    } else {
        base
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Generate 3-day forecasts for every airport at day end.
pub fn generate_forecasts(
    mut day_end_events: EventReader<DayEndEvent>,
    mut forecasts: ResMut<WeatherForecasts>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let mut rng = rand::thread_rng();
        let day = calendar.total_days();
        forecasts.last_generated_day = day;
        forecasts.forecasts.clear();

        for &airport in AIRPORTS {
            let mut predictions = Vec::new();
            for offset in 1..=3 {
                let base = random_weather(&mut rng);
                // Confidence decreases the further out the prediction
                let confidence = match offset {
                    1 => 0.85,
                    2 => 0.60,
                    _ => 0.40,
                };
                let predicted = maybe_scramble(base, confidence, &mut rng);
                predictions.push(DayForecast {
                    day_offset: offset,
                    predicted_weather: predicted,
                    actual_weather: None,
                    confidence,
                    predicted_wind_speed: rng.gen_range(0.0..35.0),
                    predicted_ceiling_ft: match predicted {
                        Weather::Clear => 25000,
                        Weather::Cloudy => 8000,
                        Weather::Rain => 3000,
                        Weather::Fog => 500,
                        Weather::Snow => 2000,
                        Weather::Storm => 1000,
                        Weather::Windy => 15000,
                    },
                });
            }
            forecasts.forecasts.push(AirportForecast {
                airport,
                predictions,
            });
        }
    }
}

/// As each day passes, improve accuracy of near-term forecasts (day_offset shrinks).
pub fn update_forecast_accuracy(
    mut day_end_events: EventReader<DayEndEvent>,
    mut forecasts: ResMut<WeatherForecasts>,
    weather_state: Res<WeatherState>,
) {
    for _ev in day_end_events.read() {
        for airport_forecast in &mut forecasts.forecasts {
            for pred in &mut airport_forecast.predictions {
                if pred.day_offset == 1 {
                    // Tomorrow's forecast becomes today's reality
                    pred.actual_weather = Some(weather_state.current);
                }
                // Shift offsets down (closer days are more accurate now)
                pred.confidence = (pred.confidence + 0.1).min(1.0);
            }
            // Remove expired (day_offset 0) entries
            airport_forecast.predictions.retain(|p| p.day_offset > 0);
            // Decrement offsets
            for pred in &mut airport_forecast.predictions {
                pred.day_offset = pred.day_offset.saturating_sub(1).max(1);
            }
        }
    }
}

/// Weather briefing — pilots with high WeatherReading skill get better forecasts.
pub fn weather_briefing_interaction(
    player_input: Res<PlayerInput>,
    location: Res<PlayerLocation>,
    forecasts: Res<WeatherForecasts>,
    mut toast_events: EventWriter<ToastEvent>,
    mut claimed: ResMut<InteractionClaimed>,
) {
    if !player_input.interact || claimed.0 {
        return;
    }
    // Weather station in control tower
    if location.zone != MapZone::ControlTower {
        return;
    }
    claimed.0 = true;

    if let Some(fc) = forecasts.all_for_airport(location.airport) {
        let summary: Vec<String> = fc
            .predictions
            .iter()
            .map(|p| {
                format!(
                    "Day+{}: {:?} ({:.0}% conf)",
                    p.day_offset,
                    p.predicted_weather,
                    p.confidence * 100.0
                )
            })
            .collect();
        toast_events.send(ToastEvent {
            message: format!("Weather Briefing: {}", summary.join(" | ")),
            duration_secs: 5.0,
        });
    } else {
        toast_events.send(ToastEvent {
            message: "No forecast data available.".to_string(),
            duration_secs: 2.0,
        });
    }
}
