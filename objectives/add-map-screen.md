# Worker: ADD-MAP-SCREEN (Full Map View)

## Scope
You may modify files under: src/ui/, src/input/, src/shared/

## CRITICAL: GameState change required
You MUST add `MapView,` to the `GameState` enum in `src/shared/mod.rs`.
After edits, re-run `shasum -a 256 src/shared/mod.rs` and update `.contract.sha256`.

## Required reading (read these files from disk before writing any code)
1. src/shared/mod.rs — search for GameState, MapId, PlayerInput, KeyBindings. Also search for `open_map`.
2. src/ui/inventory_screen.rs — the screen pattern to follow
3. src/ui/journal_screen.rs — another example of recently added screen
4. src/ui/menu_input.rs — key press → state transition wiring
5. src/world/maps.rs — find all MapId variants and map dimensions

## Deliverables

### 1. src/shared/mod.rs
- Add `MapView,` to GameState enum
- Update .contract.sha256 after edit

### 2. src/input/mod.rs
- Add `GameState::MapView => InputContext::Menu` to manage_input_context
- Ensure `open_map` key is forwarded in Menu context for toggle-close

### 3. src/ui/menu_input.rs
- Wire M key → GameState::MapView in gameplay_state_transitions
- Add toggle-close (M again returns to Playing)
- Add MapView to cancel/Esc handling

### 4. src/ui/map_screen.rs — NEW FILE (~200-300 lines)
Create a full world map screen showing all locations:

**Layout:**
```
┌─────────────────────────────────────────┐
│              HEARTHFIELD MAP            │
├─────────────────────────────────────────┤
│                                         │
│   [Forest]     [Mine]                   │
│       \          |                      │
│   [Farm] ---- [Town]                    │
│       |          |                      │
│   [Beach]    [General Store]            │
│              [Animal Shop]              │
│              [Blacksmith]               │
│                                         │
│   ★ You are here: Farm                  │
│                                         │
│   Press M or Esc to close               │
└─────────────────────────────────────────┘
```

- Display all map locations as labeled boxes/nodes
- Highlight the player's current map location with a marker (★)
- Read PlayerState.current_map or equivalent to determine player location
- Use styled text nodes (like inventory/journal screens)
- Simple static layout — no interactive navigation needed
- Show connection lines between locations using text characters (|, -, \, /)

### 5. src/ui/mod.rs — Register systems

## Validation
```
shasum -a 256 -c .contract.sha256
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/add-map-screen.md
