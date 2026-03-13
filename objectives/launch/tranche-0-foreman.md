# Foreman: Tranche 0 Control Plane

## Scope

You may only modify files under:

- `status/launch/`
- `status/foreman/`
- `objectives/launch/`

Do not edit `src/`, `assets/`, `tests/`, or the shared contract.
Do not create generic orchestration infrastructure beyond the concrete launch artifacts listed below.

## Required Reading (in this order)

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `status/foreman/dispatch-state.yaml`
4. `docs/ORCHESTRATION_TOPOLOGY.md`
5. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
6. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
7. `status/research/runtime_surface_manifest.csv`
8. `status/research/runtime_used_asset_manifest.csv`
9. `status/research/visual_mapping_manifest.csv`
10. `status/research/test_baseline_manifest.csv`

## Mission

Turn the current baseline into execution packets for the first real tranche.

This is not a coding tranche.
It is the control-plane launch tranche.

## Deliverables

Create or update:

1. `status/launch/tranche-0-report.md`
   - what was read
   - what tranche 1 will cover
   - what tranche 1 will explicitly not cover
   - active risks/confounds

2. `objectives/launch/tranche-1-player-day-start.md`
   - one bounded execution packet for boot/spawn/house-exit/sleep

3. `objectives/launch/tranche-1-farm-loop.md`
   - one bounded execution packet for the day-one farming loop

4. `status/foreman/dispatch-state.yaml`
   - update tranche 0 to completed
   - mark tranche 1 as ready
   - add the two tranche-1 lane entries with owned paths, goals, next actions, and blank validation fields

## Constraints

- Do not create more than the two tranche-1 packets.
- Keep tranche-1 packets narrow and execution-first.
- No broad theory recap.
- No audit-only drift.
- Each packet must name:
  - owned paths
  - required reading
  - exact target surfaces
  - explicit non-goals
  - validation commands
  - required report path

## Validation

Before reporting done:

- `git diff --name-only` must show only `status/launch/`, `status/foreman/`, and `objectives/launch/`
- the updated YAML must remain parseable by eye and structurally consistent

## When Done

Return:

- files changed
- tranche-1 lane names
- owned paths
- immediate next command to run tranche 1
