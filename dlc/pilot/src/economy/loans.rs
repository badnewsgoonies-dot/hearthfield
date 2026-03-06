//! Loan system — borrow to purchase aircraft, monthly payments, interest.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Loan {
    pub id: String,
    pub aircraft_id: String,
    pub principal: u32,
    pub interest_rate: f32, // annual rate (e.g. 0.08 = 8%)
    pub monthly_payment: u32,
    pub remaining_balance: u32,
    pub term_months: u32,
    pub months_paid: u32,
    pub missed_payments: u32,
    pub active: bool,
}

impl Loan {
    pub fn new(aircraft_id: &str, principal: u32, interest_rate: f32, term_months: u32) -> Self {
        let monthly = Self::calculate_monthly_payment(principal, interest_rate, term_months);
        Self {
            id: format!("loan_{}_{}", aircraft_id, principal),
            aircraft_id: aircraft_id.to_string(),
            principal,
            interest_rate,
            monthly_payment: monthly,
            remaining_balance: principal,
            term_months,
            months_paid: 0,
            missed_payments: 0,
            active: true,
        }
    }

    fn calculate_monthly_payment(principal: u32, annual_rate: f32, term_months: u32) -> u32 {
        if annual_rate == 0.0 || term_months == 0 {
            return if term_months > 0 {
                principal / term_months
            } else {
                principal
            };
        }
        let r = annual_rate / 12.0;
        let n = term_months as f32;
        let factor = r * (1.0 + r).powf(n) / ((1.0 + r).powf(n) - 1.0);
        (principal as f32 * factor).ceil() as u32
    }

    pub fn total_interest(&self) -> u32 {
        (self.monthly_payment * self.term_months).saturating_sub(self.principal)
    }

    pub fn early_payoff_amount(&self) -> u32 {
        // 5% discount for early payoff
        (self.remaining_balance as f32 * 0.95) as u32
    }

    pub fn is_defaulted(&self) -> bool {
        self.missed_payments >= 3
    }
}

/// Interest rate based on pilot rank (better rank = lower rate).
pub fn interest_rate_for_rank(rank: PilotRank) -> f32 {
    match rank {
        PilotRank::Student => 0.12,
        PilotRank::Private => 0.10,
        PilotRank::Commercial => 0.08,
        PilotRank::Senior => 0.06,
        PilotRank::Captain => 0.05,
        PilotRank::Ace => 0.04,
    }
}

/// Maximum loan-to-value ratio.
pub const MAX_LTV: f32 = 0.80; // 80% of aircraft value

/// Loan management resource.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoanPortfolio {
    pub loans: Vec<Loan>,
    pub last_payment_day: u32,
    pub total_interest_paid: u32,
}

impl LoanPortfolio {
    pub fn total_debt(&self) -> u32 {
        self.loans
            .iter()
            .filter(|l| l.active)
            .map(|l| l.remaining_balance)
            .sum()
    }

    pub fn total_monthly_payment(&self) -> u32 {
        self.loans
            .iter()
            .filter(|l| l.active)
            .map(|l| l.monthly_payment)
            .sum()
    }

    pub fn active_loan_count(&self) -> usize {
        self.loans.iter().filter(|l| l.active).count()
    }

