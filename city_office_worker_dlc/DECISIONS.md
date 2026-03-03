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
