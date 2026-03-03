# City Office Worker DLC - DECISIONS

Format: concise ADR log for accepted prototype decisions.

## ADR-001 - Keep DLC as a Separate Crate

- Date: 2026-03-03
- Status: Accepted
- Context: We needed fast iteration without destabilizing the main game crate.
- Decision: Implement the DLC as its own Rust crate at `city_office_worker_dlc/` with an independent `Cargo.toml`.
- Why: This keeps build/run/test loops isolated (`--manifest-path`) and lowers cross-project risk.

## ADR-002 - Use One Top-Level Game Plugin for Wave 0

- Date: 2026-03-03
- Status: Accepted
- Context: Early prototype required minimal wiring complexity.
- Decision: Register gameplay through `CityOfficeWorkerPlugin` in `src/game/mod.rs`.
- Why: Centralized plugin wiring made bootstrap and debugging straightforward for the first vertical slice.

## ADR-003 - Enforce Deterministic Update Order with Chained Systems

- Date: 2026-03-03
- Status: Accepted
- Context: Action handling, day checks, and summary output must run predictably each frame.
- Decision: Use a chained `Update` system pipeline (`collect input -> handlers -> end-of-day -> summary -> visuals`).
- Why: Deterministic ordering reduces race/ordering bugs and supports future headless replay tests.

## ADR-004 - Make Time Advance per Explicit Player Action

- Date: 2026-03-03
- Status: Accepted
- Context: Wave 0 focused on readable loop behavior over real-time simulation complexity.
- Decision: `P`, `C`, and `N` actions advance the day clock by configured minutes.
- Why: Turn-like time progression made balancing and debugging easier while keeping the loop immediately playable.

## ADR-005 - End Day on Shift End or Inbox Completion

- Date: 2026-03-03
- Status: Accepted
- Context: The prototype needed clear and testable day completion conditions.
- Decision: Day ends when `current_minute >= day_end_minute` or inbox reaches zero, then emits `EndOfDayEvent` once.
- Why: This gives a deterministic finish line for each day and prevents duplicate summary emissions.

## ADR-006 - Keep Wave 0 Feedback Lightweight (Console + Simple Visuals)

- Date: 2026-03-03
- Status: Accepted
- Context: UI content was not the critical path for validating the core loop.
- Decision: Use console summaries plus generated sprite/color feedback instead of full authored UI assets.
- Why: This preserved implementation velocity while still exposing enough state for tuning and debugging.

## ADR-007 - Centralize Tunable Numbers in `OfficeRules`

- Date: 2026-03-03
- Status: Accepted
- Context: Energy costs, action durations, and inbox size needed quick iteration.
- Decision: Store loop constants in `OfficeRules` resource defaults rather than scattering literals across systems.
- Why: A single tuning surface speeds balancing and keeps behavior changes auditable.

## ADR-008 - DaySummary Owns Rollover While InDay Emits Next-Day Intent

- Date: 2026-03-03
- Status: Accepted
- Context: Audit findings flagged ambiguous ownership between end-of-day detection, advancement, and rollover side effects.
- Decision: Keep `EndDayRequested -> DayAdvanced { new_day_index }` emission in `InDay` (`finalize_end_day_request`), but apply salary/reputation rollover only in DaySummary systems (`apply_day_summary_rollover`, `transition_day_summary_to_inday`).
- Why: This preserves deterministic event flow, keeps debounce behavior intact, and makes DaySummary the only state mutating rollover outcomes.

## ADR-009 - Split Simulation Systems into Lane Modules

- Date: 2026-03-03
- Status: Accepted
- Context: A single `systems.rs` file became a merge hotspot and blocked multi-lane rotation throughput.
- Decision: Decompose systems into `src/game/systems/` modules (`bootstrap`, `input`, `tasks`, `interruptions`, `day_cycle`, `visuals`, `task_board`) with a re-exporting `mod.rs`.
- Why: This enables clearer ownership boundaries, lower merge conflict risk, and easier wave-by-wave audits.

## ADR-010 - Use Seeded Replay and Autoplay as Rotation Gates

