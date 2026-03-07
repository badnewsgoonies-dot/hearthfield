//! Special mission types — SAR, medevac, celebrity, air race, photography, etc.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialMissionType {
    SearchAndRescue,
    MedicalEvacuation,
    CelebrityTransport,
    AirRace,
    AerialPhotography,
    StormChasing,
    WildlifeSurvey,
    HistoricalDelivery,
    HumanitarianAid,
    NightMailRun,
}

impl SpecialMissionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            SpecialMissionType::SearchAndRescue => "Search & Rescue",
            SpecialMissionType::MedicalEvacuation => "Medical Evacuation",
            SpecialMissionType::CelebrityTransport => "Celebrity Transport",
            SpecialMissionType::AirRace => "Air Race",
            SpecialMissionType::AerialPhotography => "Aerial Photography",
            SpecialMissionType::StormChasing => "Storm Chasing",
            SpecialMissionType::WildlifeSurvey => "Wildlife Survey",
            SpecialMissionType::HistoricalDelivery => "Historical Aircraft Delivery",
            SpecialMissionType::HumanitarianAid => "Humanitarian Aid",
            SpecialMissionType::NightMailRun => "Night Mail Run",
        }
    }

    pub fn base_reward_gold(&self) -> u32 {
        match self {
            SpecialMissionType::SearchAndRescue => 800,
            SpecialMissionType::MedicalEvacuation => 1000,
            SpecialMissionType::CelebrityTransport => 1500,
            SpecialMissionType::AirRace => 600,
            SpecialMissionType::AerialPhotography => 500,
            SpecialMissionType::StormChasing => 900,
            SpecialMissionType::WildlifeSurvey => 400,
            SpecialMissionType::HistoricalDelivery => 700,
            SpecialMissionType::HumanitarianAid => 300,
            SpecialMissionType::NightMailRun => 550,
        }
    }

    pub fn base_reward_xp(&self) -> u32 {
        match self {
            SpecialMissionType::SearchAndRescue => 150,
            SpecialMissionType::MedicalEvacuation => 200,
            SpecialMissionType::CelebrityTransport => 120,
            SpecialMissionType::AirRace => 100,
            SpecialMissionType::AerialPhotography => 80,
            SpecialMissionType::StormChasing => 180,
            SpecialMissionType::WildlifeSurvey => 70,
            SpecialMissionType::HistoricalDelivery => 130,
            SpecialMissionType::HumanitarianAid => 160,
            SpecialMissionType::NightMailRun => 110,
        }
    }

    pub fn required_rank(&self) -> PilotRank {
        match self {
            SpecialMissionType::WildlifeSurvey => PilotRank::Private,
            SpecialMissionType::AerialPhotography => PilotRank::Private,
            SpecialMissionType::NightMailRun => PilotRank::Commercial,
            SpecialMissionType::AirRace => PilotRank::Commercial,
            SpecialMissionType::SearchAndRescue => PilotRank::Commercial,
            SpecialMissionType::StormChasing => PilotRank::Senior,
            SpecialMissionType::HumanitarianAid => PilotRank::Senior,
            SpecialMissionType::HistoricalDelivery => PilotRank::Senior,
            SpecialMissionType::MedicalEvacuation => PilotRank::Captain,
            SpecialMissionType::CelebrityTransport => PilotRank::Captain,
        }
    }
}

/// Search pattern for SAR missions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchPattern {
    ParallelTrack,
    ExpandingSquare,
    SectorSearch,
    CreepingLine,
}

impl SearchPattern {
    pub fn display_name(&self) -> &'static str {
        match self {
            SearchPattern::ParallelTrack => "Parallel Track",
            SearchPattern::ExpandingSquare => "Expanding Square",
            SearchPattern::SectorSearch => "Sector Search",
            SearchPattern::CreepingLine => "Creeping Line",
        }
    }

    pub fn detection_bonus(&self) -> f32 {
        match self {
            SearchPattern::ParallelTrack => 1.0,
            SearchPattern::ExpandingSquare => 1.2,
            SearchPattern::SectorSearch => 0.8,
            SearchPattern::CreepingLine => 1.1,
        }
    }
}

/// Waypoint for air race or photography missions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MissionWaypoint {
    pub name: String,
    pub distance_from_start_nm: f32,
    pub required_altitude_ft: Option<f32>,
    pub altitude_tolerance_ft: f32,
    pub passed: bool,
}

