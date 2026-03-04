# Worker: FIX-SAVE-TRACKERS (Persist DailyTalkTracker + GiftDecayTracker)

## Scope
You may modify files under: src/npcs/ AND src/save/

## Required reading (read these files from disk before writing any code)
1. src/npcs/dialogue.rs (find DailyTalkTracker struct — needs Serialize, Deserialize)
2. src/npcs/mod.rs (find GiftDecayTracker init — check if it exists and its definition)
3. src/save/mod.rs (read the FULL file — FullSaveFile struct, ExtendedResources, ExtendedResourcesMut, write_save, handle_load_request, handle_new_game)
4. src/shared/mod.rs — search for "GiftDecayTracker" to check if it's defined there

## Bug: DailyTalkTracker and GiftDecayTracker Not Saved

### Root Cause
`DailyTalkTracker` (src/npcs/dialogue.rs) and `GiftDecayTracker` (if it exists) are initialized on startup but NOT included in FullSaveFile. This means:
- After save/load, the player can talk to NPCs unlimited times per day for friendship
- Gift decay tracking resets

### Fix Required

#### 1. DailyTalkTracker — Add serde derives
In src/npcs/dialogue.rs, change:
```rust
#[derive(Resource, Debug, Default)]
pub struct DailyTalkTracker {
```
to:
```rust
#[derive(Resource, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DailyTalkTracker {
```

#### 2. Add DailyTalkTracker to FullSaveFile
In src/save/mod.rs:
- Add to FullSaveFile struct: `#[serde(default)] pub daily_talk_tracker: crate::npcs::dialogue::DailyTalkTracker,`
- Add to ExtendedResources: `pub daily_talk_tracker: Res<'w, crate::npcs::dialogue::DailyTalkTracker>,`
- Add to ExtendedResourcesMut: `pub daily_talk_tracker: ResMut<'w, crate::npcs::dialogue::DailyTalkTracker>,`
- Add to write_save: `daily_talk_tracker: ext.daily_talk_tracker.clone(),`
- Add to handle_load_request: `*ext.daily_talk_tracker = file.daily_talk_tracker;`
- Add to handle_new_game: `*ext.daily_talk_tracker = Default::default();`

#### 3. GiftDecayTracker (if it exists)
First check if GiftDecayTracker is defined (search for it). If it exists and isn't in FullSaveFile, apply the same pattern as above. If it doesn't exist or is already saved, skip this.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-save-trackers.md
