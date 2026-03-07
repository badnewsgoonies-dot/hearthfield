//! Airline business management — start and grow your own airline.
//!
//! At Captain rank the player can establish an airline, hire pilots,
//! plan routes for passive income, and grow toward business milestones.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Airline Business Resource ───────────────────────────────────────────

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct AirlineBusiness {
    pub name: String,
    pub hub_airport: AirportId,
    pub established: bool,
    pub fleet_size: u32,
    pub employee_count: u32,
    pub monthly_revenue: u32,
    pub monthly_expenses: u32,
    pub reputation: f32,
    pub hired_pilots: Vec<HiredPilot>,
    pub active_routes: Vec<BusinessRoute>,
    pub loan_amount: u32,
    pub loan_interest_rate: f32,
    pub milestones_reached: Vec<BusinessMilestone>,
}

impl Default for AirlineBusiness {
    fn default() -> Self {
        Self {
            name: "My Airline".into(),
            hub_airport: AirportId::HomeBase,
            established: false,
            fleet_size: 0,
            employee_count: 0,
            monthly_revenue: 0,
            monthly_expenses: 0,
            reputation: 50.0,
            hired_pilots: Vec::new(),
            active_routes: Vec::new(),
            loan_amount: 0,
            loan_interest_rate: 0.05,
            milestones_reached: Vec::new(),
        }
    }
}

impl AirlineBusiness {
    pub fn monthly_profit(&self) -> i32 {
        self.monthly_revenue as i32 - self.monthly_expenses as i32
    }

    pub fn is_profitable(&self) -> bool {
        self.monthly_profit() > 0
    }
}

// ─── Hired Pilot ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HiredPilot {
    pub name: String,
    pub skill: f32,
    pub salary: u32,
    pub assigned_route: Option<String>,
    pub flights_completed: u32,
    pub morale: f32,
}

impl HiredPilot {
    pub fn generate(name: &str, skill: f32) -> Self {
        let salary = (500.0 + skill * 1500.0) as u32;
        Self {
            name: name.to_string(),
            skill,
            salary,
            assigned_route: None,
            flights_completed: 0,
            morale: 75.0,
        }
    }

    pub fn effectiveness(&self) -> f32 {
        self.skill * (self.morale / 100.0)
    }
}

// ─── Business Routes ─────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BusinessRoute {
    pub id: String,
    pub origin: AirportId,
    pub destination: AirportId,
    pub ticket_price: u32,
    pub daily_passengers: u32,
    pub operating_cost: u32,
    pub competition_factor: f32,
    pub assigned_aircraft: Option<String>,
}

impl BusinessRoute {
    pub fn daily_revenue(&self) -> u32 {
        let effective_passengers =
            (self.daily_passengers as f32 * (1.0 - self.competition_factor * 0.3)) as u32;
        effective_passengers * self.ticket_price
    }

    pub fn daily_profit(&self) -> i32 {
        self.daily_revenue() as i32 - self.operating_cost as i32
    }
}

/// Generate a route between two airports with pricing.
pub fn create_route(origin: AirportId, destination: AirportId, reputation: f32) -> BusinessRoute {
    let distance = airport_distance(origin, destination);
    let base_price = (distance * 0.5) as u32 + 20;
    let rep_multiplier = 1.0 + (reputation - 50.0) * 0.005;
    let ticket_price = (base_price as f32 * rep_multiplier) as u32;

    let daily_passengers = match distance as u32 {
        0..=200 => 15,
        201..=400 => 25,
        401..=600 => 40,
        _ => 50,
    };

    let operating_cost = (distance * 0.8) as u32 + 100;

    // Competition: busier routes have more competitors
    let competition = match destination {
        AirportId::Grandcity => 0.6,
        AirportId::Sunhaven => 0.4,
        AirportId::Windport => 0.3,
        AirportId::Skyreach => 0.2,
        _ => 0.15,
    };

    BusinessRoute {
        id: format!("{:?}-{:?}", origin, destination),
        origin,
        destination,
        ticket_price,
        daily_passengers,
        operating_cost,
        competition_factor: competition,
        assigned_aircraft: None,
    }
}

