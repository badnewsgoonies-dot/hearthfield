# Parity Baseline Matrix (R0)

Date: 2026-03-03
Scope: origin game at `/home/geni/swarm/hearthfield` vs DLC at `/home/geni/swarm/hearthfield-office-dlc/city_office_worker_dlc`

## Measurement Snapshot

| Dimension | Origin game | City Office Worker DLC | Delta |
|---|---:|---:|---:|
| Rust source lines (`src/**/*.rs`) | 38,629 | 1,978 | -36,651 |
| Total discovered tests (`cargo test -- --list`) | 375 | 8 | -367 |
| Domain folders under `src/` | 15 (`animals`, `calendar`, `crafting`, `data`, `economy`, `farming`, `fishing`, `input`, `mining`, `npcs`, `player`, `save`, `shared`, `ui`, `world`) | 1 (`game`) with modular lanes under `src/game/systems/**` | major gap |
| Clippy hard gate (`-D warnings`) | FAIL (existing dead code / argument-count debt) | PASS | DLC stricter than origin |
| Deterministic replay tests | partial | PASS (`fixed_seed_three_day_replay_is_deterministic`) | DLC green |
| Seeded autoplay no-panic tests | not baseline-gated | PASS (`five_day_seeded_autoplay_completes_without_panic`) | DLC green |

## OES-v1 Parity Definition (Execution Target)

The DLC reaches Origin-Equivalent State v1 when:

1. Architecture parity: domainized module layout equivalent to origin breadth (`time`, `tasks`, `interruptions`, `economy`, `social`, `save`, `ui`, `world`).
2. Testing parity floor: 200+ automated tests with deterministic multi-day and autoplay suites.
3. Stability parity: `cargo check`, `cargo test`, `cargo clippy -D warnings` all pass in DLC.
4. Process parity: contract-first wave packets, ADR trail, and evidence-backed gate dashboard per rotation.
5. Gameplay parity floor: complete office simulation loop with progression, social pressure, persistence, and content packs.

## Baseline Observations

1. The largest parity gap is breadth (domain count and content volume), not core runtime wiring.
2. The DLC is currently ahead of origin in lint strictness but far behind in system and test volume.
3. R1 and R2 closure should prioritize module decomposition + deterministic infrastructure before content explosion.
