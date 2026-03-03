//! Weather domain — aviation weather generation, turbulence, visibility.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

pub mod forecasting;
pub mod effects;

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<forecasting::WeatherForecasts>()
            .init_resource::<effects::IcingState>()
            .init_resource::<effects::LightningTimer>()
            .add_systems(
                Update,
                (
                    update_weather.run_if(in_state(GameState::Playing)),
                    generate_daily_weather.run_if(in_state(GameState::Playing)),
                    forecasting::generate_forecasts.run_if(in_state(GameState::Playing)),
                    forecasting::update_forecast_accuracy.run_if(in_state(GameState::Playing)),
                    forecasting::weather_briefing_interaction.run_if(in_state(GameState::Playing)),
                    effects::apply_weather_visual_effects.run_if(in_state(GameState::Playing)),
                    effects::apply_turbulence_effects.run_if(in_state(GameState::Playing)),
                    effects::apply_wind_effects.run_if(in_state(GameState::Playing)),
                    effects::apply_icing_effects.run_if(in_state(GameState::Playing)),
                    effects::lightning_flash_effects.run_if(in_state(GameState::Playing)),
                    effects::animate_weather_particles.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn update_weather(
    time: Res<Time>,
    mut weather_state: ResMut<WeatherState>,
    calendar: Res<Calendar>,
) {
    // Wind varies slightly over time
    let wind_variation = (calendar.time_of_day_secs * 0.01).sin() as f32 * 5.0;
    weather_state.wind_speed_knots = (weather_state.wind_speed_knots + wind_variation * time.delta_secs()).max(0.0);

    // Update turbulence based on weather
    weather_state.turbulence_level = match weather_state.current {
        Weather::Clear => TurbulenceLevel::None,
        Weather::Cloudy => TurbulenceLevel::Light,
        Weather::Windy => TurbulenceLevel::Moderate,
        Weather::Rain => TurbulenceLevel::Light,
        Weather::Fog => TurbulenceLevel::None,
        Weather::Snow => TurbulenceLevel::Light,
        Weather::Storm => TurbulenceLevel::Severe,
    };

    // Visibility
    weather_state.visibility_nm = match weather_state.current {
        Weather::Clear => 10.0,
        Weather::Cloudy => 8.0,
        Weather::Windy => 9.0,
        Weather::Rain => 5.0,
        Weather::Fog => 1.0,
        Weather::Snow => 3.0,
        Weather::Storm => 2.0,
    };
}

fn generate_daily_weather(
    mut day_end_events: EventReader<DayEndEvent>,
    mut weather_state: ResMut<WeatherState>,
    mut weather_events: EventWriter<WeatherChangeEvent>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let mut rng = rand::thread_rng();

        // Season-influenced weather generation
        let weather = match calendar.season {
            Season::Spring => match rng.gen_range(0..10) {
                0..=3 => Weather::Clear,
                4..=6 => Weather::Cloudy,
                7..=8 => Weather::Rain,
                _ => Weather::Windy,
            },
            Season::Summer => match rng.gen_range(0..10) {
                0..=5 => Weather::Clear,
                6..=7 => Weather::Cloudy,
                8 => Weather::Storm,
                _ => Weather::Windy,
            },
            Season::Fall => match rng.gen_range(0..10) {
                0..=2 => Weather::Clear,
                3..=5 => Weather::Cloudy,
                6..=7 => Weather::Rain,
                8 => Weather::Fog,
                _ => Weather::Windy,
            },
            Season::Winter => match rng.gen_range(0..10) {
                0..=1 => Weather::Clear,
                2..=3 => Weather::Cloudy,
                4..=6 => Weather::Snow,
                7 => Weather::Fog,
                8 => Weather::Storm,
                _ => Weather::Windy,
            },
        };

        weather_state.current = weather;
        weather_state.wind_speed_knots = rng.gen_range(0.0..30.0);
        weather_state.wind_direction_deg = rng.gen_range(0.0..360.0);
        weather_state.ceiling_ft = match weather {
            Weather::Clear => 25000,
            Weather::Cloudy => 8000,
            Weather::Rain => 3000,
            Weather::Fog => 500,
            Weather::Snow => 2000,
            Weather::Storm => 1000,
            Weather::Windy => 15000,
        };

        // Generate 3-day forecast
        weather_state.forecast.clear();
        for _ in 0..3 {
            weather_state.forecast.push(match rng.gen_range(0..7) {
                0 => Weather::Clear,
                1 => Weather::Cloudy,
                2 => Weather::Rain,
                3 => Weather::Windy,
                4 => Weather::Fog,
                5 => Weather::Snow,
                _ => Weather::Storm,
            });
        }

        weather_events.send(WeatherChangeEvent {
            new_weather: weather,
        });
    }
}
