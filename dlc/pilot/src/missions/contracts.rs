//! Long-term contract system — recurring routes, loyalty bonuses, charter contracts.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContractFrequency {
    Daily,
    Weekly,
}

impl ContractFrequency {
    pub fn display_name(&self) -> &'static str {
        match self {
            ContractFrequency::Daily => "Daily",
            ContractFrequency::Weekly => "Weekly",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Contract {
    pub id: String,
    pub airline: String,
    pub origin: AirportId,
    pub destination: AirportId,
    pub frequency: ContractFrequency,
    pub duration_days: u32,
    pub base_pay: u32,
    pub bonus_conditions: Vec<BonusCondition>,
    pub bonus_pay: u32,
    pub required_rank: PilotRank,
    pub required_aircraft_class: Option<AircraftClass>,
    pub is_charter: bool,
}

impl Contract {
    pub fn display_summary(&self) -> String {
        format!(
            "{} — {} → {} ({}, {}d, {}g/flight)",
            self.airline,
            self.origin.icao_code(),
            self.destination.icao_code(),
            self.frequency.display_name(),
            self.duration_days,
            self.base_pay,
        )
    }

    pub fn total_potential_pay(&self) -> u32 {
        let flights = match self.frequency {
            ContractFrequency::Daily => self.duration_days,
            ContractFrequency::Weekly => self.duration_days / 7,
        };
        flights * (self.base_pay + self.bonus_pay)
    }
}

#[derive(Clone, Debug)]
pub struct ActiveContract {
    pub contract: Contract,
    pub start_day: u32,
    pub flights_completed: u32,
    pub flights_required: u32,
    pub on_time_count: u32,
    pub perfect_landing_count: u32,
    pub total_earned: u32,
}

impl ActiveContract {
    pub fn is_expired(&self, current_day: u32) -> bool {
        current_day > self.start_day + self.contract.duration_days
    }

    pub fn completion_rate(&self) -> f32 {
        if self.flights_required == 0 {
            return 1.0;
        }
        self.flights_completed as f32 / self.flights_required as f32
    }

    pub fn loyalty_bonus(&self) -> u32 {
        if self.completion_rate() >= 0.9 {
            (self.total_earned as f32 * 0.15) as u32
        } else {
            0
        }
    }
}

/// Contract board resource.
#[derive(Resource, Clone, Debug, Default)]
pub struct ContractBoard {
    pub available: Vec<Contract>,
    pub active: Vec<ActiveContract>,
    pub completed_ids: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Refresh the contract board with new offerings at day end.
pub fn refresh_contract_board(
    mut day_end_events: EventReader<DayEndEvent>,
    mut board: ResMut<ContractBoard>,
    pilot_state: Res<PilotState>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let mut rng = rand::thread_rng();
        let day = calendar.total_days();

        // Only refresh every 3 days
        if day % 3 != 0 {
            return;
        }

        board.available.clear();
        let num_contracts = rng.gen_range(2..=5);

        let airports = [
            AirportId::HomeBase,
            AirportId::Windport,
            AirportId::Frostpeak,
            AirportId::Sunhaven,
            AirportId::Ironforge,
            AirportId::Grandcity,
        ];
        let airlines = [
            "SkyWing Air", "Horizon Lines", "CloudHop Express",
            "Apex Aviation", "Summit Air",
        ];

        for i in 0..num_contracts {
            let origin = airports[rng.gen_range(0..airports.len())];
            let mut dest = airports[rng.gen_range(0..airports.len())];
            while dest == origin {
                dest = airports[rng.gen_range(0..airports.len())];
            }

            let is_charter = rng.gen_bool(0.2);
            let freq = if is_charter {
                ContractFrequency::Daily
            } else if rng.gen_bool(0.5) {
                ContractFrequency::Daily
            } else {
                ContractFrequency::Weekly
            };

            let duration = if is_charter {
                1
            } else {
                rng.gen_range(7..=28)
            };

            let base_pay = if is_charter {
                rng.gen_range(800..=2000)
            } else {
                rng.gen_range(200..=600)
            };

            board.available.push(Contract {
                id: format!("contract_{}_{}", day, i),
                airline: airlines[rng.gen_range(0..airlines.len())].to_string(),
                origin,
                destination: dest,
                frequency: freq,
                duration_days: duration,
                base_pay,
                bonus_conditions: vec![BonusCondition::OnTime, BonusCondition::PerfectLanding],
                bonus_pay: base_pay / 4,
                required_rank: pilot_state.rank,
                required_aircraft_class: None,
                is_charter,
            });
        }
    }
}

/// Evaluate active contracts — check expiry, pay bonuses.
pub fn evaluate_contracts(
    mut day_end_events: EventReader<DayEndEvent>,
    mut board: ResMut<ContractBoard>,
    calendar: Res<Calendar>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        let day = calendar.total_days();
        let mut completed = Vec::new();

        for (i, active) in board.active.iter().enumerate() {
            if active.is_expired(day) {
                let bonus = active.loyalty_bonus();
                if bonus > 0 {
                    gold_events.send(GoldChangeEvent {
                        amount: bonus as i32,
                        reason: format!("Loyalty bonus: {}", active.contract.airline),
                    });
                    toast_events.send(ToastEvent {
                        message: format!(
                            "Contract complete! Loyalty bonus: {}g",
                            bonus
                        ),
                        duration_secs: 4.0,
                    });
                }
                completed.push(i);
            }
        }

        // Remove completed contracts in reverse order
        for i in completed.into_iter().rev() {
            let c = board.active.remove(i);
            board.completed_ids.push(c.contract.id);
        }
    }
}

/// Track contract flights on flight completion.
pub fn track_contract_flights(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut board: ResMut<ContractBoard>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    for ev in flight_complete_events.read() {
        for active in &mut board.active {
            if active.contract.origin == ev.origin && active.contract.destination == ev.destination {
                active.flights_completed += 1;
                active.total_earned += active.contract.base_pay;

                gold_events.send(GoldChangeEvent {
                    amount: active.contract.base_pay as i32,
                    reason: format!("Contract flight: {}", active.contract.airline),
                });

                if ev.landing_grade == "Perfect" {
                    active.perfect_landing_count += 1;
                }
                active.on_time_count += 1;
            }
        }
    }
}
