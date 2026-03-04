# Worker: PILOT LOAN/INSURANCE/BUSINESS UI SCREENS

## Context
The pilot DLC has complete backend systems for loans, insurance, and airline business management, but NO UI screens for any of them. You need to create 3 new UI screens.

## Required reading
1. `dlc/pilot/src/economy/loans.rs` — Loan, LoanPortfolio, interest_rate_for_rank()
2. `dlc/pilot/src/economy/insurance.rs` — InsurancePolicy, InsuranceClaim, coverage types
3. `dlc/pilot/src/economy/business.rs` — AirlineBusiness, HiredPilot, BusinessRoute
4. `dlc/pilot/src/shared/mod.rs` — GameState, Gold, PilotRank
5. `dlc/pilot/src/ui/shop_screen.rs` — reference for button/interaction pattern
6. `dlc/pilot/src/ui/mod.rs` — screen wiring pattern

## Deliverables

### 1. Add GameState variants to `dlc/pilot/src/shared/mod.rs`
Add: `LoanOffice`, `InsuranceOffice`, `BusinessHQ`

### 2. New file: `dlc/pilot/src/ui/loan_screen.rs`
- Show active loans from LoanPortfolio with remaining balance, interest rate, monthly payment
- "Take Loan" button: creates new Loan (use LoanPortfolio methods)
- "Pay Off" button: early payoff for selected loan
- Display available loan amounts based on PilotRank
- Systems: spawn_loan_screen, despawn_loan_screen, handle_loan_input

### 3. New file: `dlc/pilot/src/ui/insurance_screen.rs`
- Show current InsurancePolicy (if any) with coverage type, premium, deductible
- Buttons to buy/upgrade policy: Basic → Standard → Premium
- Show active claims
- Systems: spawn_insurance_screen, despawn_insurance_screen, handle_insurance_input

### 4. New file: `dlc/pilot/src/ui/business_screen.rs`
- Show AirlineBusiness overview: fleet size, employee count, active routes, revenue/expenses
- List of BusinessRoutes with income
- "Hire Pilot" and "Add Route" buttons (fire events, don't need full implementation)
- Systems: spawn_business_screen, despawn_business_screen, handle_business_input

### 5. Wire all 3 screens in `dlc/pilot/src/ui/mod.rs`
Follow existing pattern for OnEnter/OnExit/Update with run_if.

### 6. Register modules in `dlc/pilot/src/ui/mod.rs`
Add `pub mod loan_screen;`, `pub mod insurance_screen;`, `pub mod business_screen;`

## Validation
```bash
cd /home/user/hearthfield/dlc/pilot && cargo check 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo test --test headless 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo clippy -- -D warnings 2>&1
```
Done = all three pass with zero errors/warnings.

## When done
Write completion report to `/home/user/hearthfield/status/workers/fix-pilot-economy-ui.md`