// ─── Business Milestones ─────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BusinessMilestone {
    Founded,
    FirstRoute,
    FirstHire,
    FiveRoutes,
    TenEmployees,
    ProfitableMonth,
    Revenue100k,
    Revenue1m,
    AllAirportsCovered,
    FleetOfTen,
}

impl BusinessMilestone {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Founded => "Airline Founded",
            Self::FirstRoute => "First Route Established",
            Self::FirstHire => "First Employee Hired",
            Self::FiveRoutes => "Five Active Routes",
            Self::TenEmployees => "Ten Employees",
            Self::ProfitableMonth => "First Profitable Month",
            Self::Revenue100k => "100,000G Revenue",
            Self::Revenue1m => "1,000,000G Revenue",
            Self::AllAirportsCovered => "All Airports Served",
            Self::FleetOfTen => "Fleet of Ten Aircraft",
        }
    }

    pub fn reward_gold(&self) -> u32 {
        match self {
            Self::Founded => 0,
            Self::FirstRoute => 500,
            Self::FirstHire => 200,
            Self::FiveRoutes => 2000,
            Self::TenEmployees => 3000,
            Self::ProfitableMonth => 1000,
            Self::Revenue100k => 5000,
            Self::Revenue1m => 20000,
            Self::AllAirportsCovered => 10000,
            Self::FleetOfTen => 5000,
        }
    }
}

// ─── Financial Report ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct MonthlyReport {
    pub revenue_breakdown: HashMap<String, u32>,
    pub expense_breakdown: HashMap<String, u32>,
    pub total_revenue: u32,
    pub total_expenses: u32,
    pub net_profit: i32,
    pub route_performance: Vec<(String, i32)>,
}

