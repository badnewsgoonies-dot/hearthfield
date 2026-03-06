# Visual Pass 2 — Steam-Quality Orchestrator

You are the ORCHESTRATOR (GPT 5.4). You dispatch Opus 4.6 workers via `copilot -p` to fix remaining visual bugs and add polish. You do NOT write code yourself — you write worker specs and dispatch them.

## How to Dispatch Workers
For each phase, write a detailed worker objective to `/tmp/worker_vp2_N.md`, then dispatch:
```bash
source ~/.env_tokens
copilot -p "$(cat /tmp/worker_vp2_N.md)" --model claude-opus-4.6 --yolo --add-dir /home/user/hearthfield 2>&1 | tee status/workers/visual_pass2_phase_N.json
```

## CRITICAL: Read These First
1. `status/workers/visual_audit_report.md` — full bug list with severity ratings
2. `status/workers/visual_steam_quality_report.md` — what was already fixed in pass 1
3. `MANIFEST.md` — project state and remaining issues
4. `src/shared/mod.rs` — the frozen type contract (DO NOT MODIFY)

## Context
Hearthfield is a Harvest Moon farming sim in Rust/Bevy 0.15. Pass 1 fixed: BottomCenter anchors, tool animation direction, water animation, grass variation, bed/stove y-sort, NPC face-player, portraits, forageable indices, floor tiles.

## PHASES (execute in order)

### Phase 1: Crop Growth Animation
**Worker scope:** `src/farming/render.rs`
**Bug M4:** Crops jump instantly between growth stages. Add a smooth visual transition:
- When a crop advances a growth stage, briefly scale the sprite (e.g., 0.8 → 1.0 over 0.3s) to give a "pop" growth feel
- Add a `CropGrowthAnim` component with a timer that drives the scale interpolation
- The crop render system (`sync_crop_sprites`) already runs each frame — add the scale lerp there
- Read `src/farming/render.rs` and `src/farming/mod.rs` first to understand the existing crop lifecycle
- The crop atlas index function already maps stage → sprite. Just add the scale animation on stage transitions

### Phase 2: Fish Data Fix + Water Edge Transitions
**Worker scope:** `src/data/fish.rs`, `src/world/mod.rs`
**Bug C1:** Five fish (koi, ghostfish, stonefish, ice_pip, lava_eel) have sprite_index values 48-52 but fishing_atlas.png only has 48 frames (0-47). Renumber these to valid indices within 0-47 that aren't already used by other fish. Read `src/data/fish.rs` to see all current sprite_index values and pick unused ones.
**Bug M6:** Water tiles have hard borders with no edge transitions. In the tile rendering code (`sync_map_tiles` or equivalent in `src/world/mod.rs`), detect water-adjacent tiles and use appropriate edge indices from the water atlas (4 frames). At minimum:
- Water tiles adjacent to non-water should use a different visual (e.g., alternate water frame index based on neighbor count)
- This gives a subtle visual distinction at water edges vs. deep water centers

### Phase 3: Animal Animation System
**Worker scope:** `src/animals/mod.rs`, `src/animals/spawning.rs`
**Bug H4:** Animals have zero animation — chickens and cows are frozen at atlas index 0. Add a basic animation system:
- Create an `AnimalAnimationTimer` component (similar to `NpcAnimationTimer`)
- Chickens: cycle through frames 0-3 (4 frames in the 4×2 atlas) at 0.2s intervals when moving
- Cows: cycle through frames 0-2 (3 frames in the 3×2 atlas) at 0.25s intervals when moving
- When idle (not wandering), stay at frame 0
- The animal wander system already sets movement targets — check if the animal is moving toward a target to determine animation state
- Read `src/animals/mod.rs` and `src/animals/spawning.rs` first

### Phase 4: Emote Atlas Verification + Path Autotile
**Worker scope:** `src/npcs/emotes.rs`, `src/world/mod.rs`
**Bug H6:** Emote indices are self-described as "educated guesses". Read the emotes.png atlas (160×480, 10 cols × 30 rows = 300 frames). The Sprout Lands emote sheet typically has: row 0 = hearts/love, row 1 = happy/joy, row 2 = exclamation/surprise, row 3 = neutral, row 4 = sad, row 5 = angry, row 6 = question/confused. Verify the current indices against this layout and fix if wrong. Current code uses: Heart=0, Happy=10, Exclamation=20, Neutral=30, Sad=40, Angry=50, Question=60.
**Bug M2:** Path/fence autotile maps bitmask directly to atlas index. This is only correct if the tileset is ordered in ascending bitmask order. For Sprout Lands, verify or add a lookup table that maps 4-bit neighbor bitmask → correct atlas index. Read `src/world/mod.rs` functions `path_autotile_index` and `fence_autotile_index`.

### Phase 5: Final Gate Verification + Push
Run all gates:
```bash
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Fix any failures. Then:
```bash
git add -A && git commit -m "feat(visual): pass 2 — crop anim, fish fix, water edges, animal anim, emotes, autotile"
git push -u origin claude/multi-agent-orchestration-vSy7F
```
Write completion report to `status/workers/visual_pass2_report.md`.

## RULES
- Read source files BEFORE writing worker specs
- Each worker spec must include: exact file paths, what to read first, exact changes, validation commands
- Workers must run `cargo check` before reporting done
- Do NOT modify `src/shared/mod.rs` (frozen contract)
- All sprites use `Anchor::BottomCenter` — new sprite spawning must use this
- After each worker, run gates. If they fail, dispatch fix worker (max 3 retries)
- Commit after each passing phase
- Stagger worker launches by ~3 seconds

## Gate Commands
```bash
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

BEGIN NOW. Read the audit report and pass 1 report first, then dispatch Phase 1.
