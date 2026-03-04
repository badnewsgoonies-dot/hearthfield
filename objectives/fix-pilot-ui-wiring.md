# Worker: FIX-PILOT-UI-WIRING (Wire Existing Screens + Fix Shop)

## Scope
You may modify files under: dlc/pilot/src/

## Required reading
1. dlc/pilot/src/ui/mod.rs (FULL file — see which screens are wired and which are not)
2. dlc/pilot/src/shared/mod.rs (GameState enum — see existing variants: RadioComm, CrewLounge, Cutscene)
3. dlc/pilot/src/ui/radio_screen.rs (has spawn_radio_screen, despawn_radio_screen, handle_radio_input)
4. dlc/pilot/src/ui/crew_screen.rs (has spawn_crew_screen, despawn_crew_screen, handle_crew_screen_input)
5. dlc/pilot/src/ui/cutscene_runner.rs (has spawn_cutscene, despawn_cutscene, update_cutscene)
6. dlc/pilot/src/ui/shop_screen.rs (has spawn_shop_screen — but NO buy button handler)
7. dlc/pilot/src/economy/shop.rs (has PurchaseEvent, handle_purchase — the backend)
8. dlc/pilot/src/ui/inventory_screen.rs (compare with shop for Interaction pattern)

## Task 1: Wire 3 Existing GameState Screens

These GameState variants already exist but have NO OnEnter/OnExit handlers in UiPlugin:

### RadioComm (GameState::RadioComm)
In `dlc/pilot/src/ui/mod.rs`, add to `UiPlugin::build`:
```rust
.add_systems(OnEnter(GameState::RadioComm), radio_screen::spawn_radio_screen)
.add_systems(OnExit(GameState::RadioComm), radio_screen::despawn_radio_screen)
.add_systems(Update, radio_screen::handle_radio_input.run_if(in_state(GameState::RadioComm)))
```

### CrewLounge (GameState::CrewLounge)
```rust
.add_systems(OnEnter(GameState::CrewLounge), crew_screen::spawn_crew_screen)
.add_systems(OnExit(GameState::CrewLounge), crew_screen::despawn_crew_screen)
.add_systems(Update, crew_screen::handle_crew_screen_input.run_if(in_state(GameState::CrewLounge)))
```

### Cutscene (GameState::Cutscene)
```rust
.add_systems(OnEnter(GameState::Cutscene), cutscene_runner::spawn_cutscene)
.add_systems(OnExit(GameState::Cutscene), cutscene_runner::despawn_cutscene)
.add_systems(Update, cutscene_runner::update_cutscene.run_if(in_state(GameState::Cutscene)))
```

**IMPORTANT**: Read each screen's .rs file first to confirm the exact function names. The names above are my best guess — verify before using.

## Task 2: Fix Shop Screen Buy Buttons

The shop screen at `dlc/pilot/src/ui/shop_screen.rs` spawns "Buy" buttons but has NO `Interaction` handler. Players can see items but cannot purchase anything.

**Fix**: Add a `handle_shop_buy` system that:
1. Queries `Query<(&Interaction, &Parent), (Changed<Interaction>, With<Button>)>` (or similar)
2. On `Interaction::Pressed`, identify which item was clicked
3. Fire a `PurchaseEvent` with the item info
4. The backend in `economy/shop.rs` already handles `PurchaseEvent`

You'll need to:
- Add a marker component (e.g., `ShopBuyButton(usize)`) to each buy button when spawning, storing the item index
- Add a `handle_shop_buy` system reading `Interaction` changes
- Register it in UiPlugin with `.run_if(in_state(GameState::Shop))`
- Also add visual feedback: change button color on hover (Interaction::Hovered)

Reference the existing `handle_mission_screen_input` system for the interaction pattern used in this codebase.

## Task 3: Add Keyboard Navigation for Screens

Ensure each newly-wired screen has Esc to return to previous state:
- RadioComm → Playing (or Flying, whichever was active)
- CrewLounge → Playing
- Cutscene: auto-advance or skip with Esc

## Validation
```
cd dlc/pilot && cargo check && cargo test --test headless && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-pilot-ui-wiring.md
