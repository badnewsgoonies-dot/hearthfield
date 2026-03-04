# Worker: FIX-PILOT-STORY-BOARD (Story Missions on Mission Board)

## Scope
You may modify files under: dlc/pilot/src/

## Required reading
1. dlc/pilot/src/missions/board.rs (FULL file — refresh_mission_board, MissionBoard)
2. dlc/pilot/src/missions/story.rs (FULL file — StoryProgress, all_story_missions(), StoryMission, track_story_progress)
3. dlc/pilot/src/shared/mod.rs (search for Mission struct, MissionBoard, MissionStatus)
4. dlc/pilot/src/missions/mod.rs (see which systems are registered)

## Bug: Story Missions Never Appear on Mission Board

### Root Cause
`all_story_missions()` returns 27 story missions across 10 chapters. `track_story_progress` tracks completion. `show_story_mission_on_board` fires `ToastEvent` notifications.

BUT story missions are never actually injected into `MissionBoard.available`. The `refresh_mission_board` in `board.rs` only generates procedural missions. Players can never select or accept story missions.

### Fix Required

1. **In `board.rs` — modify `refresh_mission_board`** to inject the current story mission:
   - Add `story_progress: Res<StoryProgress>` parameter
   - After generating procedural missions, check StoryProgress for current chapter
   - Get the next uncompleted story mission from `all_story_missions()`
   - Convert the StoryMission to a Mission struct and insert it at position 0 (top of board)
   - Mark it visually distinct (maybe prefix title with "★ STORY: " or set a flag)

2. **Ensure the Mission struct can represent story missions:**
   - Check if Mission has an `is_story` or `mission_type` field
   - If not, add `#[serde(default)] pub is_story: bool` to Mission in shared/mod.rs
   - OR just use a naming convention like prefixing with "★ "

3. **In `story.rs` — ensure `track_story_progress`** advances after story mission completion:
   - It should already handle `MissionCompletedEvent` — verify it correctly advances `current_mission_index` and `current_chapter`

4. **In `missions/mod.rs`** — verify `track_story_progress` and `show_story_mission_on_board` are registered as systems. If not, add them.

### Important Notes
- Story missions should always appear at the top of the board
- Only ONE story mission should appear at a time (the current one)
- If the story is complete (story_finished == true), don't inject any
- Procedural missions should still appear below the story mission
- Do NOT modify dlc/pilot/src/shared/mod.rs unless absolutely necessary. Prefer adding the is_story field to the Mission struct if it exists there, or use a naming convention instead.

## Validation
```
cd dlc/pilot && cargo check && cargo test --test headless && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-pilot-story-board.md
