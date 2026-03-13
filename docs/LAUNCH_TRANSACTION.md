# Launch Transaction

Purpose: make lane launch as mechanical as checkpoint and restore.

## Repo Mechanism

Use:

```bash
bash scripts/launch-lane.sh \
  --lane-id tranche_2_collision_lighting \
  --branch launch/tranche2-collision \
  --worktree /tmp/hearthfield-t2-collision \
  --codex-home /home/geni/.codex-runs/launch-t2-collision \
  --objective /home/geni/swarm/hearthfield/objectives/launch/tranche-2-collision-lighting.md
```

The wrapper does this:

1. ensures the isolated worktree exists
2. ensures the isolated `CODEX_HOME` exists
3. syncs the current launch docs/status/objectives into that worktree
4. starts the lane non-interactively in the pinned worktree
5. writes the run log and final-message path deterministically

## Why

The orchestration stack now has three mechanical pieces:

- checkpoint
- restore
- launch

That reduces the amount of workflow state that depends on memory or shell history.

## Current Limit

This wrapper launches the lane cleanly, but it does not yet mutate the dispatch ledger itself. The ledger update still happens in the calling control plane.
