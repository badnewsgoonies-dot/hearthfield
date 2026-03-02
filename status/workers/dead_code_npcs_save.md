# dead_code audit: npcs/save batch

Date: 2026-03-02

## Scope
Audited `#[allow(dead_code)]` annotations in:
- `src/npcs/dialogue.rs`
- `src/npcs/schedule.rs`
- `src/npcs/spawning.rs`
- `src/npcs/emotes.rs`
- `src/save/mod.rs`
- `src/animals/mod.rs`
- `src/crafting/unlock.rs`
- `src/farming/mod.rs`
- `src/mining/components.rs`
- `src/mining/floor_gen.rs`

## Results

| File | Item | grep check (`grep -rn 'item_name' src/ --include='*.rs'`) | Decision | Action |
|---|---|---|---|---|
| `src/npcs/dialogue.rs` | `NpcInteractable` | only `src/npcs/dialogue.rs:10` | not used outside definition | kept `#[allow(dead_code)]` |
| `src/npcs/schedule.rs` | `ScheduleUpdateTimer` | found in `src/npcs/mod.rs` (`.init_resource::<ScheduleUpdateTimer>()`, import) | used outside definition | removed `#[allow(dead_code)]` |
| `src/npcs/schedule.rs` | `tick_schedule_timer` | only `src/npcs/schedule.rs:148` | not used outside definition | kept `#[allow(dead_code)]` |
| `src/npcs/spawning.rs` | `despawn_npcs_for_map` | only `src/npcs/spawning.rs:194` | not used outside definition | kept `#[allow(dead_code)]` |
| `src/npcs/emotes.rs` | `EmoteKind` | found in `src/npcs/gifts.rs` (`use ... EmoteKind`, `EmoteKind::from`) | used outside definition | removed `#[allow(dead_code)]` |
| `src/save/mod.rs` | `SAVE_VERSION` | found in same file at save/load logic (`version: SAVE_VERSION`, comparisons) | used outside definition | removed `#[allow(dead_code)]` |
| `src/save/mod.rs` | `current_timestamp` (`not wasm32`) | used in same file (`save_timestamp: current_timestamp()`) | used outside definition | removed `#[allow(dead_code)]` |
| `src/save/mod.rs` | `current_timestamp` (`wasm32`) | symbol has call site in same module (`current_timestamp()`) and cfg variant selected per target | used outside definition | removed `#[allow(dead_code)]` |
| `src/animals/mod.rs` | `FeedTrough` | found in `src/animals/feeding.rs` and `src/animals/spawning.rs` | used outside definition | removed `#[allow(dead_code)]` |
| `src/crafting/unlock.rs` | `_use_constants` | only `src/crafting/unlock.rs:220` | not used outside definition | kept `#[allow(dead_code)]` |
| `src/farming/mod.rs` | `FarmObjectEntity` | found in `src/farming/render.rs` (import + spawned components) | used outside definition | removed `#[allow(dead_code)]` |
| `src/mining/components.rs` | `ActiveFloor` | found across mining modules (`combat`, `hud`, `ladder`, `movement`, `spawning`, `transitions`, `mod`) | used outside definition | removed `#[allow(dead_code)]` |
| `src/mining/floor_gen.rs` | `FloorBlueprint` | found in `src/mining/spawning.rs` (import and fn params) | used outside definition | removed `#[allow(dead_code)]` |

## Final annotation state in audited files
Remaining `#[allow(dead_code)]` in scope:
- `src/npcs/dialogue.rs` (`NpcInteractable`)
- `src/npcs/schedule.rs` (`tick_schedule_timer`)
- `src/npcs/spawning.rs` (`despawn_npcs_for_map`)
- `src/crafting/unlock.rs` (`_use_constants`)

All other audited `#[allow(dead_code)]` annotations were removed.