- Date: 2026-03-03
- Status: Accepted
- Context: Deterministic behavior must be proven continuously before content scale-up.
- Decision: Add fixed-seed replay and seeded autoplay tests (`fixed_seed_three_day_replay_is_deterministic`, `five_day_seeded_autoplay_completes_without_panic`) as required early-rotation gates.
- Why: These tests make behavior drift visible immediately and provide reproducible regression signals for future waves.

## ADR-011 - Task Lifecycle Events and Terminal-State Guardrails

- Date: 2026-03-03
- Status: Accepted
- Context: Task progression was implicit and could not be audited as a first-class event flow.
- Decision: Introduce `TaskAccepted`, `TaskProgressed`, `TaskCompleted`, and `TaskFailed` events, and add `TaskBoard` lifecycle helpers (`complete_task`, `fail_task`) that reject terminal-state conflicts.
- Why: This provides explicit lifecycle semantics and prevents completed tasks from later failing in the same day.

## ADR-012 - Add Versioned Save Snapshot Skeleton Early

- Date: 2026-03-03
- Status: Accepted
- Context: Persistence invariants are a release-critical parity requirement, but durable slots are a later-wave concern.
- Decision: Land `OfficeSaveSnapshot` v1 capture/serialize/deserialize/apply scaffolding plus a DaySummary persistence hook, with identity-focused tests before full save-slot UX.
- Why: This de-risks persistence by validating core state fidelity (`TaskId` round-trip and no mid-day task regeneration) before broader save-system expansion.

## ADR-013 - Use Slot-Based File Persistence with Event-Driven Save/Load

- Date: 2026-03-03
- Status: Accepted
- Context: In-memory snapshot skeleton proved identity invariants but could not survive process restarts or validate migration behavior.
- Decision: Add file-backed slot persistence under `city_office_worker_dlc/saves` with `SaveSlotRequest`/`LoadSlotRequest` events, `SaveSlotConfig`, and migration-aware decoding (`migrate_snapshot_json` with `v0 -> v1` support).
- Why: Event-driven save/load keeps ordering deterministic within the simulation loop, while slot files and migration stubs provide durable state and forward-compatibility for future schema evolution.

## ADR-014 - Enforce Small Vertical Slices and Contract-Test Coupling

- Date: 2026-03-03
- Status: Accepted
- Context: Early origin history showed mega-commit integration churn, scope-drift commits, and delayed bug-fix bursts after large waves.
- Decision: For R5+ integration flow, enforce: no `WIP` commits, split slices above `~1,200` insertions or `>20` files, keep infra/build changes separate from gameplay/content, and require contract deltas to include wiring plus deterministic/headless tests in the same PR.
- Why: Smaller vertical slices reduce merge risk, make audits clearer, and surface contract drift immediately instead of in late stabilization passes.

## ADR-015 - Prioritize First-Seconds Stability and Deterministic Economy Hooks

- Date: 2026-03-03
- Status: Accepted
- Context: Early-session failures (duplicate singleton spawns, soft-lock risk in opening seconds) can invalidate all late-game work; R5 also needed concrete economy/progression depth without introducing nondeterminism.
- Decision: Make startup scene setup idempotent for worker/inbox/camera singletons, and introduce deterministic economy/progression primitives (salary curve with streak/burnout modifiers, XP leveling, and auto-assigned perk tracks).
- Why: This keeps the first playable seconds resilient while adding mid-run progression depth that remains replay-safe and testable.

## ADR-016 - Use Deterministic Task Content Packs Before Narrative Scale-Up

- Date: 2026-03-03
- Status: Accepted
- Context: Content scale needed to grow without introducing RNG drift or breaking replay assertions.
- Decision: Replace single-pattern task generation with deterministic template packs spanning all task kinds/priorities and day-based difficulty/economy scaling, then gate with explicit variety and scaling tests.
- Why: This increases gameplay variety and progression texture while preserving reproducibility required for long-horizon regression testing.

## ADR-017 - Model Social State as a Persisted Deterministic Graph

- Date: 2026-03-03
- Status: Accepted
- Context: R6 required social depth without compromising replay determinism or save/load fidelity.
- Decision: Introduce `SocialGraphState` with bounded `CoworkerProfile` affinity/trust values, deterministic interruption scenario selection (`seed + day + cursor`), and snapshot persistence for social graph state.
- Why: This creates a scalable social foundation while keeping behavior replay-safe and migration-friendly.
