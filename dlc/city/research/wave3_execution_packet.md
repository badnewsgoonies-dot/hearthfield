# Wave 3 Execution Packet: Contract Alignment

Date: 2026-03-03
Target repo: `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc`
Wave scope: persistence + balance pass, with contract alignment and zero gameplay regression.

## 1) Contract Sections To Implement Now (Minimal High-Leverage Subset)

Implement now (required for Wave 3):

1. `CONTRACT.md` §1 App States (minimal behavior slice)
- Enforce `OfficeGameState::InDay` simulation boundary.
- Enforce `OfficeGameState::DaySummary` as the only place that applies salary/reputation/stress rollover.
- Keep `Boot -> InDay` automatic for now to preserve current instant-play loop.

2. `CONTRACT.md` §2 Core Resources
- Add and wire: `OfficeClock`, `WorkerStats`, `TaskBoard`, `OfficeRunConfig`, `DayOutcome`.
- Keep current legacy resources temporarily as compatibility mirrors during Wave 3.

3. `CONTRACT.md` §3 Core Value Types
- Add and use: `TaskId`, `TaskKind`, `TaskPriority`, `OfficeTask`, `InterruptionKind`, `ChoiceId`.
- Minimum runtime requirement: `TaskBoard.active` uses `OfficeTask` and persists exact `TaskId` values.

4. `CONTRACT.md` §5 Events
- Add and wire: `TaskAccepted`, `TaskProgressed`, `TaskCompleted`, `TaskFailed`, `InterruptionTriggered`, `InterruptionChoiceMade`, `EndDayRequested`, `DayAdvanced`.
- Preserve old event structs short-term via adapter systems so keyboard loop remains playable.

5. `CONTRACT.md` §6 System Sets and Ordering
- Introduce `OfficeSimSet` and enforce ordered execution in `InDay`:
  `Input -> Time -> TaskGeneration -> TaskResolution -> Interruptions -> Economy -> StateTransitions -> Ui`.

6. `CONTRACT.md` §7 Persistence Contract
- Save/load at minimum: `OfficeClock.day_index`, `WorkerStats`, `TaskBoard.active` (including deadlines/progress), `OfficeRunConfig.seed`.
- Enforce invariants: exact `TaskId` round-trip and no task regeneration on mid-day load.

7. `CONTRACT.md` §8 Invariants/Test Expectations (Wave 3 critical subset)
- Stat clamps always enforced.
- Duplicate `TaskId` rejected.
- Completed task cannot fail later in same day.
- `DayAdvanced` emitted at most once/day.
- Deterministic fixed-seed replay and 5-day no-panic autoplay.

Defer to Wave 4 (explicitly not in this packet):
- Full `MainMenu`/`Paused` UX behavior.
- Full component contract migration in §4 (`PlayerOfficeWorker`, `OfficeDesk`, etc.).
- Removal of legacy event/resource names.

## 2) Exact File-By-File Change Map (`mod/events/resources/systems`)

### `src/game/mod.rs`

1. Add `OfficeGameState` initialization and runtime boundary wiring.
- `init_state::<OfficeGameState>()`.
- Startup/OnEnter bridge so current boot still lands in `InDay` automatically.

2. Register contract resources in app startup.
- `OfficeClock`, `WorkerStats`, `TaskBoard`, `OfficeRunConfig`, `DayOutcome`.
- Keep existing resource init calls for Wave 3 compatibility.

3. Register contract events.
- Add all §5 contract events to `App`.
- Keep legacy event registration for adapter period.

4. Configure ordered sets.
- Add `OfficeSimSet` configuration for required order.
- Move system registration from a single `.chain()` list to set-based ordering (still deterministic).

5. Schedule state-specific systems.
- `InDay`: simulation + adapters + persistence reads/writes.
- `DaySummary`: rollover/apply `DayOutcome`, then transition back to `InDay`.

### `src/game/events.rs`

1. Add contract event structs from §5 verbatim names.
- `TaskAccepted`, `TaskProgressed`, `TaskCompleted`, `TaskFailed`.
- `InterruptionTriggered`, `InterruptionChoiceMade`.
- `EndDayRequested`, `DayAdvanced`.

2. Keep legacy event structs during Wave 3.
- `ProcessInboxEvent`, `CoffeeBreakEvent`, `WaitEvent`, interruption/NPC events, `EndOfDayEvent` remain for compatibility.

3. Add short comment markers designating legacy events as temporary adapter surface.

### `src/game/resources.rs`

