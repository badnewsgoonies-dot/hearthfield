# R5 Social/Progression Parity Decomposition Packet

Date: 2026-03-03  
Scope: convert current R5 foundation into executable R6 lane work without contract drift.

## 1) Current Surface (What Exists)

Implemented and deterministic today:
1. Social pressure loop primitives: interruption, manager check-in, coworker help, calm/panic resolution.
2. Progression primitives: XP, level, streak, burnout days, deterministic auto-perk tracks.
3. Economy linkage: salary/reputation/stress deltas now include level/streak/burnout/perk modifiers.
4. Persistence linkage: progression state is captured in save snapshots and restored on load.

## 2) Parity Gaps vs Origin-Equivalent Intent

High-priority gaps:
1. No persistent coworker roster or per-NPC affinity states.
2. No manager arc stages (trust/performance flags, escalating check-ins, branch outcomes).
3. No social-content library (event variants, dialogue/event outcomes by relationship state).
4. No progression unlock catalog (perk selection UI, milestone rewards, unlockable actions/tasks).
5. No cross-day social memory beyond aggregate counters.

## 3) R6 Lane Decomposition

Lane `SOC-STATE`:
1. Add `CoworkerProfile` model and `SocialGraphState` resource.
2. Track affinity/trust deltas per day and normalize bounds.
3. Add save/load coverage for social state.

Lane `SOC-CONTENT`:
1. Introduce deterministic social scenario templates (manager/coworker variants).
2. Map response choices to affinity/stress/reputation outcomes.
3. Add invariant tests for deterministic scenario selection by seed/day.

Lane `PROG-UNLOCKS`:
1. Add explicit progression unlock table (level thresholds -> actions/perks/tasks).
2. Add tests for unlock determinism and replay consistency.
3. Ensure unlock state is persisted and migration-safe.

## 4) Executable Checks (Must Pass)

1. `cargo check --manifest-path city_office_worker_dlc/Cargo.toml`
2. `cargo test --manifest-path city_office_worker_dlc/Cargo.toml`
3. `cargo clippy --manifest-path city_office_worker_dlc/Cargo.toml --all-targets -- -D warnings`

Required new test classes for R6 entry:
1. Per-coworker affinity round-trip persistence.
2. Deterministic social scenario replay across 10+ days.
3. Unlock gating determinism (`same seed + same action script => same unlock timeline`).

## 5) Guardrails for R6 Dispatch

1. No `WIP` commits on integration branch.
2. Keep each slice vertically complete (contract/resource/event + systems + tests).
3. Split slices above `~1,200` insertions or `>20` files.
4. Keep infra/build changes outside gameplay/social slices.
5. Do not mark R6 complete without persistence + deterministic replay evidence.
