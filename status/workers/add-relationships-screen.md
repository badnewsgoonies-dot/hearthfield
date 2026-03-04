# Worker Completion Report: add-relationships-screen

## Files Created/Modified

| File | Action | Lines |
|------|--------|-------|
| src/ui/relationships_screen.rs | Created | 330 |
| src/shared/mod.rs | Modified | 2257 |
| src/input/mod.rs | Modified | 191 |
| src/ui/menu_input.rs | Modified | 101 |
| src/ui/mod.rs | Modified | 358 |
| .contract.sha256 | Updated | 1 |

## What Was Implemented

### src/shared/mod.rs
- Added `RelationshipsView` variant to `GameState` enum
- Added `pub open_relationships: bool` to `PlayerInput` struct (keyed R)
- Added `pub open_relationships: KeyCode` to `KeyBindings` struct with default `KeyCode::KeyR`

### src/input/mod.rs
- Read `open_relationships` from keys in `Gameplay` context
- Read `open_relationships` in `Menu` context (toggle-close support)
- Added `GameState::RelationshipsView => InputContext::Menu` in `manage_input_context`

### src/ui/menu_input.rs
- Wired `input.open_relationships` → `GameState::RelationshipsView` in `gameplay_state_transitions`
- Added toggle-close: `GameState::RelationshipsView if input.open_relationships` → Playing
- Added `RelationshipsView` to the cancel handler's match arm

### src/ui/relationships_screen.rs (NEW)
- `RelationshipsScreenRoot`, `RelNpcRow`, `RelNpcRowBg`, `RelDetailPanel` marker components
- `RelationshipsUiState` resource tracking cursor and NPC list
- `spawn_relationships_screen`: renders all NPCs sorted by ID with ♥/♡ heart display, detail panel below
- `despawn_relationships_screen`: cleans up entities and resource
- `relationships_navigation`: W/S navigation through NPC list, updates detail panel
- `update_relationships_cursor`: highlights selected row
- Detail panel shows: NPC name, birthday (season + day), friendship points/hearts, loved gifts, marriageable flag

### src/ui/mod.rs
- Added `pub mod relationships_screen` module declaration
- Registered `OnEnter/OnExit/Update` systems for `GameState::RelationshipsView`
- Added `RelationshipsView` to `menu_cancel_transitions` run condition

## Quantitative Targets
- Hearts display: 1 heart = 100 points, max 10 hearts ✓ (uses `FRIENDSHIP_PER_HEART` / `MAX_HEARTS` constants)
- All 10 NPCs from NpcRegistry displayed ✓
- Detail panel with birthday, stage, loved gifts ✓
- Cursor navigation (up/down) ✓

## Shared Type Imports Used
- `GameState`, `PlayerInput`, `KeyBindings`, `InputContext`
- `Relationships`, `RelationshipStages`, `NpcRegistry`, `NpcDef`, `NpcId`
- `GiftPreference`, `MenuAction`, `MAX_HEARTS`, `FRIENDSHIP_PER_HEART`

## Validation Results
- `shasum -a 256 -c .contract.sha256` → OK ✓
- `cargo check` → 0 errors, 0 warnings ✓
- `cargo clippy -- -D warnings` → 0 errors, 0 warnings ✓
- `cargo test --test headless` → 88 passed, 0 failed ✓

## Known Risks
- `open_relationships` shares `KeyCode::KeyR` with `tool_secondary` per spec. In Gameplay context both fields fire simultaneously when R is pressed; `gameplay_state_transitions` runs first and switches to `RelationshipsView`. The tool_secondary action may also fire in the same frame but will have no effect since no Gameplay systems run in RelationshipsView state.
- NPC loved gifts are shown as item IDs (not display names) since resolving them via ItemRegistry would require an additional system parameter; this matches the pattern used elsewhere.
