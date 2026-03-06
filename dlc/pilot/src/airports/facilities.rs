//! Airport facility interactions — refuel, repair, weather briefing, training.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct FacilityPlugin;

impl Plugin for FacilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FacilityState>().add_systems(
            Update,
            (interact_facility, process_facility_action).run_if(in_state(GameState::Playing)),
        );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacilityType {
    FuelStation,
    MaintenanceHangar,
    ControlTower,
    PassengerTerminal,
    CargoWarehouse,
    WeatherStation,
    FlightSchool,
    Cafe,
}

impl FacilityType {
    pub fn display_name(&self) -> &'static str {
        match self {
            FacilityType::FuelStation => "Fuel Station",
            FacilityType::MaintenanceHangar => "Maintenance Hangar",
            FacilityType::ControlTower => "Control Tower",
            FacilityType::PassengerTerminal => "Passenger Terminal",
            FacilityType::CargoWarehouse => "Cargo Warehouse",
            FacilityType::WeatherStation => "Weather Station",
            FacilityType::FlightSchool => "Flight School",
            FacilityType::Cafe => "Cafe",
        }
    }

    pub fn zone(&self) -> MapZone {
        match self {
            FacilityType::FuelStation => MapZone::Hangar,
            FacilityType::MaintenanceHangar => MapZone::Hangar,
            FacilityType::ControlTower => MapZone::ControlTower,
            FacilityType::PassengerTerminal => MapZone::Terminal,
            FacilityType::CargoWarehouse => MapZone::Hangar,
            FacilityType::WeatherStation => MapZone::ControlTower,
            FacilityType::FlightSchool => MapZone::Terminal,
            FacilityType::Cafe => MapZone::Lounge,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            FacilityType::FuelStation => "[F] Refuel",
            FacilityType::MaintenanceHangar => "[F] Repair Aircraft",
            FacilityType::ControlTower => "[F] File Flight Plan",
            FacilityType::PassengerTerminal => "[F] Check Passengers",
            FacilityType::CargoWarehouse => "[F] Load Cargo",
            FacilityType::WeatherStation => "[F] Weather Briefing",
            FacilityType::FlightSchool => "[F] Training Course",
            FacilityType::Cafe => "[F] Buy Food",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacilityQuality {
    Basic,
    Standard,
    Premium,
}

impl FacilityQuality {
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            FacilityQuality::Basic => 1.0,
            FacilityQuality::Standard => 0.75,
            FacilityQuality::Premium => 0.5,
        }
    }

    pub fn cost_multiplier(&self) -> f32 {
        match self {
            FacilityQuality::Basic => 1.0,
            FacilityQuality::Standard => 1.25,
            FacilityQuality::Premium => 1.5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AirportFacility {
    pub facility_type: FacilityType,
    pub quality: FacilityQuality,
    pub available: bool,
}

// ── Events ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FacilityAction {
    Refuel { amount: f32, cost: u32 },
    Repair { cost: u32 },
    WeatherBrief,
    TrainingCourse { xp: u32, cost: u32 },
    BuyFood { item_id: String, cost: u32 },
    FilePlan,
    CheckPassengers,
    LoadCargo,
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FacilityState {
    pub facilities: Vec<AirportFacility>,
    pub pending_action: Option<FacilityAction>,
}

impl FacilityState {
    pub fn for_airport(airport: AirportId) -> Vec<AirportFacility> {
        let quality = match airport {
            AirportId::HomeBase => FacilityQuality::Basic,
            AirportId::Windport | AirportId::Frostpeak => FacilityQuality::Standard,
            AirportId::Grandcity | AirportId::Skyreach => FacilityQuality::Premium,
            _ => FacilityQuality::Standard,
        };

        let mut facilities = vec![
            AirportFacility {
                facility_type: FacilityType::FuelStation,
                quality,
                available: true,
            },
            AirportFacility {
                facility_type: FacilityType::MaintenanceHangar,
                quality,
                available: true,
            },
            AirportFacility {
                facility_type: FacilityType::ControlTower,
                quality,
                available: true,
            },
            AirportFacility {
                facility_type: FacilityType::PassengerTerminal,
                quality,
                available: true,
            },
            AirportFacility {
                facility_type: FacilityType::WeatherStation,
                quality,
                available: true,
            },
            AirportFacility {
                facility_type: FacilityType::Cafe,
                quality,
                available: true,
            },
        ];

        // Bigger airports get extra facilities
        match airport {
            AirportId::Grandcity | AirportId::Skyreach | AirportId::Ironforge => {
                facilities.push(AirportFacility {
                    facility_type: FacilityType::CargoWarehouse,
                    quality,
                    available: true,
                });
            }
            _ => {}
        }

        if matches!(airport, AirportId::HomeBase | AirportId::Grandcity) {
            facilities.push(AirportFacility {
                facility_type: FacilityType::FlightSchool,
                quality,
                available: true,
            });
        }

        facilities
    }
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn interact_facility(
    input: Res<PlayerInput>,
    player_location: Res<PlayerLocation>,
    mut facility_state: ResMut<FacilityState>,
    fleet: Res<Fleet>,
    weather: Res<WeatherState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !input.interact {
        return;
    }

    // Find a matching facility for the current zone
    let facilities = FacilityState::for_airport(player_location.airport);
    let matching = facilities
        .iter()
        .find(|f| f.facility_type.zone() == player_location.zone && f.available);

    let Some(facility) = matching else {
        return;
    };

    let action = match facility.facility_type {
        FacilityType::FuelStation => {
            let cost = (50.0 * facility.quality.cost_multiplier()) as u32;
            FacilityAction::Refuel { amount: 50.0, cost }
        }
        FacilityType::MaintenanceHangar => {
            let repair_cost = if let Some(ac) = fleet.active() {
                ((100.0 - ac.condition) * 5.0 * facility.quality.cost_multiplier()) as u32
            } else {
                0
            };
            FacilityAction::Repair { cost: repair_cost }
        }
        FacilityType::WeatherStation => {
            toast_events.send(ToastEvent {
                message: format!(
                    "Weather: {:?} | Wind: {:.0}° at {:.0}kts | Vis: {:.1}nm | Ceiling: {}ft",
                    weather.current,
                    weather.wind_direction_deg,
                    weather.wind_speed_knots,
                    weather.visibility_nm,
                    weather.ceiling_ft
                ),
                duration_secs: 6.0,
            });
            FacilityAction::WeatherBrief
        }
        FacilityType::FlightSchool => {
            let cost = (200.0 * facility.quality.cost_multiplier()) as u32;
            FacilityAction::TrainingCourse { xp: 25, cost }
        }
        FacilityType::Cafe => {
            let cost = (15.0 * facility.quality.cost_multiplier()) as u32;
            FacilityAction::BuyFood {
                item_id: "coffee".to_string(),
                cost,
            }
        }
        FacilityType::ControlTower => FacilityAction::FilePlan,
        FacilityType::PassengerTerminal => FacilityAction::CheckPassengers,
        FacilityType::CargoWarehouse => FacilityAction::LoadCargo,
    };

    facility_state.pending_action = Some(action);
}

#[allow(clippy::too_many_arguments)]
pub fn process_facility_action(
    mut facility_state: ResMut<FacilityState>,
    mut gold: ResMut<Gold>,
    mut fleet: ResMut<Fleet>,
    mut pilot_state: ResMut<PilotState>,
    mut inventory: ResMut<Inventory>,
    mut toast_events: EventWriter<ToastEvent>,
    mut xp_events: EventWriter<XpGainEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    let Some(action) = facility_state.pending_action.take() else {
        return;
    };

    match action {
        FacilityAction::Refuel { amount, cost } => {
            if gold.amount < cost {
                toast_events.send(ToastEvent {
                    message: "Not enough gold to refuel.".to_string(),
                    duration_secs: 2.0,
                });
                return;
            }
            gold.amount -= cost;
            if let Some(ac) = fleet.active_mut() {
                ac.fuel = (ac.fuel + amount).min(100.0);
            }
            gold_events.send(GoldChangeEvent {
                amount: -(cost as i32),
                reason: "Refuel".to_string(),
            });
            toast_events.send(ToastEvent {
                message: format!("Refueled +{amount:.0} (-{cost}g)"),
                duration_secs: 3.0,
            });
        }
        FacilityAction::Repair { cost } => {
            if cost == 0 {
                toast_events.send(ToastEvent {
                    message: "Aircraft already in perfect condition!".to_string(),
                    duration_secs: 2.0,
                });
                return;
            }
            if gold.amount < cost {
                toast_events.send(ToastEvent {
                    message: "Not enough gold for repairs.".to_string(),
                    duration_secs: 2.0,
                });
                return;
            }
            gold.amount -= cost;
            if let Some(ac) = fleet.active_mut() {
                ac.condition = 100.0;
            }
            gold_events.send(GoldChangeEvent {
                amount: -(cost as i32),
                reason: "Repair".to_string(),
            });
            toast_events.send(ToastEvent {
                message: format!("Aircraft repaired to 100% (-{cost}g)"),
                duration_secs: 3.0,
            });
        }
        FacilityAction::TrainingCourse { xp, cost } => {
            if gold.amount < cost {
                toast_events.send(ToastEvent {
                    message: "Not enough gold for training.".to_string(),
                    duration_secs: 2.0,
                });
                return;
            }
            gold.amount -= cost;
            pilot_state.xp += xp;
            xp_events.send(XpGainEvent {
                amount: xp,
                source: "Flight School".to_string(),
            });
            gold_events.send(GoldChangeEvent {
                amount: -(cost as i32),
                reason: "Training".to_string(),
            });
            toast_events.send(ToastEvent {
                message: format!("Training complete! +{xp} XP (-{cost}g)"),
                duration_secs: 3.0,
            });
        }
        FacilityAction::BuyFood { item_id, cost } => {
            if gold.amount < cost {
                toast_events.send(ToastEvent {
                    message: "Not enough gold.".to_string(),
                    duration_secs: 2.0,
                });
                return;
            }
            gold.amount -= cost;
            inventory.add_item(&item_id, 1);
            gold_events.send(GoldChangeEvent {
                amount: -(cost as i32),
                reason: "Cafe".to_string(),
            });
            toast_events.send(ToastEvent {
                message: format!("Bought {item_id} (-{cost}g)"),
                duration_secs: 2.0,
            });
        }
        FacilityAction::WeatherBrief => {}
        FacilityAction::FilePlan => {
            toast_events.send(ToastEvent {
                message: "Flight plan filed with tower.".to_string(),
                duration_secs: 2.0,
            });
        }
        FacilityAction::CheckPassengers => {
            toast_events.send(ToastEvent {
                message: "Passengers checked in and boarded.".to_string(),
                duration_secs: 2.0,
            });
        }
        FacilityAction::LoadCargo => {
            toast_events.send(ToastEvent {
                message: "Cargo loaded and secured.".to_string(),
                duration_secs: 2.0,
            });
        }
    }
}
