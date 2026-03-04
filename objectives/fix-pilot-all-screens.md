# Worker: PILOT DLC — ALL MISSING SCREENS (GameState + Wiring + Load + Economy UI)

## Context
The pilot DLC at `/home/user/hearthfield/dlc/pilot` has many UI screen files that aren't wired, plus missing Load Game and economy management screens. You will:
1. Add 12 new GameState variants
2. Wire 8 existing screens
3. Create Load Game screen
4. Create 3 economy UI screens (Loan, Insurance, Business)

## CRITICAL: Read these files first
1. `dlc/pilot/src/shared/mod.rs` — find `enum GameState` (currently has 12 variants ending at MissionBoard)
2. `dlc/pilot/src/ui/mod.rs` — see how screens are wired (OnEnter/OnExit/Update pattern)
3. `dlc/pilot/src/ui/main_menu.rs` — find `MenuAction::Load` TODO
4. `dlc/pilot/src/save/mod.rs` — understand SaveFile, save_game, load_game functions
5. `dlc/pilot/src/economy/loans.rs` — Loan, LoanPortfolio
6. `dlc/pilot/src/economy/insurance.rs` — InsurancePolicy, InsuranceClaim
7. `dlc/pilot/src/economy/business.rs` — AirlineBusiness, HiredPilot, BusinessRoute
8. Each of these 8 existing screen files to find their exported systems:
   - `dlc/pilot/src/ui/achievement_screen.rs`
   - `dlc/pilot/src/ui/logbook_screen.rs`
   - `dlc/pilot/src/ui/map_screen.rs`
   - `dlc/pilot/src/ui/notification_center.rs`
   - `dlc/pilot/src/ui/profile_screen.rs`
   - `dlc/pilot/src/ui/tutorial.rs`
   - `dlc/pilot/src/ui/intro_sequence.rs`
   - `dlc/pilot/src/ui/settings.rs`

## Part 1: Add GameState variants
In `dlc/pilot/src/shared/mod.rs` add these to `enum GameState`:
```rust
    LoadGame,
    Logbook,
    Profile,
    Achievements,
    Settings,
    MapView,
    Notifications,
    Tutorial,
    Intro,
    LoanOffice,
    InsuranceOffice,
    BusinessHQ,
```

## Part 2: Wire 8 existing screens in `dlc/pilot/src/ui/mod.rs`
Read each screen file to find what pub fn systems it exports (spawn_, despawn_, handle_ functions).
For each, add:
- `OnEnter(GameState::X)` → spawn
- `OnExit(GameState::X)` → despawn
- `Update` with `run_if(in_state(GameState::X))` → input handler (if one exists)

## Part 3: Load Game screen
Create `dlc/pilot/src/ui/load_screen.rs`:
- Show save slots from `SaveSlots` resource
- Each slot: slot number, pilot name (or "— Empty —")
- Click/Enter to load, Esc to go back to MainMenu
- On load: call existing load_game function, transition to Playing
- Systems: spawn_load_screen, despawn_load_screen, handle_load_input

Fix `main_menu.rs` TODO:
```rust
MenuAction::Load => { next_state.set(GameState::LoadGame); }
```

Wire LoadGame state in ui/mod.rs.
Add `pub mod load_screen;` in ui/mod.rs.

## Part 4: Economy UI screens (3 new files)

### `dlc/pilot/src/ui/loan_screen.rs` (~80-120 lines)
- Read LoanPortfolio to show active loans
- "Take Loan" button, "Pay Off" button
- Display interest rates from economy backend
- Esc to return to Playing

### `dlc/pilot/src/ui/insurance_screen.rs` (~80-120 lines)
- Read InsuranceState for current policy
- Buy/upgrade buttons: Basic → Standard → Premium
- Show claims history
- Esc to return to Playing

### `dlc/pilot/src/ui/business_screen.rs` (~80-120 lines)
- Read AirlineBusiness for overview
- Show routes, fleet, employees, revenue
- Placeholder "Hire" and "Add Route" buttons
- Esc to return to Playing

Wire all 3 in ui/mod.rs. Add `pub mod` for each.

## Validation
```bash
cd /home/user/hearthfield/dlc/pilot && cargo check 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo test --test headless 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo clippy -- -D warnings 2>&1
```
Done = all three pass with zero errors/warnings.

## When done
Write completion report to `/home/user/hearthfield/status/workers/fix-pilot-all-screens.md`
