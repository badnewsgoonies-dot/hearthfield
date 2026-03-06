# Hearthfield Retrospective Executive Summary

Date: 2026-03-06  
Scope: synthesis of reports in `status/retrospective/` for code history, architecture, quality, spec gap, performance, process, Bevy best practices, and alternatives.

## 1) What went right (keep doing)

- Domain plugin decomposition was the right baseline for a 46k LOC Bevy game. It enabled parallel implementation across many gameplay areas.
- Delivery speed was very high: 305 commits over 9 active days, with peak days of 64 and 66 commits.
- Breadth-first vertical delivery worked: most core sim domains reached high implementation coverage (farming 90%, economy 90%, crafting 92%, fishing 90%, mining 93%).
- Quality gates had real value: `cargo clippy -- -D warnings` passes cleanly.
- Event rails exist and are useful for lifecycle transitions (`DayEnd`, `SeasonChange`, map transitions, gold changes), which gives a viable foundation for cleaner decoupling.
- Testing depth exists in core simulation behavior: `tests/headless.rs` contains 90 tests and covers many farming/economy/social rule paths.
- Orchestrator process produced strong auditability: 103 worker reports with clear historical trace.

## 2) What went wrong (stop doing)

- Stop using one giant shared contract file as the default integration seam. `src/shared/mod.rs` grew to 2315 lines, was modified in 39 commits, and became a rework hotspot.
- Stop treating all plugins as equal isolation units. In practice, `ui` and `save` became cross-domain integrators with deep imports, weakening boundaries.
- Stop overloading `Update` with broad full-world scans and full-state reconciliations. Several systems run expensive O(n)-style passes every frame, and at least one confirmed O(n^2) pattern exists.
- Stop allowing cross-domain ordering dependencies as implicit contracts (`.before(...)` across domain internals). This creates fragile integration coupling.
- Stop accepting high fix-loop churn as normal throughput. 31 of 103 worker reports were explicit fix passes (30.1%), and 36 of 79 objectives were fix-focused (45.6%).
- Stop conflating “spec complete” with “feature complete”: multiple spec mismatches remained (time scale, map sizing/transition mismatch, settings persistence gaps, etc.).

## 3) What we should have done differently (start doing)

- Start with explicit architecture tiers:
  - Core domains: depend only on shared contracts/events.
  - Orchestrators (UI/save): allowed integration dependencies, but tightly scoped and documented.
- Start splitting shared contracts by bounded context from day 1 (`shared/calendar`, `shared/economy`, `shared/world`, `shared/social`, `shared/events`, etc.) with a small re-export facade.
- Start using `SystemSet` taxonomy and `configure_sets(Update, ...)` early (input -> intent -> simulation -> reactions -> presentation).
- Start layering state models using Bevy 0.15 `SubStates`/`ComputedStates` and `StateScoped` entities/events for menu/overlay-heavy flows.
- Start moving deterministic simulation islands to `FixedUpdate` where appropriate (NPC/animal movement cadence, mine combat windows, selected weather/motion loops).
- Start replacing broad per-frame reconciliation with change-driven updates (dirty flags, event-triggered sync, map-local caches).
- Start with a data-driven content pipeline for static game data (items/crops/fish/npcs/shops in RON/TOML) plus strict schema/reference validation tests.
- Start tiered gates by risk:
  - Low-risk UI text/config: cheap checks.
  - Core sim/save changes: full gate suite.

## 4) Quantitative metrics

### Repo and velocity
- Total commits: 305
- Merge commits: 34
- Non-merge commits: 271
- Active development window: 2026-02-26 to 2026-03-06 (9 active days)
- Daily commits: 35, 30, 22, 34, 64, 42, 66, 7, 5
- Largest observed inter-commit stalls: ~19.2h, ~16.2h, ~12.0h

### Ownership and churn
- Top contributors by commit count: Claude (100), jim jam (84), Geni (Orchestrator) (57), badnewsgoonies (28)
- `src/shared/mod.rs`:
  - Size: 2315 LOC
  - Modified in 39 commits
  - Peak change burst date: 2026-03-02 (16 commits touching file)
