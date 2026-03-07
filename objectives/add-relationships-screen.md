# Worker: ADD-RELATIONSHIPS-SCREEN

## Scope
You may modify files under: src/ui/, src/input/, src/shared/

## CRITICAL: GameState change required
You MUST add `RelationshipsView,` to the `GameState` enum in `src/shared/mod.rs`.
Also add `open_relationships: bool` to `PlayerInput` and `open_relationships: KeyCode::KeyR` to `KeyBindings`.
After edits, re-run `shasum -a 256 src/shared/mod.rs` and update `.contract.sha256`.

## Required reading (read these files from disk before writing any code)
1. src/shared/mod.rs — search for GameState, Relationships, RelationshipStages, PlayerInput, KeyBindings
2. src/ui/inventory_screen.rs — the pattern to follow
3. src/ui/menu_input.rs — key press → state transition wiring
4. src/data/npcs.rs — NPC definitions (names, portraits)

## Deliverables

### 1. src/shared/mod.rs changes
- Add `RelationshipsView,` to GameState enum
- Add `pub open_relationships: bool` to PlayerInput struct
- Add `pub open_relationships: KeyCode` to KeyBindings with default `KeyCode::KeyR`

### 2. src/input/mod.rs
- Read open_relationships from keys in update_player_input
- Add RelationshipsView to manage_input_context → InputContext::Menu

### 3. src/ui/menu_input.rs
- Wire R key → GameState::RelationshipsView in gameplay_state_transitions
- Add toggle-close and cancel handling

### 4. src/ui/relationships_screen.rs — NEW FILE (~200-250 lines)
Show all NPCs with their current friendship hearts:

**Layout:**
```
┌─────────────────────────────────────┐
│        RELATIONSHIPS                │
├─────────────────────────────────────┤
│  Lily          ♥♥♥♥♥♡♡♡♡♡  5/10   │
│  Elena         ♥♥♥♡♡♡♡♡♡♡  3/10   │
│  Margaret      ♥♥♡♡♡♡♡♡♡♡  2/10   │
│  Marco         ♥♡♡♡♡♡♡♡♡♡  1/10   │
│  Old Tom       ♡♡♡♡♡♡♡♡♡♡  0/10   │
│  ...                                │
├─────────────────────────────────────┤
│ SELECTED: Lily                      │
│ Birthday: Spring 15                 │
│ Loves: Flowers, Honey               │
│ Status: Friend                      │
└─────────────────────────────────────┘
```

- Read Relationships resource for friendship points per NPC
- Calculate hearts: 1 heart = 100 friendship points, max 10 hearts
- Show filled/empty hearts using Unicode or styled nodes
- Cursor navigation (up/down) to select NPC for detail view
- Detail panel shows birthday, stage, and loved gifts

### 5. src/ui/mod.rs — Register systems

## Validation
```
shasum -a 256 -c .contract.sha256
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/add-relationships-screen.md
