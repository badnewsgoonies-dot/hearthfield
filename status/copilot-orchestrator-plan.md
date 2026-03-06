# Hearthfield 2.0 Orchestration Plan

Date: 2026-03-06
Orchestrator: Claude Opus 4.5 (Copilot CLI)

## Executive Summary

The Hearthfield codebase is **46K LOC**, compiles cleanly with `cargo check` and `cargo clippy -- -D warnings`, and has **87/88 tests passing** (1 test has stale expectations). The architecture is sound but needs refinement. I will **fix up the existing code** rather than rebuild - the foundation is solid.

## Current State Assessment

### Quality Gates
- ✅ `cargo check` — passes
- ✅ `cargo clippy -- -D warnings` — passes  
- ⚠️ `cargo test --test headless` — 87 pass, 1 fail (`test_baby_animal_grows_to_adult` expects 5 days but code correctly uses 7 days per spec)

### Critical Bugs (from retrospective)
1. **Time scale 10x off** — Fixed already! Code shows `time_scale: 1.0 / 6.0` (src/shared/mod.rs:108)
2. **NPC cleanup O(n²)** — `src/npcs/map_events.rs:31-40` uses `Vec::contains` in `retain`
3. **Farm west edge routing** — Goes to Beach, should go toward Mine
4. **Animal baby→adult age** — Code is correct (7 days), test is wrong (expects 5)
5. **No "being outside" happiness bonus** — Missing for animals
6. **expect() crash risk** — `src/ui/main_menu.rs:439`

### Performance Hotspots
1. **Minimap** — Full 4096-pixel texture rewrite every frame
2. **Farming render** — 3-pass full reconciliation every PostUpdate
3. **Y-sort** — 3 separate passes every frame
4. **HUD proximity** — Scans all NPCs/chests/interactables every frame
5. **Weather particles** — Unbounded count checks
6. **No FixedUpdate** — All movement in variable Update

### Architecture Improvements Needed
1. SystemSet taxonomy (Input → Intent → Simulation → Reactions → Presentation)
2. Split shared/mod.rs (2,315 lines → bounded context modules)
3. SubStates for overlays (dialogue, shop, inventory)
4. FixedUpdate for deterministic simulation
5. Reduce clone() density (496 clones, hotspot: save/mod.rs)
6. Split UiPlugin (currently mixes 15+ concerns)

---

## Execution Strategy

### Wave 1: Critical Bug Fixes (3 workers, parallel)

**Worker 1-A: Fix test + animal outside happiness**
- Fix `test_baby_animal_grows_to_adult` to expect 7 days
- Implement "being outside" happiness bonus for animals
- Files: `tests/headless.rs`, `src/animals/day_end.rs`

**Worker 1-B: Fix NPC O(n²) + farm west edge routing**
- Convert `Vec::contains` to `HashSet<Entity>` in `src/npcs/map_events.rs`
- Fix farm west edge to route toward Mine instead of Beach
- Files: `src/npcs/map_events.rs`, `src/player/interaction.rs`

**Worker 1-C: Fix expect() crash + error handling**
- Replace `expect()` with graceful fallback in `src/ui/main_menu.rs:439`
- Audit other potential panic points
- Files: `src/ui/main_menu.rs`

### Wave 2: Performance Optimizations (3 workers, parallel)

**Worker 2-A: Minimap caching**
- Only redraw minimap texture when map/entity state changes
- Add dirty flag system
- Files: `src/ui/minimap.rs`

**Worker 2-B: Farming render optimization**
- Replace 3-pass reconciliation with change detection / dirty flags
- Use Bevy's `Changed<>` query filters where appropriate
- Files: `src/farming/render.rs`

**Worker 2-C: Y-sort consolidation + HUD proximity caching**
- Combine 3 Y-sort passes into single pass
- Only rescan proximity when player moves a tile
- Files: `src/world/ysort.rs`, `src/ui/hud.rs`

### Wave 3: Architecture Foundation (2 workers, sequential then parallel)

**Worker 3-A: SystemSet taxonomy**
- Define shared SystemSet enums (Input, Intent, Simulation, Reactions, Presentation)
- Configure sets in main.rs
- Move key systems into sets
- Files: `src/shared/mod.rs`, `src/main.rs`, domain mod.rs files

**Worker 3-B: Split shared/mod.rs** (after 3-A completes)
- Split into: `shared/calendar.rs`, `shared/economy.rs`, `shared/world.rs`, `shared/social.rs`, `shared/events.rs`, etc.
- Keep `shared/mod.rs` as re-export facade
- Files: `src/shared/`

### Wave 4: Visual Polish (3 workers, parallel)

**Worker 4-A: Day transition polish**
- Smooth fade transitions for day end/start
- Gold ticker animation on gold changes
- Files: `src/ui/`, `src/calendar/`

**Worker 4-B: Tool use feedback**
- Screen shake on tool hits (subtle, 2-3 pixels)
- Particle effects for harvesting/mining
- Files: `src/player/`, `src/world/`

**Worker 4-C: NPC interaction polish**
- Gift reaction animations (heart/sweat/anger above head)
- Typewriter effect for dialogue
- Files: `src/npcs/`, `src/ui/dialogue_box.rs`

### Wave 5: Final Integration & Cleanup

**Worker 5-A: Integration tests**
- Add save/load round-trip tests
- Add mining/fishing loop integration tests
- Ensure all 93+ tests pass

**Worker 5-B: Final polish & documentation**
- Update any stale documentation
- Final clippy/test verification
- Performance spot-check

---

## Success Criteria

1. All quality gates pass: `cargo check && cargo test --test headless && cargo clippy -- -D warnings`
2. All 6 critical bugs fixed
3. Key performance hotspots addressed (minimap, farming render, Y-sort)
4. Architecture foundation laid (SystemSet taxonomy, shared split)
5. Visual polish improvements visible in player experience

## Constraints

- Max 3 parallel workers per wave
- Validate after each worker: `cargo check && cargo test --test headless`
- Commit after each successful wave
- Total budget: ~20-25 workers across all waves

---

## Status Tracking

### Wave 1: Critical Bug Fixes
- [ ] Worker 1-A: Test fix + animal happiness
- [ ] Worker 1-B: NPC O(n²) + farm routing
- [ ] Worker 1-C: expect() crash fix

### Wave 2: Performance
- [ ] Worker 2-A: Minimap caching
- [ ] Worker 2-B: Farming render optimization
- [ ] Worker 2-C: Y-sort + HUD proximity

### Wave 3: Architecture
- [ ] Worker 3-A: SystemSet taxonomy
- [ ] Worker 3-B: Split shared/mod.rs

### Wave 4: Visual Polish
- [ ] Worker 4-A: Day transitions
- [ ] Worker 4-B: Tool feedback
- [ ] Worker 4-C: NPC interaction

### Wave 5: Integration
- [ ] Worker 5-A: Integration tests
- [ ] Worker 5-B: Final cleanup
