# AI Status Overview — Phase 0 Discovery
# Created: 2026-03-05 by orchestrator agent (Claude Sonnet 4.6)
# Purpose: Ground truth snapshot before any implementation work

---

## What This Repo Actually Is

A Rust/Bevy 0.15 farming RPG with two DLC prototypes, organized as a Cargo workspace with a frozen shared type contract. The project uses a zero-scaffold orchestration approach: all coordination lives in conversation + disk artifacts, workers are dispatched via Copilot CLI or Codex CLI, and all scope enforcement is mechanical (git revert), not prompt-based.

Current wave: **Wave 11 (UX Feedback Coverage)** — dispatching 4 workers to add toast/SFX feedback to crafting, farming, fishing/mining, and weather.

---

## Base Game — What's Here

**Architecture:**
- 15 domains under `src/` (calendar, player, farming, animals, world, npcs, economy, crafting, fishing, mining, ui, save, data, input, shared)
- Type contract: `src/shared/mod.rs` (2,252 lines), checksum frozen in `.contract.sha256`
- Integration tests: `tests/headless.rs` — 88 tests as of Wave 9

**Completed waves (per ORCHESTRATOR_STATE.md):**
- Waves 1–3: 7 bugs fixed, 10 additional fixes, 3 UI screens
- Wave 4: DLC audit, pilot critical fixes, test coverage, deep parity
- Wave 5: Pilot DLC — 12 GameState variants, 12 screens wired, economy UI, inventory
- Wave 6a: Main game UI completion
- Wave 6b: City DLC full UI layer (+1,227 lines)
- Wave 7: 5 confirmed bugs fixed (R/L key collision, animal atlas, hotbar sizing, bed cutscene, dynamic key prompts)
- Wave 8: Path autotiling (4-bit bitmask, 16 tile variants) + farm object placement
- Wave 9: Cross-audit integration fixes (festival key, relationships screen, keybinding regression test, pilot sprites staged)
- Wave 10: (not explicitly named in state — likely the pilot headless test expansion)
- Wave 11: In progress — UX feedback coverage

**ROADMAP status (from ROADMAP_TO_SHIP.md):**
- Phase 1 (Playable): ~80% done. Workers dispatched for building collision, tutorial flow, visual readability.
- Phase 2 (Complete Loops): Farming loop verified working. Fishing, mining, crafting, social, economy need verification.
- Phase 3 (Visual Coherence): Not started.
- Phase 4 (Content/Narrative): Not started.
- Phase 5 (Ship/WASM): Save system verified (37 data structures round-trip). WASM build not done.

**Known open issues:**
- Building collision: players walk through walls — worker dispatched
- Tutorial flow: Days 2–3 hints missing — worker dispatched  
- Shop entry: doesn't auto-trigger GameState::Shop
- Atlas pre-loading: fallback colored rectangles on first frame
- Crafting bench interaction: unverified end-to-end
- Path autotiling: implemented in Wave 8 (replaced hardcoded crossroads index with bitmask)

---

## City Office Worker DLC — What's Here

**Location:** `dlc/city/`
**Crate name:** `city_office_worker_dlc`
**Scale:** 4,989 source lines, 30 tests

**Architecture:**
- Single `game` domain (vs. multi-domain tree in origin) — noted as a gap
- Lane-based systems: `systems/{bootstrap,input,tasks,interruptions,day_cycle,visuals,task_board}.rs`
- Event backbone: `EndDayRequested -> DayAdvanced { new_day_index }` — deterministic

**Gate dashboard (all 14 PASS as of 2026-03-03):**
- G0 Parity baseline frozen
- G1 Module topology (lane split done)
- G2 Event backbone deterministic
- G3 Fixed-seed 3-day replay stable
- G4 5-day seeded autoplay completes without panic
- G5 Task lifecycle deterministic
- G6 Save skeleton (snapshot serialize/deserialize/apply)
- G7 Durable save slots (filesystem roundtrip)
- G8 Migration stub (v0 → v1)
- G9 Quality (fmt/check/test/clippy -D warnings)
- G10 First-seconds stability (idempotent setup)
- G11 Content pack scaling (multi-kind/multi-priority templates with day-scaling)
- G12 Social determinism (seed-deterministic scenario selection)
- G13 Social persistence (save/load restores social graph)
- G14 Unlock progression (threshold/timeline/save-load deterministic)

**Current rotation:** R6 — Social/Progression Expansion (in progress)
- Social scenario branches expansion
- Endurance/balancing coverage
- Target: 200+ tests (currently 30)

**Remaining blockers toward OES-v1:**
1. Domain breadth gap (single domain vs. multi-domain origin)
2. Content scale (events/dialogue/scenario branches early-stage)
3. World/navigation parity early-stage

