# Visual & Polish Orchestrator — Steam-Quality Pass

You are an ORCHESTRATOR sub-agent. Your job is to dispatch workers via `claude -p` to fix visual bugs and polish the game to Steam-worthy quality. You do NOT write Rust code yourself — you dispatch workers who do.

## Context
Hearthfield is a Harvest Moon-style farming sim in Rust/Bevy 0.15. A comprehensive visual audit has been completed at `status/workers/visual_audit_report.md`. Read it first.

## Your Workflow (repeat for each phase)
1. Read the relevant source files to understand current state
2. Write a focused worker objective to `/tmp/worker_objective.md`
3. Dispatch the worker: `claude -p "$(cat /tmp/worker_objective.md)" --allowedTools "Read,Edit,Write,Bash,Grep,Glob" --max-turns 45 --output-format json --cwd /home/user/hearthfield 2>&1 | tee status/workers/{phase_name}.json`
4. After worker completes, run gates: `cargo check 2>&1 && cargo test --test headless 2>&1 && cargo clippy -- -D warnings 2>&1`
5. If gates fail, dispatch a fix worker (max 3 retries per phase)
6. Commit: `git add -A && git commit -m "fix: {description}"`
7. Move to next phase

## PHASES (execute in order)

### Phase 1: Quick Structural Fixes (player domain)
**Files:** `src/player/tool_anim.rs`, `src/player/movement.rs`

**Fix C4: Direction-aware tool animations**
The `action_atlas_index()` function at tool_anim.rs:36 ignores facing direction. Currently: `tool_offset + frame`. The `character_actions.png` atlas is 2 cols × 12 rows = 24 frames. Each tool has 4 frames but these are NOT directional — the atlas only has one direction per tool. So the fix is: when facing LEFT, set `sprite.flip_x = true`. When facing RIGHT, set `sprite.flip_x = false`. When facing UP vs DOWN, use the existing frames (they show a downward swing which works for both in pixel art style). The tool animation system at line 81 has access to `movement.facing` — use it to set `sprite.flip_x` when entering and during the animation.

**Fix M7: 1-frame idle flicker after tool animation while moving**
At tool_anim.rs:128, animation completion forces `PlayerAnimState::Idle`. If the player is moving, the next frame sets Walk, causing a 1-frame flicker. Fix: check if `movement.is_moving` and set `Walk` instead of `Idle` when true.

### Phase 2: Quick Structural Fixes (world domain)
**Files:** `src/world/objects.rs`, `src/world/mod.rs`

**Fix H5: Bed and Stove missing y-sort**
In `src/world/objects.rs`, the bed and stove entities are spawned with hardcoded `z=100.1` but WITHOUT `YSorted` and `LogicalPosition` components. Add `YSorted` and `LogicalPosition(Vec2::new(wc.x, wc.y))` to their spawn bundles so they participate in y-sorting like all other furniture. Search for "bed" and "stove" in spawn_interior_decorations or the furniture spawning code to find the exact locations.

**Fix H1: Water animation**
In `src/world/mod.rs`, water tiles use `water.png` which has 4 animation frames but only index 0 is rendered. Add a simple water animation system:
- Create a `WaterAnimationTimer` resource with a `Timer::from_seconds(0.5, TimerMode::Repeating)`
- Each tick, advance all water tile sprites to `(current_index + 1) % 4`
- Query for entities that have a `TextureAtlas` using the water atlas layout and cycle them
- Register the resource and system in WorldPlugin

**Fix H2: Grass tile variation**
In `src/world/mod.rs`, the `tile_atlas_info` function returns a single grass index per season. The `grass.png` tileset has 77 frames (11×7). Use a deterministic hash of `(x, y)` grid position to pick from multiple grass variants per season. For example: `let variant = ((x * 7 + y * 13) as usize) % 4; season_base + variant` where season_base offsets into different rows. This gives visual variety without any runtime cost.

### Phase 3: NPC Polish
**Files:** `src/npcs/animation.rs`, `src/npcs/spawning.rs`, `src/ui/dialogue_box.rs`

**Fix H3: NPCs face player during conversation**
When the player presses F to talk to an NPC, the NPC should turn to face the player. In the dialogue initiation code (likely in `src/npcs/dialogue.rs` or wherever `DialogueOpenEvent` is handled), after identifying the target NPC, compute the direction from NPC to player and set `NpcMovement.facing` (or equivalent) so the NPC's walk animation frame updates to face the player. The NPC animation system in `src/npcs/animation.rs` already handles facing → atlas index mapping.

**Fix C2: Portrait system**
Currently `src/ui/dialogue_box.rs` uses `portrait_index` to index into the NPC's 4×4 walk spritesheet, showing tiny walking sprites as "portraits". The fix: instead of using a TextureAtlas sprite for the portrait, use the same NPC spritesheet but render a CROPPED and SCALED version. Specifically:
- Use the walk-down row, frame 0 (index 0) as the portrait source
- Set the portrait sprite to `custom_size: Some(Vec2::new(64.0, 64.0))` (or appropriate size for the dialogue box)
- This at least shows a consistent front-facing pose instead of random walk frames
- Update the portrait_index usage to always use index 0 (front-facing idle) for each NPC

### Phase 4: Farming Visual Polish
**Files:** `src/farming/render.rs`, `src/world/objects.rs`

**Fix M3: Duplicate forageable atlas indices**
In `src/world/objects.rs`, the `forageable_atlas_index` function maps multiple items to the same index. Spread them out across the `grass_biome.png` atlas (45 frames available, indices 0-44). Assign unique indices to each forageable:
- Spring: wild_horseradish=3, daffodil=4, leek=5, dandelion=7
- Summer: grape=8, spice_berry=9, sweet_pea=10
- Fall: wild_plum=11, hazelnut=12, blackberry=13
- Winter: snow_yam=14, winter_root=15, crocus=16, crystal_fruit=17

**Fix M1: Stone and WoodFloor tile indices**
In `src/world/mod.rs`, `tile_atlas_info` for `TileKind::Stone` uses index 22 and `WoodFloor` uses index 33 from `tilled_dirt.png`. These are soil textures, not stone/wood. Use more appropriate indices from the tileset, or use the `hills.png` atlas for stone (index 0 area) and keep wood floor but pick a better index that looks like planks.

### Phase 5: Final Gate Verification
After all phases, run the full gate suite:
```bash
cargo check 2>&1
cargo test --test headless 2>&1
cargo clippy -- -D warnings 2>&1
```
All must pass. If any fail, fix and re-run.

Then commit everything with a single summary commit if not already committed per-phase.

Push: `git push -u origin claude/multi-agent-orchestration-vSy7F`

## IMPORTANT RULES
- Read `status/workers/visual_audit_report.md` before starting
- Read the actual source files before writing worker objectives
- Each worker objective must include: exact file paths, what to change, why, and validation commands
- Workers MUST run `cargo check` before reporting done
- Do NOT modify `src/shared/mod.rs` (frozen contract)
- All sprites use `Anchor::BottomCenter` now — any new sprite spawning must use this anchor
- After each phase, verify gates pass before moving to next
- If a worker breaks tests, dispatch a fix worker immediately
- Write a completion report to `status/workers/visual_steam_quality_report.md` when done
