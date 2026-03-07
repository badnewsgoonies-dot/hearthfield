# Worker: PILOT LOAD GAME SCREEN

## Context
The pilot DLC main menu has a "Load Game" button that does nothing (TODO at line ~94 of `dlc/pilot/src/ui/main_menu.rs`). You need to implement the Load Game screen.

## Required reading
1. `dlc/pilot/src/ui/main_menu.rs` — find the MenuAction::Load TODO
2. `dlc/pilot/src/save/mod.rs` — understand SaveFile structure, save_game/load_game functions
3. `dlc/pilot/src/shared/mod.rs` — find GameState enum, SaveSlots resource
4. `dlc/pilot/src/ui/mod.rs` — see screen wiring pattern

## Deliverables

### 1. New file: `dlc/pilot/src/ui/load_screen.rs`
- Show list of save slots (from `SaveSlots` resource)
- Each slot shows: slot number, pilot name, play time, last save date
- Empty slots shown as "— Empty —"
- Click or Enter to load selected save
- Esc to return to MainMenu
- Systems: `spawn_load_screen`, `despawn_load_screen`, `handle_load_input`

### 2. Add GameState::LoadGame variant
In `dlc/pilot/src/shared/mod.rs`, add `LoadGame` to enum GameState.

### 3. Wire in ui/mod.rs
Add OnEnter/OnExit/Update systems for GameState::LoadGame, following the same pattern as other screens.

### 4. Fix the TODO in main_menu.rs
Replace `MenuAction::Load => { /* TODO: open load screen */ }` with:
```rust
MenuAction::Load => {
    next_state.set(GameState::LoadGame);
}
```

### 5. Load logic
When a save slot is selected, call the existing `load_game` function from save/mod.rs, then transition to GameState::Playing.

## Validation
```bash
cd /home/user/hearthfield/dlc/pilot && cargo check 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo test --test headless 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo clippy -- -D warnings 2>&1
```
Done = all three pass with zero errors/warnings.

## When done
Write completion report to `/home/user/hearthfield/status/workers/fix-pilot-load-game.md`