    pub fn loan_for_aircraft(&self, aircraft_id: &str) -> Option<&Loan> {
        self.loans
            .iter()
            .find(|l| l.aircraft_id == aircraft_id && l.active)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Take a loan to purchase an aircraft.
pub fn take_loan(
    mut purchase_events: EventReader<PurchaseEvent>,
    pilot_state: Res<PilotState>,
    aircraft_registry: Res<AircraftRegistry>,
    gold: Res<Gold>,
    mut portfolio: ResMut<LoanPortfolio>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in purchase_events.read() {
        // Loan purchases are prefixed with "loan_"
        if !ev.item_id.starts_with("loan_") {
            continue;
        }

        let aircraft_id = ev.item_id.trim_start_matches("loan_");
        let Some(aircraft_def) = aircraft_registry.get(aircraft_id) else {
            continue;
        };

        let max_loan = (aircraft_def.purchase_price as f32 * MAX_LTV) as u32;
        let down_payment = aircraft_def.purchase_price - max_loan;

        if gold.amount < down_payment {
            toast_events.send(ToastEvent {
                message: format!(
                    "Need {}g down payment (20% of {}g).",
                    down_payment, aircraft_def.purchase_price
                ),
                duration_secs: 4.0,
            });
            continue;
        }

        let rate = interest_rate_for_rank(pilot_state.rank);
        let term = 12; // 12 game-months (seasons)
        let loan = Loan::new(aircraft_id, max_loan, rate, term);

        gold_events.send(GoldChangeEvent {
            amount: -(down_payment as i32),
            reason: format!("Down payment for {}", aircraft_def.name),
        });

        toast_events.send(ToastEvent {
            message: format!(
                "💰 Loan approved: {}g at {:.0}% APR. Monthly payment: {}g. Down: {}g.",
                loan.principal,
                rate * 100.0,
                loan.monthly_payment,
                down_payment
            ),
            duration_secs: 5.0,
        });

        portfolio.loans.push(loan);
    }
}

/// Process monthly loan payments on day end.
pub fn make_payment(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    mut portfolio: ResMut<LoanPortfolio>,
    gold: Res<Gold>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        let day = calendar.total_days();

        // Monthly payments (every 28 game-days)
        if day.saturating_sub(portfolio.last_payment_day) < 28 {
            continue;
        }
        portfolio.last_payment_day = day;

        let total_payment = portfolio.total_monthly_payment();
        if total_payment == 0 {
            continue;
        }

        if gold.amount >= total_payment {
            // Process payments for each active loan
            let mut interest_paid = 0_u32;
            let mut paid_off = Vec::new();

            for loan in &mut portfolio.loans {
                if !loan.active {
                    continue;
                }

                let interest = (loan.remaining_balance as f32 * loan.interest_rate / 12.0) as u32;
                let principal_paid = loan.monthly_payment.saturating_sub(interest);

                loan.remaining_balance = loan.remaining_balance.saturating_sub(principal_paid);
                loan.months_paid += 1;
                interest_paid += interest;

                // Loan paid off
                if loan.remaining_balance == 0 || loan.months_paid >= loan.term_months {
                    loan.active = false;
                    loan.remaining_balance = 0;
                    paid_off.push(loan.aircraft_id.clone());
                }
            }

            portfolio.total_interest_paid += interest_paid;

            for aircraft_id in &paid_off {
                toast_events.send(ToastEvent {
                    message: format!(
                        "🎉 Loan for {} paid off! Aircraft is fully yours.",
                        aircraft_id
                    ),
                    duration_secs: 4.0,
                });
            }

            let remaining_debt = portfolio.total_debt();

            gold_events.send(GoldChangeEvent {
                amount: -(total_payment as i32),
                reason: "Loan payments".to_string(),
            });

            toast_events.send(ToastEvent {
                message: format!(
                    "Loan payment: -{}g. Remaining debt: {}g",
                    total_payment, remaining_debt
                ),
                duration_secs: 3.0,
            });
        } else {
            // Missed payment
            for loan in &mut portfolio.loans {
                if loan.active {
                    loan.missed_payments += 1;
                }
            }

            let missed = portfolio
                .loans
                .iter()
                .find(|l| l.active)
                .map(|l| l.missed_payments)
                .unwrap_or(0);

            toast_events.send(ToastEvent {
                message: format!("⚠ Missed loan payment! ({}/3 before repossession)", missed),
                duration_secs: 4.0,
            });
        }
    }
}

/// Repossess aircraft if loan defaults (3 missed payments).
pub fn check_loan_default(
    mut portfolio: ResMut<LoanPortfolio>,
    mut fleet: ResMut<Fleet>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let mut repossessed = Vec::new();

    for loan in &mut portfolio.loans {
        if loan.active && loan.is_defaulted() {
            loan.active = false;
            repossessed.push(loan.aircraft_id.clone());
        }
    }

    for aircraft_id in &repossessed {
        fleet.aircraft.retain(|a| a.aircraft_id != *aircraft_id);
        if fleet.active_index >= fleet.aircraft.len() && !fleet.aircraft.is_empty() {
            fleet.active_index = 0;
        }

        toast_events.send(ToastEvent {
            message: format!(
                "⚠ {} has been repossessed due to loan default!",
                aircraft_id
            ),
            duration_secs: 5.0,
        });
    }
}

/// Early payoff of a loan.
pub fn early_payoff(
    mut purchase_events: EventReader<PurchaseEvent>,
    mut portfolio: ResMut<LoanPortfolio>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in purchase_events.read() {
        if !ev.item_id.starts_with("payoff_") {
            continue;
        }

        let aircraft_id = ev.item_id.trim_start_matches("payoff_");
        let Some(loan) = portfolio
            .loans
            .iter_mut()
            .find(|l| l.aircraft_id == aircraft_id && l.active)
        else {
            continue;
        };

        let payoff = loan.early_payoff_amount();
        loan.active = false;
        loan.remaining_balance = 0;

        gold_events.send(GoldChangeEvent {
            amount: -(payoff as i32),
            reason: format!("Early loan payoff for {}", aircraft_id),
        });

        toast_events.send(ToastEvent {
            message: format!(
                "🎉 Loan paid off early for {}! Saved {}g with early payoff discount.",
                aircraft_id,
                loan.remaining_balance.saturating_sub(payoff)
            ),
            duration_secs: 4.0,
        });
    }
}

/// Display financial overview in debug/UI.
pub fn loan_financial_summary(portfolio: &LoanPortfolio) -> String {
    let active = portfolio.active_loan_count();
    let debt = portfolio.total_debt();
    let monthly = portfolio.total_monthly_payment();
    format!(
        "Loans: {} active | Total debt: {}g | Monthly: {}g | Interest paid: {}g",
        active, debt, monthly, portfolio.total_interest_paid
    )
}