pub fn generate_monthly_report(business: &AirlineBusiness) -> MonthlyReport {
    let mut revenue_breakdown = HashMap::new();
    let mut expense_breakdown = HashMap::new();
    let mut route_perf = Vec::new();

    for route in &business.active_routes {
        let monthly_rev = route.daily_revenue() * 28;
        let monthly_cost = route.operating_cost * 28;
        revenue_breakdown
            .entry(format!("Route: {}", route.id))
            .and_modify(|v| *v += monthly_rev)
            .or_insert(monthly_rev);
        route_perf.push((route.id.clone(), (monthly_rev as i32 - monthly_cost as i32)));
    }

    let salary_total: u32 = business.hired_pilots.iter().map(|p| p.salary).sum();
    expense_breakdown.insert("Pilot salaries".into(), salary_total);
    expense_breakdown.insert(
        "Fuel & maintenance".into(),
        business.monthly_expenses.saturating_sub(salary_total),
    );

    let total_revenue: u32 = revenue_breakdown.values().sum();
    let total_expenses: u32 = expense_breakdown.values().sum();

    MonthlyReport {
        revenue_breakdown,
        expense_breakdown,
        total_revenue,
        total_expenses,
        net_profit: total_revenue as i32 - total_expenses as i32,
        route_performance: route_perf,
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Establish airline (triggered when Captain rank is reached and player confirms).
pub fn establish_airline(
    input: Res<PlayerInput>,
    pilot: Res<PilotState>,
    mut business: ResMut<AirlineBusiness>,
    location: Res<PlayerLocation>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !input.confirm || business.established || pilot.rank < PilotRank::Captain {
        return;
    }

    business.established = true;
    business.hub_airport = location.airport;
    business.name = format!("{}'s Airlines", pilot.name);
    business.milestones_reached.push(BusinessMilestone::Founded);

    toast.send(ToastEvent {
        message: format!(
            "★ {} established at {}! ★",
            business.name,
            location.airport.display_name()
        ),
        duration_secs: 5.0,
    });
}

/// Process passive income from active routes each day.
pub fn process_daily_route_income(
    mut day_events: EventReader<DayEndEvent>,
    mut business: ResMut<AirlineBusiness>,
    mut gold: ResMut<Gold>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    for _evt in day_events.read() {
        if !business.established {
            continue;
        }

        let mut total_income: i32 = 0;

        for route in &business.active_routes {
            let profit = route.daily_profit();
            total_income += profit;
        }

        // Deduct salaries (daily portion)
        let daily_salary: u32 = business.hired_pilots.iter().map(|p| p.salary / 28).sum();
        total_income -= daily_salary as i32;

        if total_income > 0 {
            gold.amount += total_income as u32;
            gold_events.send(GoldChangeEvent {
                amount: total_income,
                reason: format!("{} daily income", business.name),
            });
        } else if total_income < 0 {
            let loss = total_income.unsigned_abs();
            gold.amount = gold.amount.saturating_sub(loss);
            gold_events.send(GoldChangeEvent {
                amount: total_income,
                reason: format!("{} daily expenses", business.name),
            });
        }

        // Update monthly tracking
        if total_income > 0 {
            business.monthly_revenue += total_income as u32;
        } else {
            business.monthly_expenses += total_income.unsigned_abs();
        }
    }
}

/// Check and award business milestones.
pub fn check_business_milestones(
    mut business: ResMut<AirlineBusiness>,
    mut gold: ResMut<Gold>,
    mut toast: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    if !business.established {
        return;
    }

    let checks: Vec<(BusinessMilestone, bool)> = vec![
        (
            BusinessMilestone::FirstRoute,
            !business.active_routes.is_empty(),
        ),
        (
            BusinessMilestone::FirstHire,
            !business.hired_pilots.is_empty(),
        ),
        (
            BusinessMilestone::FiveRoutes,
            business.active_routes.len() >= 5,
        ),
        (
            BusinessMilestone::TenEmployees,
            business.hired_pilots.len() >= 10,
        ),
        (BusinessMilestone::ProfitableMonth, business.is_profitable()),
        (BusinessMilestone::FleetOfTen, business.fleet_size >= 10),
    ];

    for (milestone, condition) in checks {
        if condition && !business.milestones_reached.contains(&milestone) {
            business.milestones_reached.push(milestone);
            let reward = milestone.reward_gold();
            if reward > 0 {
                gold.amount += reward;
                gold_events.send(GoldChangeEvent {
                    amount: reward as i32,
                    reason: format!("Milestone: {}", milestone.display_name()),
                });
            }
            toast.send(ToastEvent {
                message: format!("🏆 Business milestone: {}!", milestone.display_name()),
                duration_secs: 4.0,
            });
        }
    }
}

/// Update hired pilot morale based on workload.
pub fn update_pilot_morale(mut business: ResMut<AirlineBusiness>) {
    if !business.established {
        return;
    }

    for pilot in &mut business.hired_pilots {
        if pilot.assigned_route.is_some() {
            // Working pilots slowly lose morale
            pilot.morale = (pilot.morale - 0.5).max(20.0);
        } else {
            // Idle pilots recover morale
            pilot.morale = (pilot.morale + 1.0).min(100.0);
        }
    }
}

/// Take a business loan for expansion.
pub fn take_business_loan(
    input: Res<PlayerInput>,
    pilot: Res<PilotState>,
    mut business: ResMut<AirlineBusiness>,
    mut gold: ResMut<Gold>,
    mut toast: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    // Loan triggered by hotbar_1 while at Captain rank with airline established
    if !input.hotbar_1 || !business.established || pilot.rank < PilotRank::Captain {
        return;
    }
    if business.loan_amount > 0 {
        toast.send(ToastEvent {
            message: "You already have an outstanding loan.".into(),
            duration_secs: 3.0,
        });
        return;
    }

    let loan = 10000_u32;
    business.loan_amount = loan;
    gold.amount += loan;
    gold_events.send(GoldChangeEvent {
        amount: loan as i32,
        reason: "Business loan".into(),
    });

    toast.send(ToastEvent {
        message: format!(
            "Loan approved: {}G at {:.0}% interest.",
            loan,
            business.loan_interest_rate * 100.0
        ),
        duration_secs: 4.0,
    });
}
