# Worker: FIX-NPCS

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/npcs/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/npcs/mod.rs (read the FULL file — understand plugin structure)
2. src/npcs/dialogue.rs (read the FULL file — this is where Bug 1 lives)
3. src/npcs/quests.rs (read the FULL file — this is where Bug 2 lives)
4. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Bug 1: Daily Talking Does Not Increase Friendship

### Root Cause
`handle_npc_interaction` in `src/npcs/dialogue.rs` detects the player pressing F near an NPC, builds dialogue lines, and sends `DialogueStartEvent`. But it NEVER calls `relationships.add_friendship(npc_id, amount)`. Talking to NPCs gives zero friendship — a core gameplay loop is broken.

### Fix Required
1. Add a new resource `DailyTalkTracker` with a `talked: HashSet<String>` field (in dialogue.rs). This tracks which NPCs the player has talked to today. Do NOT modify `src/shared/mod.rs`.
2. Register this resource in `NpcPlugin::build()` in `src/npcs/mod.rs` via `app.init_resource::<DailyTalkTracker>()`.
3. In `handle_npc_interaction`:
   - Change `relationships: Res<Relationships>` to `mut relationships: ResMut<Relationships>`
   - Add `mut daily_talks: ResMut<DailyTalkTracker>` parameter
   - Before sending DialogueStartEvent, check `if !daily_talks.talked.contains(npc_id)`, and if so call `relationships.add_friendship(npc_id, 20)` (20 points = 1/5 of a heart) and insert into `daily_talks.talked`
4. Add a `pub fn reset_daily_talks` system that reads `EventReader<DayEndEvent>` and clears `daily_talks.talked`. Register it in the Update schedule (run_if Playing).
5. Import `DailyTalkTracker` in mod.rs and register the resource + system.

Note: `build_dialogue_lines` takes `&Relationships` (shared ref). Since you now need `ResMut<Relationships>` for the mutation, you will need to reborrow or use the deref: pass `&relationships` to `build_dialogue_lines`.

## Bug 2: Talk Quest Completes on Gift Instead of Dialogue

### Root Cause
`track_quest_progress` in `src/npcs/quests.rs:650-664` uses `GiftGivenEvent` to complete `QuestObjective::Talk` quests. This means the player must gift an item to satisfy a "talk to NPC" quest, rather than just talking.

### Fix Required
In `track_quest_progress`:
1. Add `mut dialogue_events: EventReader<DialogueStartEvent>` parameter
2. Replace the GiftGivenEvent block (lines ~650-664) with a DialogueStartEvent block:
   ```rust
   for event in dialogue_events.read() {
       for quest in quest_log.active.iter_mut() {
           if let QuestObjective::Talk { ref npc_name, ref mut talked } = quest.objective {
               if *npc_name == event.npc_id && !*talked {
                   *talked = true;
                   newly_completed.push((quest.id.clone(), quest.reward_gold));
               }
           }
       }
   }
   ```
3. Remove the now-unused `mut gift_events: EventReader<GiftGivenEvent>` parameter (or keep it if other quest types use it — check first).
4. Update the doc comment at top of function to say `DialogueStartEvent -> QuestObjective::Talk`.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-npcs.md containing:
- Files modified (with line counts)
- What was changed and why
- Validation results (pass/fail)
