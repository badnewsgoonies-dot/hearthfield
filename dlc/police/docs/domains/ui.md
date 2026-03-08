# UI Domain Spec — Precinct

## Purpose
Main menu, HUD, and pause menu. The player-facing screens for Wave 1.

## Scope
`src/domains/ui/` — owns all UI rendering and menu state transitions.

## What This Domain Does
- **Main Menu** (OnEnter GameState::MainMenu): "New Game" and "Quit" buttons
  - New Game → transition to GameState::Playing
  - Quit → app exit
- **HUD** (GameState::Playing): persistent overlay showing:
  - Shift clock: current hour:minute, day of week
  - Fatigue bar: 0-100, green→yellow→red gradient
  - Stress bar: 0-100, blue→purple→red gradient
  - Gold amount
  - Current rank badge text
  - On-duty / off-duty indicator
  - Weather icon (text: "Clear"/"Rainy"/"Foggy"/"Snowy")
- **Pause Menu** (OnEnter GameState::Paused): "Resume" and "Quit to Menu" buttons
  - Resume → back to GameState::Playing
  - Quit to Menu → GameState::MainMenu
- Read Escape key to toggle Playing ↔ Paused

## What This Domain Does NOT Do
- Game logic (calendar, player, world, cases, etc.)
- Save/load (save domain, future wave)
- Case file screens, evidence exam, interrogation UI (future waves)

## Key Types (import from crate::shared)
- `GameState` — for state transitions and OnEnter/OnExit gating
- `ShiftClock` (Resource — read for HUD display)
- `PlayerState` (Resource — read fatigue, stress, gold)
- `PlayerInput` (Resource — read menu/cancel for pause toggle)
- Constants: `SCREEN_WIDTH`, `SCREEN_HEIGHT`, `MAX_FATIGUE`, `MAX_STRESS`

## Systems to Implement
1. `spawn_main_menu` — `OnEnter(GameState::MainMenu)`
   - Spawn "Precinct" title text, "New Game" button, "Quit" button
   - Use Bevy UI nodes (Node, Button, Text)
2. `handle_main_menu_buttons` — `Update`, gated on `GameState::MainMenu`
   - Button interaction → state transition
3. `cleanup_main_menu` — `OnExit(GameState::MainMenu)` — despawn menu entities
4. `spawn_hud` — `OnEnter(GameState::Playing)`
   - Top bar: clock, weather, rank, on-duty indicator
   - Bottom-left: fatigue bar, stress bar
   - Bottom-right: gold display
5. `update_hud` — `Update`, `UpdatePhase::Presentation`, gated on `GameState::Playing`
   - Read ShiftClock → update clock text, weather text, rank text
   - Read PlayerState → update fatigue bar width, stress bar width, gold text
6. `cleanup_hud` — `OnExit(GameState::Playing)` — despawn HUD entities
7. `toggle_pause` — `Update`, gated on `GameState::Playing`
   - If Escape pressed → GameState::Paused, set ShiftClock.time_paused = true
8. `spawn_pause_menu` — `OnEnter(GameState::Paused)`
   - Semi-transparent overlay, "Resume" and "Quit to Menu" buttons
9. `handle_pause_buttons` — `Update`, gated on `GameState::Paused`
   - Resume → Playing, set time_paused = false
   - Quit to Menu → MainMenu
10. `cleanup_pause_menu` — `OnExit(GameState::Paused)`

## Quantitative Targets
- 3 screens: main menu, HUD, pause menu
- HUD updates every frame from ShiftClock and PlayerState
- Fatigue bar: width proportional to fatigue/MAX_FATIGUE
- Stress bar: width proportional to stress/MAX_STRESS

## Decision Fields
- **Preferred**: Bevy native UI (Node, Button, Text) for all screens
- **Tempting alternative**: immediate-mode UI library (egui)
- **Consequence**: egui doesn't match the pixel art aesthetic; Bevy UI is the standard
- **Drift cue**: worker imports bevy_egui or writes custom immediate-mode renderer

- **Preferred**: simple colored rectangles for bars, plain text for labels
- **Tempting alternative**: styled UI with custom sprites, fancy layouts
- **Consequence**: over-engineering UI before gameplay exists wastes effort
- **Drift cue**: worker creates UI sprite assets or elaborate theming system

## Plugin Export
```rust
pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // OnEnter/OnExit for MainMenu, Playing, Paused
        // Update systems gated on appropriate states
    }
}
```

## Tests (minimum 5)
1. Main menu spawns with correct number of buttons
2. "New Game" button transitions to GameState::Playing
3. HUD spawns when entering Playing state
4. HUD displays correct fatigue value from PlayerState
5. Escape toggles to Paused state and sets time_paused
