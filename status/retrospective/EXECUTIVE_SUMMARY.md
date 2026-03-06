# Hearthfield Retrospective Executive Summary

Date: 2026-03-06
Scope: Base game repository retrospective across history, architecture, quality, spec conformance, performance, tooling/process, Bevy best practices, and alternatives.

## 1) What went right (keep doing)

- **Plugin-per-domain baseline worked for delivery speed.** The project shipped broad gameplay scope fast (305 commits across 9 active days) with clear domain ownership in `main.rs` wiring.
- **Strong quality gates existed and held.** `cargo clippy -- -D warnings` passes, and process discipline (contract/clamp/gates/manifest) produced low-scope violations and reproducible fix waves.
- **Feature breadth is high versus original design.** Spec-gap analysis estimates ~84% implementation overall across 12 domains, with very high completion in farming/economy/mining/fishing.
- **Tests are meaningful in core simulation areas.** `tests/headless.rs` includes 90 tests and provides broad coverage of farming, economy, calendar, animals, progression, and key gameplay loops.
- **Audit-and-fix execution cadence was effective.** Late bug-sweep campaigns resolved critical issues quickly and repeatedly.

## 2) What went wrong (stop doing)

- **Monolithic shared contract became an integration bottleneck.** `src/shared/mod.rs` grew to 2315 lines, 127 public structs/enums, 36 resources, 32 events; it was modified in 39 commits (with a 16-commit burst on 2026-03-02).
- **Architectural boundaries were declared but not enforced.** Direct cross-domain imports (especially in `ui`, `save`, and parts of `player`) eroded intended domain isolation.
- **Rework churn was high in key areas.** Highest domain churn landed in `ui` (72 commit touches; 12,981 changed lines), then `world` and `npcs`; build process shows 31/103 worker reports as `fix-*` (30.1%) and 36/79 objectives as `fix-*` (45.6%).
- **Per-frame work accumulated avoidable hotspots.** No `FixedUpdate` usage, broad per-frame scans in HUD/world/farming/minimap, and at least one confirmed O(n^2) cleanup path in NPC transition handling.
- **Spec compliance drifted in important UX/system details.** Biggest gaps are UI/settings parity, world scale targets, NPC dialogue/tree/pathfinding depth, and a few explicit behavior mismatches.

## 3) What we should have done differently (start doing)

- **Architectural tiering from day 1.** Keep domain plugins isolated; explicitly classify `ui` and `save` as orchestrator/platform layers with controlled dependency exceptions.
- **Split shared contract early.** Replace single `shared/mod.rs` with bounded modules (`shared/core_state`, `shared/world`, `shared/economy`, etc.) and a stable re-export facade.
- **Use Bevy scheduling primitives as first-class architecture.** Introduce `SystemSet` taxonomy + `configure_sets`, adopt `SubStates`/`ComputedStates` for overlay/mode logic, and apply state-scoped entities/events.
- **Move high-frequency simulation islands to fixed cadence.** Use `FixedUpdate` for deterministic simulation-heavy loops (movement/AI/combat-like systems), keep UI/input in `Update`.
- **Shift cross-domain writes to command/event rails.** Allow direct reads pragmatically, but require event/command boundaries for writes that mutate another domain’s state.
- **Data-drive content earlier.** Externalize item/crop/fish/NPC content tables (RON/TOML) with strict schema/consistency validators; keep complex game rules code-first.
- **Risk-tier gates to reduce token/time burn.** Keep strict full gates for high-risk changes, lighter gates for low-risk scope to cut repeated overhead.

## 4) Quantitative metrics

- **Codebase size:** 46,454 LOC under `src` Rust files.
- **Git history:** 305 commits total, 34 merges, 271 non-merge; active from 2026-02-26 to 2026-03-06.
- **Peak velocity:** 66 commits/day (2026-03-04), with a second major peak at 64 commits/day (2026-03-02).
- **Shared-contract churn:** `src/shared/mod.rs` touched in 39 commits.
- **Highest rework domains:** `ui` (72 touches), `world` (59), `npcs` (56), `player` (44).
- **Spec implementation estimate:** ~84% average across 12 domains.
- **Largest spec gaps:** Save & Settings (68%), UI System (72%), World & Maps (70%), NPC depth (78%).
- **Quality indicators:** Clippy gate pass; 0 TODO/FIXME/HACK/XXX markers in Rust source; 30 `unwrap()` in `src+tests` (17 in tests, 13 in src), 496 `.clone()` in `src+tests`; 70 clippy suppressions (65 are `too_many_arguments`).
- **Testing footprint:** 90 tests in `tests/headless.rs` with strong coverage in sim/economy/farming/social; weaker coverage in save/load, mining/fishing integration, UI interaction paths.
- **Process overhead:** 103 worker reports, 79 objectives, persisted planning/reporting corpus ~103k tokens minimum.

## 5) Recommended architecture for “Hearthfield 2.0”

- **Core shape:** Keep hybrid ECS (resource-centric simulation + ECS projections), not full ECS-pure rewrite.
- **Plugin model:** Preserve plugin-per-domain but split large integrators (especially UI) into smaller state-owned sub-plugins behind a parent plugin group.
- **Shared contracts:** Modularize into bounded packages plus a `shared::prelude` facade.
- **Scheduling model:** Define global phase sets (`Input -> Intent -> Simulation -> Reactions -> Presentation`) and configure once; migrate ad-hoc ordering edges into set-level ordering.
- **State model:** One high-level flow state + layered Sub/Computed states for overlays/modes; remove broad run-condition sprawl.
- **Boundary policy:**
  - Core domains: depend only on `shared` contracts/events.
  - Orchestrators (`ui`, `save`): allowed curated dependencies via adapters/read models.
  - Cross-domain mutations: command/event only.
- **Performance policy:**
  - Fixed-step for deterministic simulation islands.
  - Event-triggered/dirty-flag rendering sync instead of unconditional full reconciliation.
  - Replace O(n^2) cleanup patterns with hash-indexed sets/maps.
- **Content pipeline:** Externalized data tables + schema validation + golden-data tests.

## 6) Estimated cost comparison (actual vs optimal)

### Actual (observed/estimated)
- Persisted artifacts imply at least **~103k tokens** consumed for planning/reporting text alone.
- Practical orchestration estimate from report corpus: **~155k to ~412k tokens** for worker interactions, excluding substantial context replay overhead.
- High fix-loop rate (30-46% depending on artifact set) indicates significant repeated gate/run/report cost.

### Optimal (with hindsight architecture/process)
Expected reductions if 2.0 started with modular shared contracts, set/state architecture, explicit tiered boundaries, and risk-tier gating:

- **Fix-loop reduction:** from ~30-46% to ~15-20% (roughly halving rework waves in churn-heavy domains).
- **Gate overhead reduction:** 25-40% by tiering full vs light gate paths by risk.
- **Token/process reduction:** **~30-45% lower total orchestration spend** for equivalent feature scope.
- **Delivery shape:** fewer late-stage cross-domain rewrites, flatter defect curve, and lower integration volatility around `ui/save/shared`.

Bottom line: Hearthfield shipped quickly and broadly, but it paid a clear rework tax. Hearthfield 2.0 should keep the successful domain-plugin/hybrid-ECS foundation while front-loading architectural boundaries, Bevy scheduling primitives, and modular contracts to cut churn and cost.
