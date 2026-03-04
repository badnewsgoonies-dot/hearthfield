# Worker: ADD-JOURNAL-SCREEN (Quest Log UI)

## Scope
You may modify files under: src/ui/, src/input/, src/shared/, src/ui/

## CRITICAL: This is an integration phase — GameState change required
You MUST add `Journal,` to the `GameState` enum in `src/shared/mod.rs`.
After your edit, re-run `shasum -a 256 src/shared/mod.rs` and update `.contract.sha256` with the new hash.

## Required reading (read these files from disk before writing any code)
1. src/shared/mod.rs — search for "GameState" enum (lines ~14-30). Also search for QuestLog, Quest, QuestObjective structs.
2. src/ui/mod.rs — FULL file. Understand how screens are registered (OnEnter, OnExit, Update systems).
3. src/ui/inventory_screen.rs — FULL file. This is the PATTERN to follow for the journal screen.
4. src/ui/menu_input.rs — FULL file. Understand how key presses transition GameState.
5. src/input/mod.rs — find open_journal key binding and manage_input_context function.
6. src/npcs/quests.rs — lines 1-50. Understand Quest structure.

## Deliverables

### 1. src/shared/mod.rs — Add GameState::Journal
Add `Journal,` variant to the GameState enum. Then update .contract.sha256:
```bash
shasum -a 256 src/shared/mod.rs > .contract.sha256
```

### 2. src/input/mod.rs — Map Journal → InputContext::Menu
In `manage_input_context()`, add `GameState::Journal => InputContext::Menu,` to the match.

### 3. src/ui/menu_input.rs — Wire J key
In `gameplay_state_transitions()`:
- Add: `if input.open_journal { next.set(GameState::Journal); }`

In `menu_cancel_transitions()`:
- Add `GameState::Journal` to the cancel match arm that returns to Playing
- Add toggle-close: if already in Journal and J pressed again, return to Playing

### 4. src/ui/journal_screen.rs — NEW FILE (~300-400 lines)
Create a Quest Log screen following the inventory_screen.rs pattern:

**Marker components:**
- `JournalScreenRoot` — root UI node
- `QuestListItem { index: usize }` — per-quest row
- `QuestDetailPanel` — detail view at bottom

**Resource:**
```rust
#[derive(Resource)]
pub struct JournalUiState {
    pub cursor: usize,
    pub quest_ids: Vec<String>,  // IDs of quests in current view
}
```

**Systems:**
- `spawn_journal_screen` — OnEnter(Journal): build full UI showing active quests from QuestLog
- `despawn_journal_screen` — OnExit(Journal): despawn all, remove resource
- `update_quest_display` — Update system: re-render quest list from QuestLog
- `update_cursor_highlight` — Update system: highlight selected quest
- `journal_navigation` — Update system: handle MenuAction (up/down/cancel)

**Layout:**
```
┌─────────────────────────────────────┐
│          QUEST LOG                  │
├─────────────────────────────────────┤
│ > Quest Title 1                     │  ← highlighted
│   From: NPC Name                    │
│   Progress: 3/5 items               │
│                                     │
│   Quest Title 2                     │
│   From: NPC Name                    │
│   Status: Complete ✓                │
├─────────────────────────────────────┤
│ DETAILS                             │
│ [Full description of selected quest]│
│ Reward: 120g                        │
│ Days remaining: 3                   │
└─────────────────────────────────────┘
```

Use the exact same UI styling as inventory_screen.rs (fonts, colors, node structure, BackgroundColor values).

**Quest progress display per QuestObjective:**
- Deliver: "Deliver: {delivered}/{quantity} {item_id}"
- Catch: "Catch: {fish_id}" + checkmark if delivered
- Harvest: "Harvest: {harvested}/{quantity} {crop_id}"
- Mine: "Mine: {collected}/{quantity} {item_id}"
- Talk: "Talk to {npc_name}" + checkmark if talked
- Slay: "Slay: {slain}/{quantity} {monster_kind}"

### 5. src/ui/mod.rs — Register journal systems
Add `pub mod journal_screen;` and register OnEnter/OnExit/Update systems using the exact inventory pattern.

## Validation
```
shasum -a 256 -c .contract.sha256   # Updated contract must pass
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/add-journal-screen.md
