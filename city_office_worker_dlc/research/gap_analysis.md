# Gap Analysis: `CONTRACT.md` vs current `src/game/*`

Scope checked: `CONTRACT.md` and:
- `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`
- `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`
- `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/components.rs`
- `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/events.rs`
- `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs`

## Top 10 gaps by impact

| # | Priority | Gap | Current implementation evidence | Exact file targets likely needed |
|---|---|---|---|---|
| 1 | P0 | Contract app state machine is missing (`OfficeGameState` + `InDay`/`DaySummary` semantics). | No `States` type exists; systems always run in one chained `Update` pipeline. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs` |
| 2 | P0 | Core resource model does not match contract (`OfficeClock`, `WorkerStats`, `TaskBoard`, `OfficeRunConfig`, `DayOutcome`). | Only `OfficeRules`, `InboxState`, `DayClock`, `DayStats` exist; contract resource names/fields are absent. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs` |
| 3 | P0 | Task domain value types are missing (`TaskId`, `TaskKind`, `TaskPriority`, `OfficeTask`, `InterruptionKind`, `ChoiceId`). | No task/interruption value types are defined anywhere in `src/game/*`. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/events.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs` |
| 4 | P0 | Event contract is not implemented (required task/interruption/day events + flow). | Existing events are `ProcessInboxEvent`, `CoffeeBreakEvent`, `WaitEvent`, `EndOfDayEvent`; none of `TaskAccepted/TaskProgressed/TaskCompleted/TaskFailed/Interruption* /EndDayRequested/DayAdvanced`. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/events.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs` |
| 5 | P0 | End-of-day transition contract is missing (`EndDayRequested` -> evaluation -> single `DayAdvanced`, rollover only in `DaySummary`). | Current flow only sets `clock.ended = true` and prints summary; no day advance event, no rollover state boundary. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs` |
| 6 | P1 | Required system set taxonomy and ordering (`OfficeSimSet`) are absent. | No `SystemSet` enum; one `.chain()` tuple is used instead of contract sets/order. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs` |
| 7 | P1 | Persistence contract is unimplemented (save/load required fields and invariants). | No save/load code or persistence module in `src/game/*`; no handling for TaskId round-trip or mid-day load. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/persistence.rs` (new) |
| 8 | P1 | Deterministic run config and interruption generation are missing (`seed`, `max_tasks_per_day`, hourly interruption chance). | `OfficeRunConfig` absent; no RNG-driven task generation/interruptions in systems. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs` |
| 9 | P1 | Required invariants are not enforced via dedicated systems (stat clamps, task progress bounds, duplicate TaskId rejection, completed-not-failed-later). | No normalization/rejection systems exist; current logic only clamps coffee energy locally. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/events.rs` |
| 10 | P2 | Component contract mismatch (`PlayerOfficeWorker`, `OfficeDesk`, `Interactable`, `NpcCoworker`, `NpcRole`) prevents shared vocabulary alignment. | Current components are `OfficeWorker`, `WorkerAvatar`, `InboxAvatar`; contract component set is absent. | `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/components.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs` |

## Smallest viable next wave

1. Establish contract types/resources/events first.
- Add `OfficeGameState`, `OfficeSimSet`, all contract resources/value types/events.
- Target files: `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/resources.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/events.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/components.rs`.

2. Implement minimal compliant simulation loop in `InDay`.
- Input -> time -> task progression -> interruption roll -> economy/state transition.
- Add dedicated normalization and task-ID uniqueness checks.
- Target file: `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs`.

3. Implement end-day state transition contract.
- `EndDayRequested` evaluation, rollover only in `DaySummary`, emit `DayAdvanced` once/day.
- Target files: `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/systems.rs`, `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`.

4. Add minimum verification + persistence skeleton.
- Add invariant/headless tests and minimal save/load surface preserving TaskIds and active tasks.
- Target files: `/home/geni/swarm/hearthfield/city_office_worker_dlc/tests/headless.rs` (new), `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/persistence.rs` (new), `/home/geni/swarm/hearthfield/city_office_worker_dlc/src/game/mod.rs`.
