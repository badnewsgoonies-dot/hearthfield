# Worker: [NAME]

## Scope (hard allowlist — enforced mechanically, not by judgment)
You may only modify files under: [allowed_prefix_1] [AND allowed_prefix_2 if needed]
All out-of-scope edits will be reverted after you finish.
Do NOT edit the shared contract unless this task has been explicitly promoted to integration work.
Do NOT create orchestration infrastructure. Implement only the scoped deliverables.

## Required reading (in this order)
1. `docs/spec.md`
2. `[docs/domains/domain.md or other disk spec file]`
3. `[main game: src/shared/mod.rs | pilot: dlc/pilot/src/shared/mod.rs | city: relevant shared type files such as dlc/city/src/game/resources.rs and dlc/city/src/game/events.rs]`
4. `[any additional repo files the worker must read before editing]`

## Required imports (use these exactly, do not redefine locally)
- `[List exact shared types, enums, events, resources, constants, or APIs]`

## Deliverables
- `[List files to modify or create]`
- `[List behaviors, fixes, exports, or tests to implement]`

## Quantitative targets (non-negotiable)
- `[Explicit counts]`
- `[Constants and formulas with literal values]`
- `[Tables, lists, thresholds, or enumerated detail that must be preserved]`

## Validation (run before reporting done)
```bash
[crate-appropriate command 1]
[crate-appropriate command 2]
[crate-appropriate command 3]
```

Done = every validation command passes, no required tests are skipped, and the deliverables and quantitative targets are met.

## Mandatory lane closeout
After the worker completes, the orchestrator/foreman must:

1. Clamp scope mechanically with `bash scripts/clamp-scope.sh [allowed_prefix_1] [allowed_prefix_2...]`
2. Re-run the validation commands on the clamped result
3. Review the clamped diff only

If the clamped result is empty or no longer valid, discard the run and relaunch.
Do not ask for acceptance on an unclamped bounded-lane result.

## When done
Write completion report to `status/workers/[name].md` containing:
- Files created or modified
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail plus counts where relevant)
- Assumptions made
- Known risks or open integration items
