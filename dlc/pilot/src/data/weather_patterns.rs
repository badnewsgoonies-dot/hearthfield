//! Weather pattern definitions for each airport, with seasonal variations.

use crate::shared::*;

/// Describes the baseline weather behavior and seasonal variations at an airport.
#[derive(Clone, Debug)]
pub struct WeatherPattern {
    pub base_conditions: Vec<WeightedWeather>,
    pub seasonal_overrides: [(Season, Vec<WeightedWeather>); 4],
    pub special_events: Vec<SpecialWeatherEvent>,
    pub avg_wind_knots: f32,
    pub fog_chance: f32,
}

/// A weather condition with a probability weight (0.0–1.0).
#[derive(Clone, Debug)]
pub struct WeightedWeather {
    pub weather: Weather,
    pub weight: f32,
}

impl WeightedWeather {
    pub fn new(weather: Weather, weight: f32) -> Self {
        Self { weather, weight }
    }
}

/// Rare or extreme weather events that can occur at specific airports.
#[derive(Clone, Debug)]
pub struct SpecialWeatherEvent {
    pub name: String,
    pub description: String,
    pub weather: Weather,
    pub chance_per_day: f32,
    pub seasons: Vec<Season>,
}

impl SpecialWeatherEvent {
    fn new(name: &str, desc: &str, weather: Weather, chance: f32, seasons: Vec<Season>) -> Self {
        Self {
            name: name.to_string(),
            description: desc.to_string(),
            weather,
            chance_per_day: chance,
            seasons,
        }
    }
}

fn base_seasonal(
    spring: Vec<WeightedWeather>,
    summer: Vec<WeightedWeather>,
    fall: Vec<WeightedWeather>,
    winter: Vec<WeightedWeather>,
) -> [(Season, Vec<WeightedWeather>); 4] {
    [
        (Season::Spring, spring),
        (Season::Summer, summer),
        (Season::Fall, fall),
        (Season::Winter, winter),
    ]
}

