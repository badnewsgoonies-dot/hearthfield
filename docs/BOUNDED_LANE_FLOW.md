# Bounded Lane Flow

Purpose: define the mandatory execution flow for any bounded worker or foreman lane in this repo.

This is the standard operating rule for scoped execution.
It exists so scope control is mechanical and repeatable, not remembered ad hoc in chat.

## Rule

For any bounded lane with owned paths:

1. run the lane
2. clamp scope mechanically
3. rerun validation on the clamped result
4. review only the clamped result
5. accept or relaunch

This is mandatory.

## Canonical Sequence

```text
lane runs
-> clamp
-> rerun required validation
-> review clamped result
-> accept or relaunch
```

Not:

```text
lane runs
-> review raw diff
-> notice drift
-> try to reason about whether it is acceptable
```

## Why

Prompt-only scope control is weak under pressure.
Bounded lanes are only trustworthy after mechanical scope enforcement.

The repo already has the mechanism:

- `scripts/clamp-scope.sh`

This document makes the follow-through explicit.

## Type Discoveries Before Acting

Unexpected findings inside a lane are not all the same thing. Type them first:

- Current-base reproducible runtime bug inside owned scope:
  fix it and add or strengthen the regression.
- Fragile seam with missing direct coverage:
  add the regression first. Only change gameplay code if that regression proves a real failure.
- Fidelity gap:
  escalate or backlog it unless the tranche explicitly owns that behavior.
- UX friction:
  record it unless the tranche explicitly owns usability or polish for that surface.
- Comment drift, naming drift, or doc mismatch:
  report it unless it directly impedes the tranche.
- Out-of-scope issue:
  record it and do not widen the lane ad hoc.

`Reproducible` means a named command, test, or exact observed path, not surprise or suspicion.

## Acceptance Rule

A bounded-lane result is reviewable only if:

- it has been clamped
- it still passes its lane validation after clamp

If clamping empties the useful work or breaks the fix:

- do not salvage by hand
- treat the run as invalid
- relaunch with tighter instructions or re-scope as integration work

If a lane makes an in-scope fix after finding a real bug or fragile seam:

- clamp the result
- rerun the required validation, including the strengthened regression
- review only the clamped post-validation result

## When To Relaunch

Relaunch the lane if any of these happen:

- out-of-scope edits dominate the useful work
- the clamped result is empty
- the clamped result fails the required validation
- the lane drifted into audit-only mode and never produced a usable scoped result

## Review Rule

Review:

- the clamped diff
- the validation results after clamp
- the lane report

Do not review:

- the raw unclamped lane output as if it were mergeable

## Relationship To Other Docs

- `AGENTS.md` states the repo-level rule
- `objectives/TEMPLATE.md` states the worker-packet rule
- this file is the dedicated operational reference

If there is any disagreement, the stricter interpretation wins.
