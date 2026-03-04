# Worker: CITY DLC — FULL UI LAYER (Main Menu, HUD, Pause, Summary, Task Board, Interruption UI)

## Context
The city office worker DLC at `/home/user/hearthfield/dlc/city` has a complete backend (40/40 tests) but ZERO UI. Currently the game is invisible — just colored sprites. You need to create the entire UI layer from scratch.

## CRITICAL: Read these files first (IN ORDER)
1. `dlc/city/src/game/mod.rs` — Plugin wiring, OfficeGameState enum, system sets
2. `dlc/city/src/game/resources.rs` — ALL resource types (WorkerStats, DayClock, InboxState, PlayerMindState, PlayerCareerState, TaskBoard, DayOutcome, etc.)
3. `dlc/city/src/game/events.rs` — ALL event types
4. `dlc/city/src/game/components.rs` — Entity components
5. `dlc/city/src/game/systems/bootstrap.rs` — setup_scene, boot_to_main_menu, main_menu_to_in_day, toggle_pause
6. `dlc/city/src/game/systems/day_cycle.rs` — Day end flow
7. `dlc/city/src/game/systems/visuals.rs` — Current sprite-only visual system
8. `dlc/city/src/main.rs` — App entry point

## Architecture Decision
Create a NEW `ui` module inside the `game` module: `dlc/city/src/game/ui/`. This keeps the single-crate structure.

## Part 0: Font Infrastructure
Bevy 0.15 uses `TextFont` for text styling. Since this DLC uses DefaultPlugins, Bevy's default font is available. Use `Handle<Font>::default()` or load a system font. The simplest approach: use `default()` for all TextFont handles (Bevy provides a built-in monospace fallback).

Actually — look at what the main Hearthfield game and pilot DLC do for fonts. Then do the same pattern. If they load from an asset path, check if that path exists for the city DLC. If no font asset exists, use `Default::default()` for all Font handles.

## Part 1: Create UI module structure

Create these files:
- `dlc/city/src/game/ui/mod.rs` — UiPlugin, font resource, all pub mod declarations
- `dlc/city/src/game/ui/main_menu.rs` — Main menu screen
- `dlc/city/src/game/ui/hud.rs` — In-game HUD overlay
- `dlc/city/src/game/ui/pause_menu.rs` — Pause screen
- `dlc/city/src/game/ui/day_summary.rs` — End-of-day results screen
- `dlc/city/src/game/ui/task_board.rs` — Task list with progress bars
- `dlc/city/src/game/ui/interruption.rs` — Interruption choice popup

## Part 2: Main Menu Screen (`main_menu.rs`, ~100-120 lines)

Replace the auto-transition `main_menu_to_in_day` with a real menu.

- Title: "CITY OFFICE WORKER" in large text
- Subtitle: "A Hearthfield DLC"
- Buttons: "New Game", "Load Game", "Quit"
- New Game → reset resources + transition to InDay
- Load Game → send LoadSlotRequest(0) + transition to InDay
- Quit → AppExit event
- Systems: spawn_main_menu, despawn_main_menu, handle_main_menu_input

**IMPORTANT:** Remove or replace `main_menu_to_in_day` in game/mod.rs — it currently auto-skips to InDay. The new main menu must wait for player input.

## Part 3: HUD Overlay (`hud.rs`, ~150-180 lines)

Persistent overlay during InDay state showing:
- Top bar: "Day {N} — {HH:MM}" (from DayClock, use format_clock function from resources.rs)
- Left panel stats:
  - Energy: {value}/{max} with color bar (green→red)
  - Stress: {value}/{max} with color bar (low=green, high=red)
  - Focus: {value}/{max} with color bar
  - Money: ${amount}
  - Reputation: {value}
- Right panel: "Inbox: {remaining}/{starting}" items
- Bottom: "Pending interruptions: {N}" (if > 0, highlighted red)
- Key hints bar: "P:Process C:Coffee N:Wait I:Interrupt 1:Calm 2:Panic M:Manager H:Help"

Systems: spawn_hud, despawn_hud, update_hud (runs every frame in InDay)

## Part 4: Pause Menu (`pause_menu.rs`, ~80-100 lines)

