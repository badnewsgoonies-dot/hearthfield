# Worker Report: FIX-PILOT-UI-WIRING

## Files Modified

- `dlc/pilot/src/ui/mod.rs` — wired RadioComm, CrewLounge, Cutscene states; registered shop buy/keyboard systems (~30 lines added)
- `dlc/pilot/src/ui/shop_screen.rs` — added `ShopBuyButton(usize)` marker, `handle_shop_buy`, `handle_shop_keyboard` (~50 lines added)
- `dlc/pilot/src/ui/crew_screen.rs` — added `handle_crew_screen_input` (Esc → Playing) (~10 lines added)

## What Was Implemented

### Task 1: Wired 3 GameState Screens
- **RadioComm**: `OnEnter` → `spawn_radio_screen`, `OnExit` → `despawn_radio_screen`, `Update` → `handle_radio_input` (already had Esc→Playing)
- **CrewLounge**: `OnEnter` → `spawn_crew_screen`, `OnExit` → `despawn_crew_screen`, `Update` → `handle_crew_screen_input` (new, Esc→Playing)
- **Cutscene**: `OnEnter` → `spawn_cutscene_overlay`, `OnExit` → `despawn_cutscene_overlay`, `Update` → `run_cutscene` + `skip_cutscene` (skip_cutscene handles Esc)
  - Note: actual function names in cutscene_runner.rs were `spawn_cutscene_overlay`/`despawn_cutscene_overlay`/`run_cutscene`/`skip_cutscene`, not the names in the spec

### Task 2: Shop Buy Button Handler
- Added `ShopBuyButton(pub usize)` component marking each button with its listing index
- `handle_shop_buy`: queries `Changed<Interaction>` on `ShopBuyButton` buttons, fires `PurchaseEvent` on Pressed, changes color on Hovered/None for visual feedback
- `handle_shop_keyboard`: Esc → Playing
- Both registered in UiPlugin under `.run_if(in_state(GameState::Shop))`

### Task 3: Keyboard Navigation
- RadioComm: already handled in `handle_radio_input` (Esc → Playing)
- CrewLounge: new `handle_crew_screen_input` (Esc → Playing)
- Cutscene: `skip_cutscene` (Esc clears steps and → Playing)
- Shop: new `handle_shop_keyboard` (Esc → Playing)

## Validation Results

- `cargo check`: PASS
- `cargo clippy -- -D warnings`: PASS
- `cargo test --test headless`: 75 passed, 1 failed (`test_mission_refresh`) — **pre-existing failure** introduced by prior worker changes to `src/missions/board.rs` (added `Res<StoryProgress>` not registered in test). Confirmed by reverting changes (git stash) and running that test in isolation — it passed on stash, meaning the failure is from prior unstaged changes to missions/board.rs unrelated to UI wiring.

## Known Risks
- `test_mission_refresh` failure needs to be fixed by the missions/story worker — `StoryProgress` resource needs to be registered in the headless test app.
