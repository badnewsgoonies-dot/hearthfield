# Documentation Stack

Use this to understand which document answers which question.

## Entry Points

1. [QUICK_START.md](/home/geni/swarm/hearthfield/docs/QUICK_START.md)
   - how to begin a session quickly
2. [HEARTHFIELD_BUILD_METHODOLOGY.md](/home/geni/swarm/hearthfield/docs/HEARTHFIELD_BUILD_METHODOLOGY.md)
   - how to operate during the session
3. [THREE_PRINCIPLES_OF_ORCHESTRATED_BUILDS.md](/home/geni/swarm/hearthfield/docs/THREE_PRINCIPLES_OF_ORCHESTRATED_BUILDS.md)
   - why the orchestration model is shaped this way

## Doctrine

- [UNIVERSAL_SOFTWARE_KERNEL.md](/home/geni/swarm/hearthfield/docs/UNIVERSAL_SOFTWARE_KERNEL.md)
  - software-wide doctrine
- [UNIVERSAL_GAME_KERNEL.md](/home/geni/swarm/hearthfield/docs/UNIVERSAL_GAME_KERNEL.md)
  - game-specific doctrine
- [HEARTHFIELD_OPERATING_KERNEL.md](/home/geni/swarm/hearthfield/docs/HEARTHFIELD_OPERATING_KERNEL.md)
  - Hearthfield-specific adapter

## Procedure

- [SUB_AGENT_PLAYBOOK.md](/home/geni/swarm/hearthfield/docs/SUB_AGENT_PLAYBOOK.md)
  - Tier `M` / `C` multi-worker procedure
- [BOUNDED_LANE_FLOW.md](/home/geni/swarm/hearthfield/docs/BOUNDED_LANE_FLOW.md)
  - mandatory post-run clamp/validate/review flow

## Orchestration Infrastructure

- [CHECKPOINT_TRANSACTION.md](/home/geni/swarm/hearthfield/docs/CHECKPOINT_TRANSACTION.md)
- [LAUNCH_TRANSACTION.md](/home/geni/swarm/hearthfield/docs/LAUNCH_TRANSACTION.md)
- [ORCHESTRATION_TOPOLOGY.md](/home/geni/swarm/hearthfield/docs/ORCHESTRATION_TOPOLOGY.md)

## Live Truth

These change often and outrank stale prose:

- [.memory/STATE.md](/home/geni/swarm/hearthfield/.memory/STATE.md)
- [dispatch-state.yaml](/home/geni/swarm/hearthfield/status/foreman/dispatch-state.yaml)
- tranche reports in [status/launch](/home/geni/swarm/hearthfield/status/launch)
- worker reports in [status/workers](/home/geni/swarm/hearthfield/status/workers)

## Important Rule

If a stable doctrine doc disagrees with live state, code, tests, or a verified
report, the live verified source wins.