1. Add contract resource/value-type definitions.
- §2 resources + §3 value types with contract names and field semantics.

2. Add defaults/mapping needed for current prototype continuity.
- Provide defaults compatible with existing gameplay values (legacy timing and tuning preserved during Wave 3).
- Keep clock/task fields in ranges expected by contract (including clamps).

3. Add normalization and validation helpers used by systems.
- Worker stat clamp helper.
- Task progress clamp helper.
- Duplicate `TaskId` detection/guard helper.

4. Keep existing legacy resources for adapter period.
- `OfficeRules`, `InboxState`, `DayClock`, `PlayerMindState`, `PlayerCareerState`, `DayStats` remain until Wave 4 cleanup.

### `src/game/systems.rs`

1. Add adapter systems (Wave 3 migration backbone).
- Legacy input/events -> contract events bridge.
- Legacy resources <-> contract resources sync (single direction per phase to avoid feedback loops).
- Contract day-end -> legacy summary event bridge so current summary output still works.

2. Add contract-compliant simulation systems by set.
- `Time`: clock advancement and pause/time-scale handling.
- `TaskGeneration`: deterministic intake honoring `OfficeRunConfig`/seed.
- `TaskResolution`: apply `TaskProgressed`, emit `TaskCompleted`/`TaskFailed`, enforce no duplicate terminal outcomes.
- `Interruptions`: handle trigger + choice outcomes via contract event pair.
- `Economy`: compute `DayOutcome`, enforce stat clamps.
- `StateTransitions`: emit `EndDayRequested`, evaluate once, emit `DayAdvanced` once/day.

3. Add persistence systems.
- Save snapshot writer (on `DayAdvanced` and/or explicit debug trigger).
- Load snapshot reader at boot/start-day path.
- Mid-day load reconciliation that preserves existing active tasks and IDs.

4. Preserve current player-visible loop while migrating.
- Keep existing visual update and summary print systems active.
- Keep keybind behavior unchanged during Wave 3.

## 3) Backward Compatibility Strategy (Keep Current Loop Playable)

Compatibility mode for Wave 3 is staged and reversible:

1. Stage A: Shadow Contract (default for first integration pass)
- Legacy loop remains authoritative for gameplay outcomes.
- Contract resources/events are populated in parallel by adapter systems.
- Goal: prove contract flow + persistence without changing player-facing behavior.

2. Stage B: Contract Authoritative with Legacy Facade
- Contract systems become authoritative for day progression/outcome.
- Legacy `EndOfDayEvent` and legacy read-model values are emitted from contract state for existing summary/visual paths.
- Goal: remove semantic drift while preserving current controls and output.

3. Stage C: Cleanup trigger (Wave 4)
- Remove legacy resources/events after gates have been green for a full wave.

Guardrails to prevent regressions:
- Keep controls unchanged (`P/C/N/I/1/2/M/H`) throughout Wave 3.
- If save file missing/corrupt, fall back to clean day start (no crash, no soft-lock).
- Preserve current playability defaults during migration; apply balance retune only after persistence round-trip is stable.
- Add one-switch rollback resource/flag for adapter authority direction if regression appears.

## 4) Wave 3 Exit Gates

Wave 3 is complete only when all gates below are PASS:

1. Contract Alignment Gate
- Drift count for implemented sections (§1/§2/§3/§5/§6/§7/§8 subset) is `0`.
- No alternate naming introduced outside `CONTRACT.md`.

2. Persistence Round-Trip Gate
- Save -> load -> save produces byte-equivalent semantic state for required fields.
- Exact preservation of all `TaskId` values.
- Mid-day load does not regenerate already active tasks.

3. Simulation Stability Gate
- Fixed-seed 3-day replay yields identical day outcomes across runs.
- 5-day autoplay completes with `panic_count = 0` and no deadlock.

4. Invariant Gate
- `WorkerStats` clamps never violated.
- Duplicate `TaskId` rejected.
- Completed tasks do not later fail same day.
- `DayAdvanced` emitted at most once/day.

5. Backward-Compatibility Gate
- Current keyboard loop remains playable end-to-end during migration.
- End-of-day summary still renders correctly while adapters are active.

6. Tooling/Quality Gate
- `cargo check --manifest-path city_office_worker_dlc/Cargo.toml` passes.
- `cargo test --manifest-path city_office_worker_dlc/Cargo.toml` passes.
- `cargo clippy --manifest-path city_office_worker_dlc/Cargo.toml -- -D warnings` passes.
- Scope guard confirms only allowlisted files changed per lane.