When Esc pressed during InDay (already transitions to Paused state):
- Semi-transparent dark overlay
- "PAUSED" title
- Buttons: "Resume", "Save Game", "Quit to Menu"
- Resume → back to InDay
- Save → send SaveSlotRequest(0), show "Saved!" toast, back to InDay
- Quit to Menu → transition to MainMenu

Systems: spawn_pause_menu, despawn_pause_menu, handle_pause_input

## Part 5: Day Summary Screen (`day_summary.rs`, ~120-150 lines)

Replace the auto-transition in DaySummary state with a real screen.

Show DayOutcome resource data:
- "DAY {N} COMPLETE" header
- Tasks completed: {N} / Tasks failed: {N}
- Salary earned: ${amount}
- Reputation change: +{delta} or -{delta}
- Stress change: +{delta} or -{delta}
- Level: {N} (XP: {current}/{threshold})
- If level up occurred: "LEVEL UP! New perk: {perk_name}"
- Button: "Continue to Day {N+1}"

**IMPORTANT:** Modify `transition_day_summary_to_inday` in game/systems/day_cycle.rs — it currently auto-advances. Change it to only transition when the player presses Enter/Space or clicks Continue. OR: add a new system `handle_day_summary_input` that transitions on player input, and remove the auto-transition from `transition_day_summary_to_inday`.

Systems: spawn_day_summary, despawn_day_summary, handle_day_summary_input

## Part 6: Task Board Display (`task_board.rs`, ~100-120 lines)

Shows during InDay state (part of HUD or separate panel):
- "TASK BOARD" header
- For each active task (from TaskBoard.active):
  - Task kind icon/label (DataEntry, Filing, EmailTriage, PermitReview)
  - Priority badge (Low=gray, Medium=yellow, High=orange, Critical=red)
  - Progress bar: {progress * 100}%
  - Deadline: "Due: {HH:MM}" or "OVERDUE" in red
- Show completed count: "{completed_today.len()} done / {failed_today.len()} failed"

Systems: spawn_task_board, despawn_task_board, update_task_board

## Part 7: Interruption UI (`interruption.rs`, ~80-100 lines)

When pending_interruptions > 0, show a popup overlay:
- "INTERRUPTION!" header (red)
- Current scenario description (you can make these up based on the 6 scenario templates)
- Two response buttons: "Stay Calm (1)" and "Panic! (2)"
- Calm → send ResolveCalmlyEvent
- Panic → send PanicResponseEvent
- Show in InDay state when pending_interruptions > 0

Systems: spawn_interruption_popup, despawn_interruption_popup, update_interruption_visibility

## Part 8: Wire everything in game/mod.rs

Add `pub mod ui;` to game/mod.rs.
Register all UI systems in the plugin:
- OnEnter(MainMenu) → spawn_main_menu
- OnExit(MainMenu) → despawn_main_menu
- Update + MainMenu → handle_main_menu_input
- OnEnter(InDay) → spawn_hud, spawn_task_board
- OnExit(InDay) → despawn_hud, despawn_task_board
- Update + InDay → update_hud, update_task_board, update_interruption_visibility
- OnEnter(Paused) → spawn_pause_menu
- OnExit(Paused) → despawn_pause_menu
- Update + Paused → handle_pause_input
- OnEnter(DaySummary) → spawn_day_summary
- OnExit(DaySummary) → despawn_day_summary
- Update + DaySummary → handle_day_summary_input

Remove the old `main_menu_to_in_day` system registration (it auto-skips the menu).
Modify `transition_day_summary_to_inday` to NOT auto-transition (let the UI handle it).

## Part 9: Remove println! statements

In `bootstrap.rs`, the `setup_scene` function has a println! at the end. Remove it — the HUD now shows this info visually.

## Validation
```bash
cd /home/user/hearthfield/dlc/city && cargo check 2>&1
cd /home/user/hearthfield/dlc/city && cargo test 2>&1
cd /home/user/hearthfield/dlc/city && cargo clippy -- -D warnings 2>&1
```
Done = all three pass with zero errors/warnings. Note: city DLC tests use `cargo test` (not `--test headless`).

## When done
Write completion report to `/home/user/hearthfield/status/workers/city-ui-screens.md`
