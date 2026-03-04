# Worker: WIRE 8 PILOT UI SCREENS

## Context
The pilot DLC at `/home/user/hearthfield/dlc/pilot` has 8 UI screen files that exist but are not wired to GameState transitions. You need to:
1. Add 8 new GameState variants to `dlc/pilot/src/shared/mod.rs`
2. Wire each screen's spawn/despawn/input systems in `dlc/pilot/src/ui/mod.rs`

## Required reading (read these files before writing any code)
1. `dlc/pilot/src/shared/mod.rs` — the type contract, find `enum GameState`
2. `dlc/pilot/src/ui/mod.rs` — see how existing screens (RadioComm, CrewLounge, Cutscene, Shop) are wired
3. Each of the 8 screen files listed below — understand what systems they export

## Step 1: Add GameState variants
In `dlc/pilot/src/shared/mod.rs`, add these variants to `enum GameState` (before the closing `}`):
```
Logbook,
Profile,
Achievements,
Settings,
MapView,
Notifications,
Tutorial,
Intro,
```

## Step 2: Wire each screen in `dlc/pilot/src/ui/mod.rs`
For each screen, follow the EXACT pattern used for RadioComm/CrewLounge/Cutscene:
- `OnEnter(GameState::X)` → spawn system
- `OnExit(GameState::X)` → despawn system
- `Update` with `run_if(in_state(GameState::X))` → input handler

The 8 screens and their files:
1. `achievement_screen.rs` → GameState::Achievements
2. `logbook_screen.rs` → GameState::Logbook
3. `map_screen.rs` → GameState::MapView
4. `notification_center.rs` → GameState::Notifications
5. `profile_screen.rs` → GameState::Profile
6. `tutorial.rs` → GameState::Tutorial
7. `intro_sequence.rs` → GameState::Intro
8. `settings.rs` → GameState::Settings

For each file, read it first to find what pub fn systems are exported (spawn_, despawn_, handle_ functions). If a screen file has no input handler, just wire spawn/despawn.

## Step 3: Add keybinds for screens that need them
In the player input handling (likely `dlc/pilot/src/input/mod.rs` or wherever key bindings are processed), add keybinds for screens that should be accessible from Playing state:
- L → Logbook
- P → Profile
- Tab → Achievements (or whatever makes sense)
- Only add binds if there's an existing pattern for screen toggle keys

## Validation (run before reporting done)
```bash
cd /home/user/hearthfield/dlc/pilot && cargo check 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo test --test headless 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo clippy -- -D warnings 2>&1
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to `/home/user/hearthfield/status/workers/fix-pilot-wire-screens.md`
