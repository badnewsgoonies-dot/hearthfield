# Worker Report: FIX-PILOT-STORY-BOARD

## Files Modified
- `dlc/pilot/src/missions/board.rs` — added `StoryProgress` import and story mission injection logic (~28 lines added)

## What Was Implemented

Modified `refresh_mission_board` in `board.rs` to:
1. Added `story_progress: Res<StoryProgress>` parameter
2. Added `use super::story::StoryProgress;` import
3. After generating procedural missions, checks `StoryProgress` for the current chapter's next uncompleted mission
4. Converts the `StoryMission` to a `MissionDef` and inserts it at position 0 (top of board) with title prefixed `"★ STORY: "`
5. Guards: skips injection if `story_finished == true`, if player rank is too low, or if the mission is already in `completed_ids`

Used naming convention (`"★ STORY: "` prefix) instead of modifying `shared/mod.rs` — no shared contract changes were needed.

`track_story_progress` and `show_story_mission_on_board` were already registered in `missions/mod.rs`. No changes needed there.

## Quantitative Targets Hit
- 27 story missions across 10 chapters: served one at a time from position 0 on the board
- Procedural missions remain below the story mission (inserted at index 0, procedural fill from index 1+)

## Shared Type Imports Used
- `MissionDef`, `MissionType`, `MissionDifficulty`, `MissionBoard`, `PilotRank`, `AirportId` (all from `crate::shared::*`)
- `StoryProgress` from `super::story`

## Validation Results
- `cargo check` — ✅ PASS
- `cargo clippy -- -D warnings` — ✅ PASS (0 warnings)
- `cargo test --test headless` — ✅ PASS (76 passed, 0 failed)

## Known Risks for Integration
- Story missions use `MissionType::Charter` as a placeholder type. If UI renders mission type labels, story missions will show "Private Charter". If a dedicated `Story` variant is desired later, it would require adding to `MissionType` in `shared/mod.rs`.
- `completed_ids` check prevents story missions from re-appearing once accepted+completed, assuming `tracking::handle_mission_complete` pushes completed mission IDs to `MissionBoard.completed_ids`.