/// Returns the weather pattern definition for a given airport.
pub fn get_airport_weather_pattern(airport: AirportId) -> WeatherPattern {
    match airport {
        AirportId::HomeBase => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Clear, 0.45),
                WeightedWeather::new(Weather::Cloudy, 0.30),
                WeightedWeather::new(Weather::Rain, 0.15),
                WeightedWeather::new(Weather::Windy, 0.10),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Clear, 0.40),
                    WeightedWeather::new(Weather::Rain, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.60),
                    WeightedWeather::new(Weather::Cloudy, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Cloudy, 0.35),
                    WeightedWeather::new(Weather::Rain, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.20),
                    WeightedWeather::new(Weather::Cloudy, 0.40),
                ],
            ),
            special_events: vec![],
            avg_wind_knots: 8.0,
            fog_chance: 0.05,
        },

        AirportId::Grandcity => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Cloudy, 0.35),
                WeightedWeather::new(Weather::Clear, 0.25),
                WeightedWeather::new(Weather::Rain, 0.20),
                WeightedWeather::new(Weather::Windy, 0.10),
                WeightedWeather::new(Weather::Storm, 0.10),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Rain, 0.30),
                    WeightedWeather::new(Weather::Cloudy, 0.30),
                ],
                vec![
                    WeightedWeather::new(Weather::Storm, 0.20),
                    WeightedWeather::new(Weather::Clear, 0.30),
                ],
                vec![
                    WeightedWeather::new(Weather::Cloudy, 0.40),
                    WeightedWeather::new(Weather::Windy, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.25),
                    WeightedWeather::new(Weather::Windy, 0.20),
                ],
            ),
            special_events: vec![
                SpecialWeatherEvent::new(
                    "Urban Heat Island",
                    "Extreme heat buildup causes severe thermal turbulence over the city",
                    Weather::Clear,
                    0.08,
                    vec![Season::Summer],
                ),
                SpecialWeatherEvent::new(
                    "Summer Thunderstorm",
                    "Sudden violent thunderstorms with heavy rain and hail",
                    Weather::Storm,
                    0.12,
                    vec![Season::Summer],
                ),
            ],
            avg_wind_knots: 12.0,
            fog_chance: 0.10,
        },

        AirportId::Sunhaven => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Clear, 0.50),
                WeightedWeather::new(Weather::Cloudy, 0.20),
                WeightedWeather::new(Weather::Rain, 0.15),
                WeightedWeather::new(Weather::Windy, 0.10),
                WeightedWeather::new(Weather::Storm, 0.05),
            ],
            seasonal_overrides: base_seasonal(
                vec![WeightedWeather::new(Weather::Clear, 0.55)],
                vec![
                    WeightedWeather::new(Weather::Storm, 0.15),
                    WeightedWeather::new(Weather::Rain, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Rain, 0.25),
                    WeightedWeather::new(Weather::Storm, 0.10),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.50),
                    WeightedWeather::new(Weather::Windy, 0.15),
                ],
            ),
            special_events: vec![
                SpecialWeatherEvent::new(
                    "Tropical Storm",
                    "Sudden intense storms with strong winds and torrential rain",
                    Weather::Storm,
                    0.10,
                    vec![Season::Summer, Season::Fall],
                ),
                SpecialWeatherEvent::new(
                    "Hurricane Warning",
                    "Rare but devastating hurricane forces airport closure",
                    Weather::Storm,
                    0.02,
                    vec![Season::Summer],
                ),
            ],
            avg_wind_knots: 10.0,
            fog_chance: 0.03,
        },

        AirportId::Frostpeak => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Snow, 0.30),
                WeightedWeather::new(Weather::Cloudy, 0.25),
                WeightedWeather::new(Weather::Clear, 0.20),
                WeightedWeather::new(Weather::Windy, 0.15),
                WeightedWeather::new(Weather::Fog, 0.10),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Clear, 0.30),
                    WeightedWeather::new(Weather::Snow, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.45),
                    WeightedWeather::new(Weather::Cloudy, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.25),
                    WeightedWeather::new(Weather::Windy, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.50),
                    WeightedWeather::new(Weather::Windy, 0.20),
                ],
            ),
            special_events: vec![
                SpecialWeatherEvent::new(
                    "Whiteout Blizzard",
                    "Zero visibility blizzard shuts down all flight operations",
                    Weather::Snow,
                    0.08,
                    vec![Season::Winter],
                ),
                SpecialWeatherEvent::new(
                    "Mountain Wave",
                    "Dangerous standing wave turbulence on the lee side of peaks",
                    Weather::Windy,
                    0.12,
                    vec![Season::Winter, Season::Fall],
                ),
            ],
            avg_wind_knots: 18.0,
            fog_chance: 0.12,
        },

        AirportId::Ironforge => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Cloudy, 0.40),
                WeightedWeather::new(Weather::Rain, 0.25),
                WeightedWeather::new(Weather::Clear, 0.15),
                WeightedWeather::new(Weather::Fog, 0.10),
                WeightedWeather::new(Weather::Windy, 0.10),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Rain, 0.30),
                    WeightedWeather::new(Weather::Cloudy, 0.35),
                ],
                vec![
                    WeightedWeather::new(Weather::Cloudy, 0.35),
                    WeightedWeather::new(Weather::Clear, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Fog, 0.20),
                    WeightedWeather::new(Weather::Rain, 0.30),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.15),
                    WeightedWeather::new(Weather::Cloudy, 0.40),
                ],
            ),
            special_events: vec![SpecialWeatherEvent::new(
                "Industrial Smog",
                "Heavy industrial haze reduces visibility to near IFR minimums",
                Weather::Fog,
                0.10,
                vec![Season::Summer, Season::Fall],
            )],
            avg_wind_knots: 8.0,
            fog_chance: 0.15,
        },

        AirportId::Cloudmere => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Fog, 0.35),
                WeightedWeather::new(Weather::Cloudy, 0.25),
                WeightedWeather::new(Weather::Rain, 0.20),
                WeightedWeather::new(Weather::Windy, 0.10),
                WeightedWeather::new(Weather::Clear, 0.10),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Fog, 0.40),
                    WeightedWeather::new(Weather::Rain, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Fog, 0.25),
                    WeightedWeather::new(Weather::Clear, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Fog, 0.45),
                    WeightedWeather::new(Weather::Rain, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Fog, 0.35),
                    WeightedWeather::new(Weather::Snow, 0.10),
                ],
            ),
            special_events: vec![SpecialWeatherEvent::new(
                "Sea Fog Bank",
                "Dense advection fog rolls in from the ocean, lasting for days",
                Weather::Fog,
                0.15,
                vec![Season::Spring, Season::Fall],
            )],
            avg_wind_knots: 14.0,
            fog_chance: 0.40,
        },

        AirportId::Stormwatch => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Storm, 0.25),
                WeightedWeather::new(Weather::Windy, 0.25),
                WeightedWeather::new(Weather::Rain, 0.20),
                WeightedWeather::new(Weather::Cloudy, 0.15),
                WeightedWeather::new(Weather::Clear, 0.15),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Storm, 0.30),
                    WeightedWeather::new(Weather::Rain, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Storm, 0.35),
                    WeightedWeather::new(Weather::Windy, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Storm, 0.25),
                    WeightedWeather::new(Weather::Rain, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.20),
                    WeightedWeather::new(Weather::Windy, 0.30),
                ],
            ),
            special_events: vec![
                SpecialWeatherEvent::new(
                    "Supercell Thunderstorm",
                    "Massive rotating thunderstorm with potential tornado activity",
                    Weather::Storm,
                    0.10,
                    vec![Season::Spring, Season::Summer],
                ),
                SpecialWeatherEvent::new(
                    "Research Storm",
                    "Intentionally monitored severe weather event for scientific data",
                    Weather::Storm,
                    0.15,
                    vec![Season::Spring, Season::Summer, Season::Fall],
                ),
                SpecialWeatherEvent::new(
                    "Ice Storm",
                    "Freezing rain coats everything in ice, extremely dangerous for aircraft",
                    Weather::Snow,
                    0.06,
                    vec![Season::Winter],
                ),
            ],
            avg_wind_knots: 22.0,
            fog_chance: 0.08,
        },

        AirportId::Duskhollow => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Clear, 0.55),
                WeightedWeather::new(Weather::Windy, 0.20),
                WeightedWeather::new(Weather::Cloudy, 0.15),
                WeightedWeather::new(Weather::Rain, 0.05),
                WeightedWeather::new(Weather::Storm, 0.05),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Clear, 0.55),
                    WeightedWeather::new(Weather::Windy, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.60),
                    WeightedWeather::new(Weather::Windy, 0.15),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.50),
                    WeightedWeather::new(Weather::Windy, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.45),
                    WeightedWeather::new(Weather::Cloudy, 0.25),
                ],
            ),
            special_events: vec![
                SpecialWeatherEvent::new(
                    "Sandstorm",
                    "A wall of sand reduces visibility to zero and damages aircraft",
                    Weather::Windy,
                    0.08,
                    vec![Season::Spring, Season::Summer],
                ),
                SpecialWeatherEvent::new(
                    "Heat Shimmer",
                    "Extreme heat creates visual distortion on approach, misleading pilots",
                    Weather::Clear,
                    0.15,
                    vec![Season::Summer],
                ),
            ],
            avg_wind_knots: 10.0,
            fog_chance: 0.02,
        },

        AirportId::Windport => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Clear, 0.40),
                WeightedWeather::new(Weather::Cloudy, 0.20),
                WeightedWeather::new(Weather::Rain, 0.20),
                WeightedWeather::new(Weather::Windy, 0.10),
                WeightedWeather::new(Weather::Storm, 0.10),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Clear, 0.45),
                    WeightedWeather::new(Weather::Rain, 0.15),
                ],
                vec![
                    WeightedWeather::new(Weather::Storm, 0.15),
                    WeightedWeather::new(Weather::Clear, 0.35),
                ],
                vec![
                    WeightedWeather::new(Weather::Rain, 0.25),
                    WeightedWeather::new(Weather::Storm, 0.15),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.40),
                    WeightedWeather::new(Weather::Windy, 0.20),
                ],
            ),
            special_events: vec![SpecialWeatherEvent::new(
                "Typhoon Warning",
                "Seasonal typhoon forces evacuation of the island chain",
                Weather::Storm,
                0.04,
                vec![Season::Summer, Season::Fall],
            )],
            avg_wind_knots: 15.0,
            fog_chance: 0.05,
        },

        AirportId::Skyreach => WeatherPattern {
            base_conditions: vec![
                WeightedWeather::new(Weather::Clear, 0.45),
                WeightedWeather::new(Weather::Windy, 0.25),
                WeightedWeather::new(Weather::Cloudy, 0.15),
                WeightedWeather::new(Weather::Snow, 0.10),
                WeightedWeather::new(Weather::Fog, 0.05),
            ],
            seasonal_overrides: base_seasonal(
                vec![
                    WeightedWeather::new(Weather::Clear, 0.40),
                    WeightedWeather::new(Weather::Windy, 0.25),
                ],
                vec![
                    WeightedWeather::new(Weather::Clear, 0.55),
                    WeightedWeather::new(Weather::Windy, 0.20),
                ],
                vec![
                    WeightedWeather::new(Weather::Windy, 0.30),
                    WeightedWeather::new(Weather::Clear, 0.30),
                ],
                vec![
                    WeightedWeather::new(Weather::Snow, 0.30),
                    WeightedWeather::new(Weather::Windy, 0.30),
                ],
            ),
            special_events: vec![
                SpecialWeatherEvent::new(
                    "Mountain Rotor",
                    "Violent rotational turbulence near the mountain runway",
                    Weather::Windy,
                    0.10,
                    vec![Season::Winter, Season::Spring],
                ),
                SpecialWeatherEvent::new(
                    "Clear Air Turbulence",
                    "Invisible turbulence in perfectly clear skies at high altitude",
                    Weather::Clear,
                    0.08,
                    vec![Season::Spring, Season::Summer, Season::Fall, Season::Winter],
                ),
            ],
            avg_wind_knots: 20.0,
            fog_chance: 0.04,
        },
    }
}