**Assessment:** This DLC is the tighter, more verifiable of the two. All quality gates are green. The 30 tests are robust (deterministic replay, save round-trips, endurance traces). Main gap is content volume and domain breadth, not correctness.

---

## Pilot DLC (Skywarden) — What's Here

**Location:** `dlc/pilot/`
**Crate name:** `skywarden`
**Scale:** ~31K LOC, 14 domains, 68 tests (from memory metadata)

**Architecture (14 domains):**
`aircraft, airports, crew, data, economy, flight, input, lib.rs, main.rs, missions, player, save, shared, ui, weather, world`

**Assessment from research context:**
- Wide horizontal scaffolding with hermetic domains
- Only 1 cross-domain call outside main.rs
- Coverage-first output — domains compile cleanly but may have false-green hermetic domains
- Built without "audit from player's perspective" instruction (contrast: City DLC was built with it)

**Objective files suggest recent work:**
- `fix-pilot-headless-tests.md` — adding tests for save/load, airports, etc.
- `fix-pilot-all-screens.md`, `fix-pilot-economy-ui.md`, etc. — multiple screen/system fix passes

**Key risk:** Hermetic domains = each domain compiles in isolation but doesn't actually call any other. A player running this may hit dead ends where systems exist but aren't wired together. The `fix-pilot-headless-tests.md` objective is actively addressing this.

---

## Gate Run Results (This Session)

Run in container (cargo not installed — results reflect local gate state, not container):

```
Gate 1: Contract Integrity      ✓ PASS
Gate 2: Type Check (cargo)      ✗ FAIL — cargo not available in this container
Gate 3: Integration Tests       ✗ FAIL — cargo not available in this container
Gate 4: Clippy                  ✗ FAIL — cargo not available in this container
Gate 5: Connectivity Check      ✓ PASS (all domains import from shared contract)
```

**Note:** Gates 2–4 fail only because `cargo` is not installed in this analysis container. Per ORCHESTRATOR_STATE.md, gates were green after Wave 9 (commit 4059f7b). The last confirmed gate state: 88/88 tests pass, zero clippy warnings, contract checksum OK.

---

## What Is and Isn't Ready

| Thing | State | Confidence |
|---|---|---|
| Type contract frozen + checksum valid | ✅ | High — Gate 1 passed |
| Base game compiles | ✅ | High — 88 tests passing as of Wave 9 |
| Farming loop end-to-end | ✅ | High — explicitly verified in ROADMAP |
| Save system round-trip | ✅ | High — 37 structures verified |
| Building collision | ⚠️ | Known bug, worker dispatched |
| Tutorial flow Days 1–7 | ⚠️ | ~80% — tutorial improvements in-flight |
| Fishing loop | ❓ | Needs verification |
| Mining loop | ❓ | Needs verification |
| Crafting bench | ❓ | Needs verification |
| Social/quest systems | ❓ | Needs verification |
| City DLC 3-day loop | ✅ | High — G3/G4 prove 3- and 5-day seeded runs complete |
| City DLC quality gates | ✅ | High — all 14 green as of 2026-03-03 |
| Pilot DLC 2-flight loop | ❓ | Unknown — hermetic domain risk |
| Pilot DLC test coverage | ⚠️ | Objective file suggests tests being added now |
| WASM build | ❌ | Not started |
| Full year playthrough (112 days) | ❌ | Not verified |

---

## Open Risks

1. **Pilot hermetic domains:** Skywarden's 14 domains may not wire together into a playable loop. No confirmed end-to-end "player completes a flight" verification exists in the status files. This is the highest-risk item.

2. **Building collision** (base game): Players can walk through walls. Explicitly noted as "worker dispatched" but not yet committed in the wave log.

3. **Atlas pre-loading:** Fallback colored rectangles visible on first frame. A player's first impression is broken sprites.

4. **City DLC content scale:** 30 tests is solid for correctness but the 200-test OES-v1 target means content volume is not there yet. The DLC is mechanically sound but thin.

5. **Wave 11 in-flight:** 4 workers dispatched but not committed. Run-gates may not pass until these land.

---

## Recommended Next Steps (in priority order)

1. **Verify building collision fix landed** — check if Wave 11 or pending workers addressed it, or dispatch targeted fix
2. **Verify pilot end-to-end** — dispatch a "can a player complete 2 flights?" verification worker
3. **Run gates locally** (`./scripts/run-gates.sh`) and confirm 88 tests still green post-Wave 11
4. **Verify fishing and mining loops** — both are "needs verification" on the ROADMAP
5. **Atlas pre-load** — dispatch a single-scope worker to eager-load all atlases in Loading state
