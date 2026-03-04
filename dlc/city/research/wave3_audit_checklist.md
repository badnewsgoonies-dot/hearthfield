# Wave 3 Post-Integration Audit Checklist

Use this checklist after integrating Wave 3 changes into `city_office_worker_dlc`.

Auditor:
Date:
Commit SHA:

## 1) State Machine Correctness

- [ ] `OfficeGameState` includes exactly `Boot`, `MainMenu`, `InDay`, `DaySummary`, `Paused` (per `CONTRACT.md`).
  - Check: `rg -n "enum OfficeGameState|Boot|MainMenu|InDay|DaySummary|Paused" city_office_worker_dlc/src city_office_worker_dlc/CONTRACT.md`
  - Pass criteria: all five states are present, and no alternate replacement enum is used for core flow.

- [ ] Only valid transitions are reachable: `Boot -> MainMenu -> InDay -> DaySummary -> InDay`, plus pause/unpause.
  - Check: inspect transition systems and `NextState<OfficeGameState>` writes.
  - Command: `rg -n "NextState<OfficeGameState>|set\\(OfficeGameState::" city_office_worker_dlc/src`
  - Pass criteria: no direct transition bypasses `DaySummary` for end-of-day rollover.

- [ ] Salary/reputation rollover logic runs only in `DaySummary`.
  - Check: search writes to `money`/`reputation` and confirm state gating.
  - Command: `rg -n "money|reputation|salary|rollover" city_office_worker_dlc/src`
  - Pass criteria: rollover writes are gated to `OfficeGameState::DaySummary`.

- [ ] In-day system ordering matches contract (`Input -> Time -> TaskGeneration -> TaskResolution -> Interruptions -> Economy -> StateTransitions -> Ui`).
  - Check: plugin wiring and system set ordering.
  - Command: `rg -n "OfficeSimSet|add_systems\\(Update|\\.chain\\(\\)" city_office_worker_dlc/src`
  - Pass criteria: no ordering that allows UI/state transition side effects to run before core sim steps.

- [ ] `Paused` prevents time/task progression.
  - Check: pause gate test or run condition audit.
  - Pass criteria: while paused, minute-of-day, task progress, and day transition counters remain unchanged across `app.update()`.

## 2) Event Flow (`EndDayRequested` -> `DayAdvanced`)

- [ ] End-of-day trigger emits exactly one `EndDayRequested` when either shift end is reached or work is complete.
  - Check: headless test with both trigger paths.
  - Pass criteria: exactly one request event per day.

- [ ] `EndDayRequested` is consumed to evaluate board/stats and emits exactly one `DayAdvanced`.
  - Check: event reader/writer chain in state transition systems.
  - Pass criteria: one `DayAdvanced { new_day_index = previous_day + 1 }` per `EndDayRequested`.

- [ ] `DayAdvanced` cannot fire without a preceding `EndDayRequested`.
  - Check: negative test (no request sent).
  - Pass criteria: zero `DayAdvanced` events across multiple updates.

- [ ] Duplicate end-day requests in the same day do not produce duplicate `DayAdvanced`.
  - Check: send repeated `EndDayRequested` in one day.
  - Pass criteria: first request advances day; subsequent requests in same day are ignored/debounced.

- [ ] If legacy `EndOfDayEvent` still exists, adapter wiring is explicit and one-way.
  - Check: `rg -n "EndOfDayEvent|EndDayRequested|DayAdvanced" city_office_worker_dlc/src`
  - Pass criteria: either legacy event is removed, or adapter emits one `EndDayRequested` and does not double-advance.

## 3) TaskBoard Invariants

- [ ] `TaskBoard.active` has no duplicate `TaskId`.
  - Check: invariant/unit test inserts duplicate IDs.
  - Pass criteria: duplicate insertion is rejected or normalized deterministically.

- [ ] `OfficeTask.progress` is always clamped to `[0.0, 1.0]`.
  - Check: apply positive/negative deltas and assert clamp.
  - Pass criteria: values never underflow below `0.0` or overflow above `1.0`.

- [ ] Completion/failure exclusivity is enforced for same-day lifecycle.
  - Check: once task completes, attempt late failure path.
  - Pass criteria: completed task cannot later emit `TaskFailed` in same day.

- [ ] `completed_today` and `failed_today` stay disjoint.
  - Check: end-of-day validation step.
  - Pass criteria: set intersection is always empty.

- [ ] Terminal tasks are removed from `active` exactly once.
  - Check: completion and failure handlers.
  - Pass criteria: task ID appears in one terminal list and no longer appears in `active`.

- [ ] Save/load preserves TaskBoard identity and content.
  - Check: serialize + deserialize mid-day with active tasks.
  - Pass criteria: same `TaskId`s, progress, and deadlines; no regenerated duplicate tasks after load.

## 4) Deterministic Tests

- [ ] Fixed-seed 3-day replay is bit-for-bit stable for day outcomes.
  - Check: run deterministic replay twice with identical seed/action script.
  - Pass criteria: per-day summary fields match exactly (`day_index`, completed/failed counts, salary/reputation delta, stress delta).

- [ ] 5-day autoplay completes with no panic.
  - Check: headless autoplay test.
  - Pass criteria: test exits cleanly and reports no panic/unwind.

- [ ] Event ordering is deterministic under repeated identical inputs.
  - Check: run interruption/task sequence test multiple times.
  - Pass criteria: identical counters/outcomes each run.

- [ ] Randomness is seed-driven only.
  - Check: audit RNG initialization/source.
  - Pass criteria: no `thread_rng`/clock-based randomness in simulation-critical paths.

## 5) Compile/Test/Clippy Gates

Run from `/home/geni/swarm/hearthfield-office-dlc`:

- [ ] `cargo fmt --manifest-path city_office_worker_dlc/Cargo.toml --all -- --check`
- [ ] `cargo check --manifest-path city_office_worker_dlc/Cargo.toml`
- [ ] `cargo test --manifest-path city_office_worker_dlc/Cargo.toml`
- [ ] `cargo clippy --manifest-path city_office_worker_dlc/Cargo.toml --all-targets -- -D warnings`

Pass criteria:
- All commands exit `0`.
- No ignored failing tests.
- No clippy warnings (hard fail on warnings).

## 6) Evidence Log

- [ ] Attach command outputs (or CI links) for all gate commands.
- [ ] Link/record test names added for event flow + invariants.
- [ ] Record any waivers with owner + follow-up issue.