- Highest domain touch counts: UI (72), World (59), NPCs (56), Player (44)
- Highest churn (added+deleted): UI (12,981), NPCs (9,925), World (8,608), Data (6,326)

### Quality and test posture
- `cargo clippy -- -D warnings`: pass
- Rust LOC under `src/`: 46,454
- `unwrap()` in `src+tests`: 30 (17 in tests, 8 shared, 5 npc schedules)
- `#[allow(clippy::...)]` in `src+tests`: 70 (65 are `too_many_arguments`)
- `.clone()` in `src+tests`: 496 (hotspot: `src/save/mod.rs`)
- `tests/headless.rs`: 90 tests
- Gaps: save/load migration robustness, mining/fishing full-loop integration depth, UI interaction integration coverage, negative/error-path coverage

### Spec coverage (estimated)
- Calendar 80%, Player 82%, Farming 90%, Animals 78%, World 68%, NPCs 76%, Economy 90%, Crafting 92%, Fishing 90%, Mining 93%, UI 74%, Save/Settings 72%
- Main misses/mismatches: time-scale mismatch, incomplete settings persistence/remap parity, world size/transition drift, some interaction-model differences

### Performance risk indicators
- No `FixedUpdate` usage found
- Confirmed O(n^2) pattern in NPC transition cleanup (`retain` + `contains`)
- Multiple heavy per-frame broad scans in UI/world/farming/minimap/weather paths
- Mine tile-layer duplication between world map tiles and mining tiles increases entity pressure

## 5) Recommended architecture for Hearthfield 2.0

### Core approach
- Keep hybrid ECS (best fit for this project’s scope), but formalize boundaries and scheduling.

### Target structure
- Plugin hierarchy:
  - `CorePluginGroup` (calendar, player, world, farming, animals, economy, crafting, fishing, mining, npcs)
  - `InterfacePluginGroup` (hud, menus, dialogue, overlays, minimap)
  - `PlatformPluginGroup` (save, input adapters, telemetry/debug)
- Shared contracts:
  - Break into bounded modules; preserve `shared::prelude` for ergonomics.
- Cross-domain writes:
  - Prefer command/event boundaries; avoid direct deep imports across gameplay domains.
- Scheduling:
  - Define `SystemSet` phases; configure once; limit ad-hoc `.before/.after` edges.
  - Introduce fixed-step islands for deterministic mechanics.
- State model:
  - High-level flow state + sub/computed states for overlays/modes.
  - Use state-scoped entities/events for lifecycle cleanup.
- Data model:
  - Externalize static content to validated data files.
  - Keep core formulas/rules in code.
- Testing strategy:
  - Keep headless simulation tests.
  - Add save/load round-trip + migration golden tests.
  - Add integration tests for mining/fishing loops and key UI flows.
  - Add performance budget tests for high-entity scenarios.

## 6) Estimated cost comparison (actual vs optimal)

### Actual (estimated from artifacts)
- Persisted artifact footprint lower bound: ~103k tokens.
- Likely orchestration total (worker runs + overhead): ~155k to ~412k tokens.
- Premium-request proxy from report volume: roughly ~100+ premium-class requests.
- Rework overhead signals:
  - 30.1% of worker reports were fix passes.
  - 45.6% of objectives were fix-oriented.

### Optimal (counterfactual with early architecture discipline)
- If shared contract had been split early, sets/states standardized early, and tiered gates applied:
  - Expected fix-report ratio: 30.1% -> ~15-20%
  - Expected objective fix share: 45.6% -> ~25-30%
  - Expected orchestration token reduction: ~25-40%
  - Expected integration defect reduction in UI/save/player/NPC hotspots: material, likely with fewer late audit campaigns

### Net
- This project optimized for speed-to-breadth and achieved it.
- It paid for that with elevated integration churn and process overhead.
- Hearthfield 2.0 should preserve plugin-per-domain speed while enforcing stronger boundary/scheduling/data architecture from the first 20% of development.

