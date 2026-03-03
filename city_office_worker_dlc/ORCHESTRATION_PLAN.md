# City Office Worker DLC - Orchestration Plan

This plan applies the orchestration findings from `docs/model is the orchestrator draft 13.docx` to a practical DLC build workflow.

## Operating Principles

1. Mechanical scope enforcement over prompt trust.
- Workers may compile globally, but are only allowed to keep edits inside owned files.
- After each worker run, revert out-of-scope edits automatically.
- Why: prompt-only scope fails under compiler pressure; mechanical enforcement is reliable.

2. Shared contract is the vocabulary.
- `CONTRACT.md` is the canonical language for states, resources, components, and events.
- Workers implement against contract names, not local synonyms.
- Any contract rename must be approved in `CONTRACT.md` before code changes.

3. Disk is source of truth.
- Coordination state lives in files, not chat memory.
- Minimum docs in this folder:
  - `README.md` (product/usage)
  - `CONTRACT.md` (shared vocabulary)
  - `ORCHESTRATION_PLAN.md` (execution policy)
  - `STATUS.md` (wave status and blockers)
  - `TASKS.md` (owned tasks + file allowlists)
  - `DECISIONS.md` (architecture decisions)

4. Stabilize in waves.
- Ship in short waves with hard, measurable gates.
- No wave advances if gates fail.

5. Continuous dual-track loop.
- Keep 1-3 investigation lanes active while implementation lanes run.
- Investigation outputs feed the next wave's task packets and acceptance gates.
- Integrator lane merges outputs and updates contract/status docs before next dispatch.

## Contract Bible First (Orchestrator Responsibility)

As orchestrator, create/freeze the type contract bible (`CONTRACT.md`) first, then dispatch waves.
- No lane starts without a contract section reference.
- Lane packets must cite exact contract sections and owned files.
- Contract drift is treated as a blocker, not a warning.

## Wave Pattern (from early Hearthfield sequencing)

Use this repeatable pattern for each milestone:
1. `C0 Contract Freeze`: lock shared vocabulary and invariants.
2. `W1 Parallel Stubs`: worker lanes scaffold module surfaces against contract.
3. `W2 Feature Waves`: lane-specific implementation in short bursts.
4. `I1 Integration`: integrator lane resolves seams and enforces naming.
5. `H1 Hardening`: bug-fix/audit pass under compiler/test pressure.
6. `T1 Test Expansion`: add deterministic and regression coverage for new behavior.
7. `P1 Polish`: UX/visual/content pass after stability gates are green.

## Mechanical Scope Workflow

Per worker task:

1. Define file allowlist in `TASKS.md`.
2. Worker executes implementation and local checks.
3. Run scope guard:

```bash
# from repo root (project-specific helper)
./city_office_worker_dlc/tools/scope_guard.sh --allow-file city_office_worker_dlc/TASKS.md
```

If helper script is not available, equivalent fallback:

```bash
# example fallback: revert only files outside allowlist
# (replace ALLOWED_* values with current owned paths)
ALLOWED_1='city_office_worker_dlc/src/tasks/'
ALLOWED_2='city_office_worker_dlc/src/shared/'

for f in $(git diff --name-only); do
  case "$f" in
    "$ALLOWED_1"*|"$ALLOWED_2"*) ;;
    *) git restore --staged --worktree "$f" ;;
  esac
done
```

4. Re-run compile/tests.
5. Integrator lane merges and verifies contract alignment.
6. Only then mark task done in `STATUS.md`.

## Stabilization Waves

### Wave 0 - Skeleton and Contract Wiring

Scope:
- Create crate skeleton.
- Register app states and plugin order.
- Add contract stubs for resources/events/components.

Gate:
- `cargo check --manifest-path city_office_worker_dlc/Cargo.toml` passes.
- No out-of-scope diff after scope guard.

### Wave 1 - Playable Core Day Loop

Scope:
- Office clock + day transitions.
- Task intake/execution/completion.
- End-of-day summary with payout and stat updates.

Gate:
- One full day playable without panic.
- 3 deterministic headless day simulations pass.

### Wave 2 - Interruptions and NPC Pressure

Scope:
- Interruption events with branching outcomes.
- Manager/coworker interactions that modify stress/reputation.

Gate:
- At least 5 interruption scenarios reachable.
- Contract event flow verified by headless tests.

### Wave 3 - Persistence and Balance Pass

Scope:
- Save/load for core resources.
- Economy tuning (salary, penalties, overtime rewards).

Gate:
- Save/load round-trip keeps all core values unchanged.
- 5-day autoplay does not deadlock and stays within stat invariants.

### Wave 4 - Hardening and Release Candidate

Scope:
- UI polish, error handling, content cleanup.
- Performance sanity pass and bug triage.

Gate:
- `cargo test --manifest-path city_office_worker_dlc/Cargo.toml` passes.
- `cargo clippy --manifest-path city_office_worker_dlc/Cargo.toml -- -D warnings` passes.
- No P0/P1 known issues in `STATUS.md`.

## Measurable Gates Dashboard

Track these per wave in `STATUS.md`:

- `G1 Compile`: check result (`pass/fail`).
- `G2 Scope`: out-of-scope files touched (`count`, target `0`).
- `G3 Loop`: completed day simulations (`count`, target per wave).
- `G4 Contract`: event/resource drift violations (`count`, target `0`).
- `G5 Stability`: panic count in test/autoplay runs (`target 0`).
- `G6 Quality`: clippy warnings and failing tests (`target 0`).

## Worker Lane Ownership (Suggested)

- Lane A: `office_time` + state transitions.
- Lane B: `tasks` + task resolution.
- Lane C: `interruptions` + dialog outcomes.
- Lane D: `economy` + progression.
- Lane E: `ui` + HUD/day summary.
- Lane F: `save` + persistence.

All lanes must import shared vocabulary from `CONTRACT.md`; cross-lane coupling is only through defined events/resources.
