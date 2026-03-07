# Worker Report: Missing UI Screens (Calendar, Stats, Settings)

## Files Created
- `src/ui/calendar_screen.rs` (~230 lines) — Calendar overlay with 28-day grid
- `src/ui/stats_screen.rs` (~175 lines) — Statistics overlay with two-column layout
- `src/ui/settings_screen.rs` (~270 lines) — Settings overlay with volume control and keybinds

## Files Modified
- `src/ui/mod.rs` — Added module declarations and system registrations for all 3 screens

## Implementation Approach
Since `GameState` has no Calendar/Stats/Settings variants and `src/shared/mod.rs` is frozen/checksummed, all 3 screens use the **overlay/toggle pattern** (same as debug_overlay and chest_screen):
- Each screen has a `*OverlayState` resource with a `visible: bool` field
- Toggle via function keys: F1=Calendar, F2=Stats, F4=Settings (F3 is debug overlay)
- Escape also closes any open overlay
- Screens spawn/despawn reactively during `GameState::Playing`
- `GlobalZIndex(60)` ensures overlays render above game world

## Features
### Calendar Screen (F1)
- Season name + year header
- Day-of-week header row (Mon-Sun)
- 4x7 grid for 28-day season
- Current day highlighted in green
- Festival days marked with asterisks and orange highlight
- NPC birthdays shown with blue labels
- Festival names: Egg Fest (Spring 13), Luau (Summer 11), Harvest (Fall 16), W.Star (Winter 25)

### Statistics Screen (F2)
- 10 stat rows from PlayStats resource (crops_harvested through festivals_attended)
- Two-column label|value layout
- Auto-refreshes when PlayStats changes while overlay is open

### Settings Screen (F4)
- Volume control with Left/Right arrow keys (0-100 in steps of 10)
- Visual volume bar [|||||-----] format
- Read-only keybinds display showing all 12 key bindings from KeyBindings resource
- AudioVolume resource persists across open/close cycles

## Shared Type Imports Used
- `GameState`, `Calendar`, `Season`, `PlayStats`, `KeyBindings`, `NpcRegistry`, `NpcDef`
- `UiFontHandle` from `super`

## Validation Results
- `cargo check` — PASS
- `cargo clippy -- -D warnings` — PASS (0 warnings)
- `cargo test --test headless` — PASS (88 passed, 0 failed, 2 ignored)
- `shasum -a 256 -c .contract.sha256` — PASS (src/shared/mod.rs: OK)

## Known Risks
- None. Overlays are self-contained and don't interact with game state transitions.