impl MissionWaypoint {
    pub fn new(name: &str, dist: f32, alt: Option<f32>) -> Self {
        Self {
            name: name.to_string(),
            distance_from_start_nm: dist,
            required_altitude_ft: alt,
            altitude_tolerance_ft: 500.0,
            passed: false,
        }
    }
}

/// Active special mission state.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpecialMissionState {
    pub active_type: Option<SpecialMissionType>,
    pub search_progress: f32, // 0.0-1.0 for SAR
    pub survivors_found: u32,
    pub search_pattern: Option<SearchPattern>,
    pub race_waypoints: Vec<MissionWaypoint>,
    pub photo_waypoints: Vec<MissionWaypoint>,
    pub time_bonus_secs: f32, // Medevac speed bonus timer
    pub celebrity_demands_met: u32,
    pub celebrity_demands_total: u32,
    pub storm_data_collected: f32, // 0.0-1.0
    pub wildlife_counted: u32,
    pub cargo_fragility: f32, // Historical delivery
    pub aid_packages_remaining: u32,
    pub mail_stops_completed: u32,
    pub mail_stops_total: u32,
    pub completion_timer: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// TEMPLATE GENERATION
// ═══════════════════════════════════════════════════════════════════════════

/// Generate special mission definitions.
pub fn get_special_missions() -> Vec<MissionDef> {
    vec![
        // Search and Rescue
        MissionDef {
            id: "special_sar_cloudmere".into(),
            title: "Mountain Search & Rescue".into(),
            description: "Fly a search pattern over the Cloudmere highlands. Locate missing hikers and radio coordinates to ground teams.".into(),
            mission_type: MissionType::Rescue,
            origin: AirportId::HomeBase,
            destination: AirportId::Cloudmere,
            reward_gold: 800,
            reward_xp: 150,
            time_limit_hours: Some(4),
            required_rank: PilotRank::Commercial,
            required_aircraft_class: Some(AircraftClass::TwinProp),
            passenger_count: 2,
            cargo_kg: 100.0,
            bonus_conditions: vec![BonusCondition::OnTime],
            difficulty: MissionDifficulty::Hard,
        },
        // Medical Evacuation
        MissionDef {
            id: "special_medevac_frostpeak".into(),
            title: "Critical Medevac".into(),
            description: "Time-critical patient transport from Frostpeak. Every minute counts — fly fast and land smooth.".into(),
            mission_type: MissionType::Medical,
            origin: AirportId::Frostpeak,
            destination: AirportId::Grandcity,
            reward_gold: 1000,
            reward_xp: 200,
            time_limit_hours: Some(2),
            required_rank: PilotRank::Captain,
            required_aircraft_class: Some(AircraftClass::LightJet),
            passenger_count: 3,
            cargo_kg: 50.0,
            bonus_conditions: vec![BonusCondition::OnTime, BonusCondition::PerfectLanding],
            difficulty: MissionDifficulty::Expert,
        },
        // Celebrity Transport
        MissionDef {
            id: "special_celebrity_sunhaven".into(),
            title: "Pop Star to Sunhaven".into(),
            description: "Fly international pop star Cassandra Belle to Sunhaven. She demands smooth flying, privacy, and champagne. Paparazzi awaiting at destination.".into(),
            mission_type: MissionType::VIP,
            origin: AirportId::Grandcity,
            destination: AirportId::Sunhaven,
            reward_gold: 1500,
            reward_xp: 120,
            time_limit_hours: Some(4),
            required_rank: PilotRank::Captain,
            required_aircraft_class: Some(AircraftClass::LightJet),
            passenger_count: 4,
            cargo_kg: 600.0,
            bonus_conditions: vec![BonusCondition::PerfectLanding, BonusCondition::NoTurbulenceDamage],
            difficulty: MissionDifficulty::Hard,
        },
        // Air Race
        MissionDef {
            id: "special_air_race".into(),
            title: "Skywarden Air Race".into(),
            description: "Timed flight through waypoints from HomeBase to Windport and back. Beat the ghost time for bonus gold.".into(),
            mission_type: MissionType::AirShow,
            origin: AirportId::HomeBase,
            destination: AirportId::HomeBase,
            reward_gold: 600,
            reward_xp: 100,
            time_limit_hours: Some(2),
            required_rank: PilotRank::Commercial,
            required_aircraft_class: Some(AircraftClass::SingleProp),
            passenger_count: 0,
            cargo_kg: 0.0,
            bonus_conditions: vec![BonusCondition::OnTime, BonusCondition::PerfectLanding],
            difficulty: MissionDifficulty::Medium,
        },
        // Aerial Photography
        MissionDef {
            id: "special_photo_duskhollow".into(),
            title: "Desert Aerial Photography".into(),
            description: "Fly specific routes at precise altitudes over Duskhollow dunes for a nature magazine. Steady flying is essential.".into(),
            mission_type: MissionType::Survey,
            origin: AirportId::Duskhollow,
            destination: AirportId::Duskhollow,
            reward_gold: 500,
            reward_xp: 80,
            time_limit_hours: None,
            required_rank: PilotRank::Private,
            required_aircraft_class: Some(AircraftClass::SingleProp),
            passenger_count: 1,
            cargo_kg: 30.0,
            bonus_conditions: vec![BonusCondition::NoTurbulenceDamage],
            difficulty: MissionDifficulty::Easy,
        },
        // Storm Chasing
        MissionDef {
            id: "special_storm_chase".into(),
            title: "Storm Front Research".into(),
            description: "Fly near a developing storm system off Stormwatch to collect atmospheric data. Get close, but NOT too close.".into(),
            mission_type: MissionType::Survey,
            origin: AirportId::Stormwatch,
            destination: AirportId::Stormwatch,
            reward_gold: 900,
            reward_xp: 180,
            time_limit_hours: None,
            required_rank: PilotRank::Senior,
            required_aircraft_class: Some(AircraftClass::TwinProp),
            passenger_count: 2,
            cargo_kg: 200.0,
            bonus_conditions: vec![BonusCondition::BadWeatherFlight],
            difficulty: MissionDifficulty::Expert,
        },
        // Wildlife Survey
        MissionDef {
            id: "special_wildlife_sunhaven".into(),
            title: "Coral Bay Wildlife Count".into(),
            description: "Low-altitude survey flight counting marine wildlife along the Sunhaven coast. Fly below 500ft for best observation.".into(),
            mission_type: MissionType::Survey,
            origin: AirportId::Sunhaven,
            destination: AirportId::Sunhaven,
            reward_gold: 400,
            reward_xp: 70,
            time_limit_hours: None,
            required_rank: PilotRank::Private,
            required_aircraft_class: Some(AircraftClass::SingleProp),
            passenger_count: 1,
            cargo_kg: 20.0,
            bonus_conditions: vec![BonusCondition::LowFuelUsage],
            difficulty: MissionDifficulty::Easy,
        },
        // Historical Aircraft Delivery
        MissionDef {
            id: "special_vintage_delivery".into(),
            title: "Vintage Biplane Ferry".into(),
            description: "Carefully ferry a restored 1940s biplane from Ironforge museum to Skyreach airshow. Handle with extreme care — no hard landings!".into(),
            mission_type: MissionType::Delivery,
            origin: AirportId::Ironforge,
            destination: AirportId::Skyreach,
            reward_gold: 700,
            reward_xp: 130,
            time_limit_hours: Some(8),
            required_rank: PilotRank::Senior,
            required_aircraft_class: Some(AircraftClass::SingleProp),
            passenger_count: 0,
            cargo_kg: 0.0,
            bonus_conditions: vec![BonusCondition::PerfectLanding, BonusCondition::NoTurbulenceDamage],
            difficulty: MissionDifficulty::Hard,
        },
        // Humanitarian Aid
        MissionDef {
            id: "special_humanitarian_duskhollow".into(),
            title: "Desert Relief Drop".into(),
            description: "Deliver emergency supplies to remote communities near Duskhollow after flooding. Multiple drop zones.".into(),
            mission_type: MissionType::Cargo,
            origin: AirportId::Grandcity,
            destination: AirportId::Duskhollow,
            reward_gold: 300,
            reward_xp: 160,
            time_limit_hours: Some(6),
            required_rank: PilotRank::Senior,
            required_aircraft_class: None,
            passenger_count: 0,
            cargo_kg: 2000.0,
            bonus_conditions: vec![BonusCondition::OnTime],
            difficulty: MissionDifficulty::Hard,
        },
        // Night Mail Run
        MissionDef {
            id: "special_night_mail".into(),
            title: "Night Mail Express".into(),
            description: "Overnight mail delivery circuit: HomeBase → Windport → Ironforge → HomeBase. Fly through the darkness with only instruments and stars.".into(),
            mission_type: MissionType::Delivery,
            origin: AirportId::HomeBase,
            destination: AirportId::HomeBase,
            reward_gold: 550,
            reward_xp: 110,
            time_limit_hours: Some(8),
            required_rank: PilotRank::Commercial,
            required_aircraft_class: None,
            passenger_count: 0,
            cargo_kg: 500.0,
            bonus_conditions: vec![BonusCondition::NightFlight, BonusCondition::OnTime],
            difficulty: MissionDifficulty::Medium,
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Initialize special mission state when a special mission is accepted.
pub fn init_special_mission(
    mut mission_accepted: EventReader<MissionAcceptedEvent>,
    _mission_board: Res<MissionBoard>,
    mut state: ResMut<SpecialMissionState>,
) {
    for ev in mission_accepted.read() {
        let mission_type = match ev.mission_id.as_str() {
            id if id.starts_with("special_sar") => Some(SpecialMissionType::SearchAndRescue),
            id if id.starts_with("special_medevac") => Some(SpecialMissionType::MedicalEvacuation),
            id if id.starts_with("special_celebrity") => {
                Some(SpecialMissionType::CelebrityTransport)
            }
            id if id.starts_with("special_air_race") => Some(SpecialMissionType::AirRace),
            id if id.starts_with("special_photo") => Some(SpecialMissionType::AerialPhotography),
            id if id.starts_with("special_storm") => Some(SpecialMissionType::StormChasing),
            id if id.starts_with("special_wildlife") => Some(SpecialMissionType::WildlifeSurvey),
            id if id.starts_with("special_vintage") => Some(SpecialMissionType::HistoricalDelivery),
            id if id.starts_with("special_humanitarian") => {
                Some(SpecialMissionType::HumanitarianAid)
            }
            id if id.starts_with("special_night_mail") => Some(SpecialMissionType::NightMailRun),
            _ => None,
        };

        if let Some(stype) = mission_type {
            *state = SpecialMissionState::default();
            state.active_type = Some(stype);

            match stype {
                SpecialMissionType::SearchAndRescue => {
                    state.search_pattern = Some(SearchPattern::ExpandingSquare);
                }
                SpecialMissionType::AirRace => {
                    state.race_waypoints = vec![
                        MissionWaypoint::new("START", 0.0, None),
                        MissionWaypoint::new("TURN1", 30.0, Some(3000.0)),
                        MissionWaypoint::new("MIDPOINT", 60.0, Some(5000.0)),
                        MissionWaypoint::new("TURN2", 90.0, Some(3000.0)),
                        MissionWaypoint::new("FINISH", 120.0, None),
                    ];
                }
                SpecialMissionType::AerialPhotography => {
                    state.photo_waypoints = vec![
                        MissionWaypoint::new("DUNE_NORTH", 10.0, Some(500.0)),
                        MissionWaypoint::new("CANYON_PASS", 25.0, Some(800.0)),
                        MissionWaypoint::new("OASIS", 40.0, Some(500.0)),
                        MissionWaypoint::new("MESA_TOP", 55.0, Some(1000.0)),
                    ];
                }
                SpecialMissionType::CelebrityTransport => {
                    state.celebrity_demands_total = 3;
                }
                SpecialMissionType::HumanitarianAid => {
                    state.aid_packages_remaining = 5;
                }
                SpecialMissionType::NightMailRun => {
                    state.mail_stops_total = 3;
                }
                _ => {}
            }
        }
    }
}

/// Update special mission progress during flight.
pub fn update_special_mission(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut state: ResMut<SpecialMissionState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let Some(stype) = state.active_type else {
        return;
    };

    if flight_state.phase == FlightPhase::Idle {
        return;
    }

    let dt = time.delta_secs();
    state.completion_timer += dt;

    match stype {
        SpecialMissionType::SearchAndRescue => {
            // Progress based on low altitude flying in search area
            if flight_state.altitude_ft < 2000.0
                && matches!(
                    flight_state.phase,
                    FlightPhase::Cruise | FlightPhase::Descent
                )
            {
                let pattern_bonus = state
                    .search_pattern
                    .as_ref()
                    .map(|p| p.detection_bonus())
                    .unwrap_or(1.0);
                state.search_progress += 0.01 * pattern_bonus * dt;

                if state.search_progress >= 0.5 && state.survivors_found == 0 {
                    state.survivors_found = 3;
                    toast_events.send(ToastEvent {
                        message: "🔍 Survivors spotted! Radioing coordinates to ground teams."
                            .to_string(),
                        duration_secs: 5.0,
                    });
                }
            }
        }
        SpecialMissionType::MedicalEvacuation => {
            state.time_bonus_secs += dt;
        }
        SpecialMissionType::StormChasing => {
            let near_storm = weather.current == Weather::Storm || weather.current == Weather::Rain;
            if near_storm
                && matches!(
                    flight_state.phase,
                    FlightPhase::Cruise | FlightPhase::Climb | FlightPhase::Descent
                )
            {
                state.storm_data_collected = (state.storm_data_collected + 0.008 * dt).min(1.0);
                if state.storm_data_collected >= 1.0 {
                    toast_events.send(ToastEvent {
                        message: "📊 Storm data collection complete! Head back to base."
                            .to_string(),
                        duration_secs: 4.0,
                    });
                }
            }
        }
        SpecialMissionType::WildlifeSurvey => {
            if flight_state.altitude_ft < 500.0 && flight_state.phase == FlightPhase::Cruise {
                let t = time.elapsed_secs();
                if (t * 0.5).sin() > 0.9 {
                    state.wildlife_counted += 1;
                    toast_events.send(ToastEvent {
                        message: format!(
                            "🦅 Wildlife spotted! Total count: {}",
                            state.wildlife_counted
                        ),
                        duration_secs: 2.0,
                    });
                }
            }
        }
        SpecialMissionType::HistoricalDelivery => {
            // Fragility increases with turbulence
            let turb_damage = match weather.turbulence_level {
                TurbulenceLevel::None => 0.0,
                TurbulenceLevel::Light => 0.001,
                TurbulenceLevel::Moderate => 0.01,
                TurbulenceLevel::Severe => 0.05,
            };
            state.cargo_fragility = (state.cargo_fragility + turb_damage * dt).min(1.0);
            if state.cargo_fragility > 0.5 {
                toast_events.send(ToastEvent {
                    message: "⚠ The vintage aircraft is taking stress damage! Fly smoother!"
                        .to_string(),
                    duration_secs: 3.0,
                });
            }
        }
        SpecialMissionType::AirRace => {
            update_race_waypoints(&flight_state, &mut state, &mut toast_events);
        }
        SpecialMissionType::AerialPhotography => {
            update_photo_waypoints(&flight_state, &mut state, &mut toast_events);
        }
        _ => {}
    }
}

fn update_race_waypoints(
    flight_state: &FlightState,
    state: &mut SpecialMissionState,
    toast_events: &mut EventWriter<ToastEvent>,
) {
    let distance_flown = flight_state.distance_total_nm - flight_state.distance_remaining_nm;
    for wp in &mut state.race_waypoints {
        if wp.passed {
            continue;
        }
        if distance_flown >= wp.distance_from_start_nm {
            wp.passed = true;
            toast_events.send(ToastEvent {
                message: format!("🏁 Waypoint {} passed!", wp.name),
                duration_secs: 2.0,
            });
        }
    }
}

fn update_photo_waypoints(
    flight_state: &FlightState,
    state: &mut SpecialMissionState,
    toast_events: &mut EventWriter<ToastEvent>,
) {
    let distance_flown = flight_state.distance_total_nm - flight_state.distance_remaining_nm;
    for wp in &mut state.photo_waypoints {
        if wp.passed {
            continue;
        }
        if distance_flown >= wp.distance_from_start_nm {
            let alt_ok = wp.required_altitude_ft.is_none_or(|req| {
                (flight_state.altitude_ft - req).abs() <= wp.altitude_tolerance_ft
            });
            if alt_ok {
                wp.passed = true;
                toast_events.send(ToastEvent {
                    message: format!("📸 Photo taken at {}!", wp.name),
                    duration_secs: 2.0,
                });
            } else {
                toast_events.send(ToastEvent {
                    message: format!(
                        "⚠ Wrong altitude for {}! Need {:.0}ft ± {:.0}ft",
                        wp.name,
                        wp.required_altitude_ft.unwrap_or(0.0),
                        wp.altitude_tolerance_ft
                    ),
                    duration_secs: 3.0,
                });
            }
        }
    }
}

/// Calculate special mission completion bonus.
pub fn complete_special_mission(
    mut flight_complete: EventReader<FlightCompleteEvent>,
    mut state: ResMut<SpecialMissionState>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut xp_events: EventWriter<XpGainEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in flight_complete.read() {
        let Some(stype) = state.active_type else {
            continue;
        };

        let mut bonus_gold: i32 = 0;
        let mut bonus_xp: u32 = 0;
        let mut summary = String::new();

        match stype {
            SpecialMissionType::SearchAndRescue => {
                if state.survivors_found > 0 {
                    bonus_gold += state.survivors_found as i32 * 100;
                    bonus_xp += state.survivors_found * 30;
                    summary = format!("{} survivors rescued!", state.survivors_found);
                }
            }
            SpecialMissionType::MedicalEvacuation => {
                // Speed bonus: less time = more bonus
                let time_min = state.time_bonus_secs / 60.0;
                if time_min < 60.0 {
                    bonus_gold += 500;
                    bonus_xp += 100;
                    summary = format!("Patient delivered in {:.0} min — speed bonus!", time_min);
                } else {
                    summary = format!("Patient delivered in {:.0} min.", time_min);
                }
            }
            SpecialMissionType::CelebrityTransport => {
                let met = state.celebrity_demands_met;
                let total = state.celebrity_demands_total;
                bonus_gold += met as i32 * 200;
                summary = format!("{}/{} VIP demands satisfied.", met, total);
            }
            SpecialMissionType::AirRace => {
                let passed = state.race_waypoints.iter().filter(|w| w.passed).count();
                let total = state.race_waypoints.len();
                bonus_gold += passed as i32 * 50;
                bonus_xp += passed as u32 * 15;
                summary = format!("{}/{} waypoints cleared.", passed, total);
            }
            SpecialMissionType::AerialPhotography => {
                let taken = state.photo_waypoints.iter().filter(|w| w.passed).count();
                let total = state.photo_waypoints.len();
                bonus_gold += taken as i32 * 75;
                summary = format!("{}/{} photos captured.", taken, total);
            }
            SpecialMissionType::StormChasing => {
                let pct = (state.storm_data_collected * 100.0) as u32;
                bonus_gold += pct as i32 * 5;
                bonus_xp += pct;
                summary = format!("{}% storm data collected.", pct);
            }
            SpecialMissionType::WildlifeSurvey => {
                bonus_gold += state.wildlife_counted as i32 * 10;
                summary = format!("{} animals counted.", state.wildlife_counted);
            }
            SpecialMissionType::HistoricalDelivery => {
                let condition = ((1.0 - state.cargo_fragility) * 100.0) as u32;
                bonus_gold += condition as i32 * 5;
                summary = format!("Aircraft delivered at {}% condition.", condition);
            }
            SpecialMissionType::HumanitarianAid => {
                let delivered = 5 - state.aid_packages_remaining;
                bonus_xp += delivered * 30;
                summary = format!("{}/5 aid packages delivered.", delivered);
            }
            SpecialMissionType::NightMailRun => {
                let stops = state.mail_stops_completed;
                let total = state.mail_stops_total;
                bonus_gold += stops as i32 * 100;
                summary = format!("{}/{} mail stops completed.", stops, total);
            }
        }

        if bonus_gold > 0 {
            gold_events.send(GoldChangeEvent {
                amount: bonus_gold,
                reason: format!("Special mission bonus: {}", stype.display_name()),
            });
        }
        if bonus_xp > 0 {
            xp_events.send(XpGainEvent {
                amount: bonus_xp,
                source: format!("Special mission: {}", stype.display_name()),
            });
        }

        toast_events.send(ToastEvent {
            message: format!(
                "✨ {} complete! {} +{}g +{}xp",
                stype.display_name(),
                summary,
                bonus_gold,
                bonus_xp
            ),
            duration_secs: 5.0,
        });

        *state = SpecialMissionState::default();
    }
}
