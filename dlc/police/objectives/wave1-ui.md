# Worker: UI Domain (Wave 1)

## Scope (mechanically enforced)
You may only create/modify files under: `dlc/police/src/domains/ui/`

## Required reading (read BEFORE writing code)
1. `dlc/police/docs/spec.md`
2. `dlc/police/docs/domains/ui.md` (CRITICAL)
3. `dlc/police/src/shared/mod.rs`
4. `dlc/police/src/main.rs`

## Interpretation contract
Before coding, extract from ui.md:
- Bevy native UI (Node, Button, Text) — not egui
- Simple colored bars and plain text — not styled sprites
- 3 screens only: main menu, HUD, pause menu

## Required imports (from crate::shared)
- `GameState`
- `ShiftClock`, `PlayerState`
- `PlayerInput`
- `SCREEN_WIDTH`, `SCREEN_HEIGHT`, `MAX_FATIGUE`, `MAX_STRESS`

## Deliverables
Create `dlc/police/src/domains/ui/mod.rs` containing:
- `pub struct UiPlugin;` implementing `Plugin`

### Main Menu (GameState::MainMenu)
- `spawn_main_menu` — OnEnter(MainMenu)
  - Title: "PRECINCT" in large text, centered
  - "New Game" button → transitions to GameState::Playing
  - "Quit" button → AppExit event
  - Dark background
- `handle_main_menu_buttons` — Update, gated on MainMenu
- `cleanup_main_menu` — OnExit(MainMenu)

### HUD (GameState::Playing)
- `spawn_hud` — OnEnter(Playing)
  - Top bar: clock text (HH:MM), day of week, weather, rank, "ON DUTY"/"OFF DUTY"
  - Bottom-left: fatigue bar (green rect, width proportional to fatigue/100), stress bar (blue rect)
  - Bottom-right: gold amount text
- `update_hud` — Update, UpdatePhase::Presentation, gated on Playing
  - Read ShiftClock → format hour:minute, day_of_week display_name, weather, rank
  - Read PlayerState → fatigue bar width, stress bar width, gold text
- `cleanup_hud` — OnExit(Playing)

### Pause Menu (GameState::Paused)
- `toggle_pause` — Update, gated on Playing
  - Keyboard Escape → NextState(Paused), set ShiftClock.time_paused = true
- `spawn_pause_menu` — OnEnter(Paused)
  - Semi-transparent dark overlay
  - "PAUSED" title, "Resume" button, "Quit to Menu" button
- `handle_pause_buttons` — Update, gated on Paused
  - Resume → NextState(Playing), set time_paused = false
  - Quit to Menu → NextState(MainMenu)
- `cleanup_pause_menu` — OnExit(Paused)

## Quantitative targets
- 3 screens total
- HUD updates every frame
- Fatigue bar: node width = (fatigue / MAX_FATIGUE) * 200.0 pixels
- Stress bar: node width = (stress / MAX_STRESS) * 200.0 pixels
- Clock format: "06:00" (zero-padded)

## Failure patterns to avoid
- Using egui or any external UI library
- Creating sprite-based UI with texture atlases
- Forgetting OnExit cleanup (entity leak between states)
- Mutating ShiftClock or PlayerState (read-only except time_paused toggle)
- Building case file, evidence, interrogation, or skill tree screens (future waves)

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```

## When done
Write report to `dlc/police/status/workers/ui.md`
