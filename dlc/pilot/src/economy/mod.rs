//! Economy domain — gold management, XP/rank progression, shop, achievements.

use crate::shared::*;
use bevy::prelude::*;

pub mod achievements;
pub mod business;
pub mod finances;
pub mod gold;
pub mod insurance;
pub mod loans;
pub mod market;
pub mod progression;
pub mod shop;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<finances::FinancialLedger>()
            .init_resource::<market::MarketState>()
            .init_resource::<insurance::InsuranceState>()
            .init_resource::<loans::LoanPortfolio>()
            .init_resource::<business::AirlineBusiness>()
            .add_systems(
                Update,
                (
                    gold::apply_gold_changes.run_if(in_state(GameState::Playing)),
                    progression::apply_xp.run_if(in_state(GameState::Playing)),
                    progression::check_rank_up.run_if(in_state(GameState::Playing)),
                    achievements::check_achievements.run_if(in_state(GameState::Playing)),
                    shop::handle_purchase.run_if(in_state(GameState::Playing)),
                    finances::calculate_bonuses.run_if(in_state(GameState::Playing)),
                    finances::process_fuel_costs.run_if(in_state(GameState::Playing)),
                    finances::process_maintenance_costs.run_if(in_state(GameState::Playing)),
                    market::update_market_prices.run_if(in_state(GameState::Playing)),
                    market::seasonal_price_effects.run_if(in_state(GameState::Playing)),
                    insurance::purchase_insurance.run_if(in_state(GameState::Playing)),
                    insurance::process_premium_payments.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(
                Update,
                (
                    insurance::file_insurance_claim.run_if(in_state(GameState::Playing)),
                    insurance::enforce_insurance_requirement.run_if(in_state(GameState::Flying)),
                    loans::take_loan.run_if(in_state(GameState::Playing)),
                    loans::make_payment.run_if(in_state(GameState::Playing)),
                    loans::check_loan_default.run_if(in_state(GameState::Playing)),
                    loans::early_payoff.run_if(in_state(GameState::Playing)),
                    business::establish_airline.run_if(in_state(GameState::Playing)),
                    business::process_daily_route_income.run_if(in_state(GameState::Playing)),
                    business::check_business_milestones.run_if(in_state(GameState::Playing)),
                    business::update_pilot_morale.run_if(in_state(GameState::Playing)),
                    business::take_business_loan.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}
