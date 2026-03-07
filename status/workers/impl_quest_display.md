# Quest Display Fix — Implementation Notes

## Changes (src/npcs/quests.rs only)

### Issue 1: Quest titles used raw NPC IDs
After picking `giver` (an NPC ID key), added a `giver_display` lookup:
```rust
let giver_display = npc_registry
    .npcs
    .get(&giver)
    .map(|d| d.name.clone())
    .unwrap_or_else(|| giver.clone());
```
All 5 quest-type title format strings updated to use `giver_display` instead of `giver`.

For the talk quest, added a parallel `target_npc_display` lookup and used it in the title:
```rust
title: format!("{}: Visit {} for {}", talk_tmpl.0, target_npc_display, giver_display)
```
`target_npc` (the ID) is still stored in `QuestObjective::Talk { npc_name }` for matching.

### Issue 2: `child_lily` fallback
Replaced:
```rust
.unwrap_or_else(|| "child_lily".to_string())
```
With:
```rust
.unwrap_or_else(|| TALK_NPCS[0].to_string())  // "margaret"
```

## Verification
- `cargo check` passes with no errors.
