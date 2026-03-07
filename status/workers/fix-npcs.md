# Worker Report: FIX-NPCS

## Files Modified

| File | Changes |
|------|---------|
| `src/npcs/dialogue.rs` | Added `DailyTalkTracker` resource, modified `handle_npc_interaction` to grant 20 friendship on first daily talk, added `reset_daily_talks` system |
| `src/npcs/mod.rs` | Imported `DailyTalkTracker` and `reset_daily_talks`, registered resource, added system to Update schedule |
| `src/npcs/quests.rs` | Replaced `GiftGivenEvent` with `DialogueStartEvent` in `track_quest_progress` for Talk quest objectives, updated doc comment, fixed talk_description text |

## Bug 1: Daily Talking Does Not Increase Friendship

**Root cause:** `handle_npc_interaction` in `dialogue.rs` sent `DialogueStartEvent` but never called `relationships.add_friendship()`.

**Fix applied:**
1. Added `DailyTalkTracker` resource with a `HashSet<String>` field `talked` to track which NPCs have been talked to today.
2. Changed `relationships: Res<Relationships>` to `mut relationships: ResMut<Relationships>` in `handle_npc_interaction`.
3. Added `mut daily_talks: ResMut<DailyTalkTracker>` parameter.
4. Before sending `DialogueStartEvent`, check if the NPC hasn't been talked to today. If not, call `relationships.add_friendship(npc_id, 20)` (20 points = 1/5 of a heart) and insert into tracker.
5. Added `reset_daily_talks` system that listens to `DayEndEvent` and clears the tracker.
6. Registered the resource and system in `NpcPlugin::build()`.

## Bug 2: Talk Quest Completes on Gift Instead of Dialogue

**Root cause:** `track_quest_progress` in `quests.rs` used `GiftGivenEvent` to complete `QuestObjective::Talk` quests, requiring a gift instead of just talking.

**Fix applied:**
1. Replaced `mut gift_events: EventReader<GiftGivenEvent>` with `mut dialogue_events: EventReader<DialogueStartEvent>`.
2. Replaced the `GiftGivenEvent` iteration block with `DialogueStartEvent` iteration, matching on `event.npc_id`.
3. Updated the doc comment to reflect `DialogueStartEvent -> QuestObjective::Talk`.
4. Updated `talk_description()` text to say "have a chat" / "have a conversation" instead of "give a gift".

## Shared Type Imports Used
- `Relationships` (ResMut) with `add_friendship(npc_id, 20)`
- `DialogueStartEvent` (EventReader in quests, EventWriter in dialogue)
- `DayEndEvent` (EventReader in reset_daily_talks)
- `QuestObjective::Talk` (pattern match)
- No changes to `src/shared/mod.rs`

## Validation Results
- `cargo check` -- PASS (zero errors)
- `cargo clippy -- -D warnings` -- PASS (zero warnings)
