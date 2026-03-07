//! Gold management — transaction log, daily expenses, milestones.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A recorded financial transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub day: u32,
    pub amount: i32,
    pub reason: String,
}

/// Persistent transaction log.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct TransactionLog {
    pub entries: Vec<Transaction>,
}

impl TransactionLog {
    pub fn record(&mut self, day: u32, amount: i32, reason: &str) {
        self.entries.push(Transaction {
            day,
            amount,
            reason: reason.to_string(),
        });
    }

    pub fn net_for_day(&self, day: u32) -> i32 {
        self.entries
            .iter()
            .filter(|t| t.day == day)
            .map(|t| t.amount)
            .sum()
    }

    pub fn earnings_total(&self) -> u32 {
        self.entries
            .iter()
            .filter(|t| t.amount > 0)
            .map(|t| t.amount as u32)
            .sum()
    }

    pub fn spending_total(&self) -> u32 {
        self.entries
            .iter()
            .filter(|t| t.amount < 0)
            .map(|t| t.amount.unsigned_abs())
            .sum()
    }
}

const GOLD_MILESTONES: &[u32] = &[1_000, 5_000, 10_000, 50_000, 100_000, 500_000, 1_000_000];

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct GoldMilestones {
    pub reached: Vec<u32>,
}

/// Helper: attempt to spend gold.
pub fn spend_gold(
    gold: &mut Gold,
    amount: u32,
    reason: &str,
    log: &mut TransactionLog,
    day: u32,
) -> bool {
    if gold.amount < amount {
        return false;
    }
    gold.amount -= amount;
    log.record(day, -(amount as i32), reason);
    true
}

/// Helper: add gold earnings.
pub fn add_gold(gold: &mut Gold, amount: u32, reason: &str, log: &mut TransactionLog, day: u32) {
    gold.amount += amount;
    log.record(day, amount as i32, reason);
}

/// Net worth including fleet value.
pub fn net_worth(gold: &Gold, fleet: &Fleet, registry: &AircraftRegistry) -> u32 {
    let fleet_value: u32 = fleet
        .aircraft
        .iter()
        .map(|ac| {
            let base_price = registry
                .get(&ac.aircraft_id)
                .map_or(0, |d| d.purchase_price);
            let depreciation = (base_price as f32 * 0.02 * ac.total_flights as f32) as u32;
            base_price.saturating_sub(depreciation).max(base_price / 5)
        })
        .sum();
    gold.amount + fleet_value
}

/// Main system: process gold change events with logging and milestone checks.
pub fn apply_gold_changes(
    mut events: EventReader<GoldChangeEvent>,
    mut gold: ResMut<Gold>,
    mut economy_stats: ResMut<EconomyStats>,
    mut toast_events: EventWriter<ToastEvent>,
    mut log: ResMut<TransactionLog>,
    mut milestones: ResMut<GoldMilestones>,
    calendar: Res<Calendar>,
) {
    for ev in events.read() {
        let day = calendar.total_days();
        if ev.amount > 0 {
            let gain = ev.amount as u32;
            gold.amount += gain;
            economy_stats.total_earned += gain;
            log.record(day, ev.amount, &ev.reason);

            for &milestone in GOLD_MILESTONES {
                if economy_stats.total_earned >= milestone
                    && !milestones.reached.contains(&milestone)
                {
                    milestones.reached.push(milestone);
                    toast_events.send(ToastEvent {
                        message: format!("💰 Lifetime earnings: {}g!", format_gold(milestone)),
                        duration_secs: 4.0,
                    });
                }
            }
        } else {
            let cost = (-ev.amount) as u32;
            if gold.amount >= cost {
                gold.amount -= cost;
                economy_stats.total_spent += cost;
                log.record(day, ev.amount, &ev.reason);
            } else {
                toast_events.send(ToastEvent {
                    message: format!("Not enough gold! Need {}g, have {}g", cost, gold.amount),
                    duration_secs: 2.5,
                });
            }
        }
    }
}

/// Daily expenses at day end.
pub fn apply_daily_expenses(
    mut day_end_events: EventReader<DayEndEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    fleet: Res<Fleet>,
) {
    for _ev in day_end_events.read() {
        let hangar_cost = fleet.aircraft.len() as i32 * 10;
        if hangar_cost > 0 {
            gold_events.send(GoldChangeEvent {
                amount: -hangar_cost,
                reason: "Daily hangar rental".to_string(),
            });
        }
        let insurance = fleet.aircraft.len() as i32 * 5;
        if insurance > 0 {
            gold_events.send(GoldChangeEvent {
                amount: -insurance,
                reason: "Aircraft insurance".to_string(),
            });
        }
    }
}

fn format_gold(amount: u32) -> String {
    if amount >= 1_000_000 {
        format!("{:.1}M", amount as f32 / 1_000_000.0)
    } else if amount >= 1_000 {
        format!("{:.1}K", amount as f32 / 1_000.0)
    } else {
        format!("{}", amount)
    }
}
