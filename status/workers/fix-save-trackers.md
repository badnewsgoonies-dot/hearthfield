# Worker Report: FIX-SAVE-TRACKERS

## Files Modified
- `src/npcs/dialogue.rs` — added `Clone, serde::Serialize, serde::Deserialize` to `DailyTalkTracker`
- `src/npcs/map_events.rs` — added `Clone, serde::Serialize, serde::Deserialize` to `GiftDecayTracker`
- `src/npcs/mod.rs` — made `dialogue` and `map_events` modules `pub` so save/mod.rs can reference the types
- `src/save/mod.rs`:
  - Added `daily_talk_tracker` and `gift_decay_tracker` fields to `FullSaveFile` (`#[serde(default)]`)
  - Added both to `ExtendedResources` (read-only refs)
  - Added both to `ExtendedResourcesMut` (mutable refs)
  - Added both params to native `write_save` function signature + body
  - Added both params to WASM `write_save` stub
  - Added both to `handle_save_request` call to `write_save`
  - Added `*ext.daily_talk_tracker = file.daily_talk_tracker` and gift variant to `handle_load_request`
  - Added `*ext.daily_talk_tracker = Default::default()` and gift variant to `handle_new_game`

## What Was Implemented
- `DailyTalkTracker` is now serialized/deserialized on save/load, preventing unlimited daily NPC friendship exploit after reload
- `GiftDecayTracker` is now serialized/deserialized on save/load, preserving consecutive-days-without-gift counters across sessions

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (88 passed, 0 failed)
- `cargo clippy -- -D warnings` — PASS

## Known Risks
- None: `#[serde(default)]` on both new `FullSaveFile` fields ensures backwards compatibility with existing saves
