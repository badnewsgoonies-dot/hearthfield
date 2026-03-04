//! Financial tracking — contract payments, bonuses, fuel costs, maintenance, income statements.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Default)]
pub struct IncomeEntry {
    pub day: u32,
    pub amount: i32,
    pub category: String,
}

/// Tracks detailed financial history.
#[derive(Resource, Clone, Debug, Default)]
pub struct FinancialLedger {
    pub entries: Vec<IncomeEntry>,
    pub weekly_salary_day: u32, // last day salary was paid
}

impl FinancialLedger {
    pub fn record(&mut self, day: u32, amount: i32, category: &str) {
        self.entries.push(IncomeEntry {
            day,
            amount,
            category: category.to_string(),
        });
    }

    pub fn daily_summary(&self, day: u32) -> i32 {
        self.entries
            .iter()
            .filter(|e| e.day == day)
            .map(|e| e.amount)
            .sum()
    }

    pub fn weekly_summary(&self, current_day: u32) -> i32 {
        self.entries
            .iter()
            .filter(|e| current_day.saturating_sub(e.day) < 7)
            .map(|e| e.amount)
            .sum()
    }

    pub fn monthly_summary(&self, current_day: u32) -> i32 {
        self.entries
            .iter()
            .filter(|e| current_day.saturating_sub(e.day) < 28)
            .map(|e| e.amount)
            .sum()
    }

    pub fn income_this_period(&self, current_day: u32, period_days: u32) -> i32 {
        self.entries
            .iter()
            .filter(|e| current_day.saturating_sub(e.day) < period_days && e.amount > 0)
            .map(|e| e.amount)
            .sum()
    }

    pub fn expenses_this_period(&self, current_day: u32, period_days: u32) -> i32 {
        self.entries
            .iter()
            .filter(|e| current_day.saturating_sub(e.day) < period_days && e.amount < 0)
            .map(|e| e.amount.abs())
            .sum()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Calculate and pay bonuses after each flight.
pub fn calculate_bonuses(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut ledger: ResMut<FinancialLedger>,
    calendar: Res<Calendar>,
) {
    for ev in flight_complete_events.read() {
        let day = calendar.total_days();

        // On-time bonus (all flights currently count)
        let on_time_bonus: i32 = 25;
        gold_events.send(GoldChangeEvent {
            amount: on_time_bonus,
            reason: "On-time arrival bonus".to_string(),
        });
        ledger.record(day, on_time_bonus, "on_time_bonus");

        // Landing quality bonus
        let landing_bonus: i32 = match ev.landing_grade.as_str() {
            "Perfect" => 75,
            "Good" => 40,
            _ => 0,
        };
        if landing_bonus > 0 {
            gold_events.send(GoldChangeEvent {
                amount: landing_bonus,
                reason: "Smooth landing bonus".to_string(),
            });
            ledger.record(day, landing_bonus, "landing_bonus");
        }
    }
}

/// Deduct fuel costs after each flight based on distance and aircraft.
pub fn process_fuel_costs(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut ledger: ResMut<FinancialLedger>,
    calendar: Res<Calendar>,
) {
    for ev in flight_complete_events.read() {
        let day = calendar.total_days();
        // Fuel cost: roughly 2g per unit of fuel used
        let fuel_cost = (ev.fuel_used * 2.0) as i32;
        if fuel_cost > 0 {
            gold_events.send(GoldChangeEvent {
                amount: -fuel_cost,
                reason: "Fuel costs".to_string(),
            });
            ledger.record(day, -fuel_cost, "fuel_cost");
        }
    }
}

/// Scheduled maintenance deductions at day end.
pub fn process_maintenance_costs(
    mut day_end_events: EventReader<DayEndEvent>,
    fleet: Res<Fleet>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut ledger: ResMut<FinancialLedger>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let day = calendar.total_days();
        // Maintenance cost per owned aircraft per week
        if !day.is_multiple_of(7) {
            return;
        }
        let aircraft_count = fleet.aircraft.len() as i32;
        let maintenance = aircraft_count * 50;
        if maintenance > 0 {
            gold_events.send(GoldChangeEvent {
                amount: -maintenance,
                reason: "Weekly fleet maintenance".to_string(),
            });
            ledger.record(day, -maintenance, "maintenance");
        }
    }
}
