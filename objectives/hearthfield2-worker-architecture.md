# Hearthfield 2.0 Worker — Architecture Wave

## Mission
Introduce foundational scheduling architecture improvements using Bevy 0.15 patterns with minimal behavior drift.

## Required changes
1. SystemSet taxonomy
- Add shared SystemSet phases:
  - Input
  - Intent
  - Simulation
  - Reactions
  - Presentation
- Define in shared module(s) and configure in `main.rs` for `Update` schedule.

2. Incremental adoption in selected domains
- Assign existing systems into sets where obvious and low-risk:
  - `input`
  - `player`
  - `ui` (core update systems)
  - `world` (movement/presentation where safe)
- Remove fragile ad-hoc ordering only where replaced by clear set ordering.

3. FixedUpdate island (small, safe)
- Move one deterministic simulation subsystem into `FixedUpdate` with stable timestep config.
- Candidate: animal wandering OR NPC movement cadence (pick the safer one).
- Ensure gameplay remains correct.

4. Keep compatibility
- No large state-machine rewrite in this wave.
- No shared module mega-split required here.

## Constraints
- Minimize churn outside targeted scheduling surfaces.
- Keep compile/test green.

## Validation
Run and pass:
- `cargo check`
- `cargo test --test headless`

## Deliverables
1. SystemSet taxonomy and configured schedule.
2. Selected domain systems migrated to sets.
3. One fixed-step simulation island implemented.
4. Worker report:
- `status/workers/hearthfield2-worker-architecture-report.md`
