# Quest & Dialogue Bug Fixes — Status

All 4 fixes applied. `cargo check` passes with no errors.

## Changes Made

### src/npcs/quests.rs

**Bug 1 — `track_monster_slain` missing completion logic (CRITICAL)**
- Added `mut completed_writer: EventWriter<QuestCompletedEvent>` parameter.
- Added `newly_completed` collection, deduplication, and `completed_writer.send(...)` loop — matching the exact pattern used in `track_quest_progress`.
- Quest removal from `quest_log.active` is handled by the existing `handle_quest_completed` system (same as all other quest types).

**Bug 4 — `TALK_NPCS` stale placeholder IDs**
- Replaced all 10 fake IDs (`mayor_thomas`, `marcus`, `dr_iris`, `old_pete`, `chef_rosa`, `miner_gil`, `librarian_faye`, `farmer_dale`, `child_lily`) with the real canonical IDs from `src/data/npcs.rs`:
  `margaret`, `marco`, `lily`, `old_tom`, `elena`, `mira`, `doc`, `mayor_rex`, `sam`, `nora`.

### src/npcs/dialogue.rs

**Bug 2 — Birthday line addresses NPC by their own name**
- Changed `format!("Oh! Today is my birthday! I can't believe you remembered, {}!", npc_def.name)` to `"Oh! Today is my birthday! I can't believe you remembered!".to_string()`.

**Bug 3 — Missing seasonal comments in `npc_season_comment`**
Added 5 new match arms:
- `("lily", Season::Fall)` — leaves and playing outside
- `("doc", Season::Summer)` — summer health / hydration tips
- `("margaret", Season::Summer)` — summer berry baking
- `("old_tom", Season::Spring)` — spring fishing
- `("elena", Season::Fall)` — fall tool sharpening and autumn forge light
