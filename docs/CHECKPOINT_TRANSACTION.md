# Checkpoint Transaction

Purpose: turn orchestration checkpointing from a careful manual ritual into one verified operation.

## What A Checkpoint Preserves

A usable orchestration checkpoint is a compound recovery object:

1. control-plane state
   - a forked Codex conversation/session
2. filesystem state
   - the exact worktree, branch, commit, and dirty diff state
3. ledger state
   - `status/foreman/dispatch-state.yaml`
   - `AGENTS.md`
   - `.memory/STATE.md`
   - worker reports and tranche notes

Preserving only one of those is partial recovery.

## Repo Mechanism

Use:

```bash
bash scripts/checkpoint-state.sh \
  --label tranche1-clean \
  --session 019ce4dc-9c4f-7250-a23f-bc539b7d90d1 \
  --worktree /tmp/hearthfield-t1-farm \
  --allow-prefix src/farming \
  --allow-prefix src/economy/gold.rs \
  --allow-prefix src/economy/shipping.rs \
  --allow-prefix src/player/tools.rs \
  --allow-prefix tests/headless.rs \
  --allow-prefix status/workers/tranche-1-farm-loop.md \
  --strict-untracked \
  --ledger status/foreman/dispatch-state.yaml
```

The wrapper does this in order:

1. preflight and fail closed
2. run `codex fork` in the exact worktree, not through a picker
3. bound the child session and let it record the preserved-fork acknowledgement
4. verify the new session exists and did not execute commands
5. write a checkpoint manifest under `status/checkpoints/`
6. append the checkpoint to `status/foreman/checkpoints.yaml`

## Current Guarantees

The MVP wrapper guarantees:

- exact worktree pinning
- no interactive directory picker
- snapshot session id recorded automatically
- manifest written automatically
- checkpoint ledger updated automatically
- no post-fork commands allowed in the child session
- optional allowlist enforcement for lane-owned tracked diffs
- optional strict enforcement for untracked files outside the allowlist
- transaction lock so two checkpoints cannot run at once
- `--dry-run` preflight mode

## Dry Run

Use this when you want the checkpointability verdict without creating a fork:

```bash
bash scripts/checkpoint-state.sh \
  --label tranche1-farm-preflight \
  --session 019ce4dc-9c4f-7250-a23f-bc539b7d90d1 \
  --worktree /tmp/hearthfield-t1-farm \
  --allow-prefix src/farming \
  --allow-prefix tests/headless.rs \
  --allow-prefix status/workers/tranche-1-farm-loop.md \
  --strict-untracked \
  --dry-run
```

## Smoke Test

The repo includes a dry-run smoke test for the wrapper:

```bash
bash scripts/test-checkpoint-state.sh
```

It exercises:

- successful dry-run preflight
- failure on tracked files outside the allowlist
- failure on untracked files outside the allowlist when `--strict-untracked` is enabled

## Where This Can Leapfrog Next

This checkpoint primitive is useful beyond preserving one good lane:

- before risky integration or merge work
- before launching variants from a proven foreman run
- before offloading orchestration to another machine
- before widening a tranche into a more expensive surface
- before test-expansion or hardening passes that might otherwise blur a clean baseline

The important part is that the checkpoint is now mechanical. We can use it as a reusable safety rail instead of a manual ritual.

## Restore

Use:

```bash
bash scripts/restore-checkpoint.sh \
  --manifest status/checkpoints/20260313T043737Z-tranche1-farm-clean-v7.yaml
```

Or resolve by label:

```bash
bash scripts/restore-checkpoint.sh --label tranche1-farm-clean-v7
```

By default this verifies:

- worktree path
- branch and head commit
- tracked/untracked diff hashes
- snapshot session file hash
- `AGENTS.md`
- `.memory/STATE.md`
- `dispatch-state.yaml`

If verification passes, it prints the exact `codex resume` and `codex exec resume` commands.

If you want the script to reopen the checkpoint directly:

```bash
bash scripts/restore-checkpoint.sh \
  --manifest status/checkpoints/20260313T043737Z-tranche1-farm-clean-v7.yaml \
  --resume-interactive
```

There is also a smoke test:

```bash
bash scripts/test-restore-checkpoint.sh
```

## Current Limits

The Codex CLI does not yet expose a native non-interactive `checkpoint` primitive.

So the repo wrapper still has to drive `codex fork` indirectly and verify the resulting session file. That is much safer than manual handling, but it is still a wrapper around current CLI behavior, not a first-class platform feature.
