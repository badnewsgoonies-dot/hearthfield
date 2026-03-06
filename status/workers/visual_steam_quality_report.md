# Visual Steam-Quality Pass — Completion Report

**Date:** 2026-03-06
**Branch:** `claude/multi-agent-orchestration-vSy7F`
**Commits:** 4 (one per phase)

## Summary

All 5 phases of the visual polish plan executed successfully. All gates pass (cargo check, cargo test --test headless, cargo clippy -- -D warnings). Contract `src/shared/mod.rs` verified intact.

## Phase 1: Player Tool Animation (commit 3b1fc01)
**File:** `src/player/tool_anim.rs`
- **C4 fix:** Tool animations now set `sprite.flip_x = true` when player faces left, giving directional awareness
- **M7 fix:** Tool animation completion checks `movement.is_moving` and transitions to `Walk` instead of `Idle` to prevent 1-frame flicker

## Phase 2: World Visual Polish (commit 0105cd5)
**Files:** `src/world/mod.rs`, `src/world/objects.rs`
- **H5 fix:** Added `YSorted` + `LogicalPosition` components to bed and stove interactables so they participate in y-sorting
- **H1 fix:** Added `WaterTile` marker component, `WaterAnimationTimer` resource, and `animate_water_tiles` system that cycles water tiles through 4 atlas frames at 0.5s intervals
- **H2 fix:** Grass tiles now use positional hash `(x*7 + y*13) % 4` to pick from 4 variants per season, breaking the monotone grid pattern

## Phase 3: NPC Polish (commit 71ed565)
**Files:** `src/npcs/dialogue.rs`, `src/ui/dialogue_box.rs`
- **H3 fix:** When player initiates dialogue, NPC movement target is set to player position, causing the animation system to face the NPC toward the player
- **C2 fix:** Dialogue portraits now always use atlas index 0 (front-facing idle pose) instead of arbitrary walk-frame indices

## Phase 4: Forageable & Tile Fixes (commit 583bc9f)
**Files:** `src/world/objects.rs`, `src/world/mod.rs`
- **M3 fix:** Winter forageables given unique atlas indices (snow_yam=18, winter_root=19, crocus=20, crystal_fruit=21), eliminating 4 duplicates
- **M1 fix:** Stone tiles now use `hills.png` atlas (index 0) instead of tilled_dirt. Wood floor uses tilled_dirt index 6 for a better plank appearance.

## Phase 5: Final Verification
- `cargo check`: PASS
- `cargo test --test headless`: 88 passed, 0 failed, 2 ignored
- `cargo clippy -- -D warnings`: PASS (zero warnings)
- Contract checksum: VERIFIED (src/shared/mod.rs: OK)
- Push: up-to-date on remote

## Bugs Fixed (by audit ID)
| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| C4 | Critical | Tool animations ignore facing direction | FIXED |
| C2 | Critical | Portrait system shows walk frames | FIXED |
| H1 | High | Water completely static | FIXED |
| H2 | High | Grass monotone | FIXED |
| H3 | High | NPCs don't face player on talk | FIXED |
| H5 | High | Bed/stove always on top | FIXED |
| M1 | Medium | Stone/wood floor wrong textures | FIXED |
| M3 | Medium | Duplicate forageable indices | FIXED |
| M7 | Medium | 1-frame flicker on tool end | FIXED |

## Not Addressed (out of scope for this pass)
- C1: Fish sprite indices out of bounds (no fish rendering exists yet)
- C3: Animals are colored rectangles (requires new sprite assets)
- H4: No animal animation system (requires new sprite assets)
- H6: Emote atlas indices are guesses (needs asset verification)
- M2: Path/fence autotile bitmask assumption (needs asset verification)
- M4: Crop growth has no animation
- M5: Season change strips custom_size (latent, harmless at current TILE_SIZE)
- M6: No water edge transitions (significant new feature)
