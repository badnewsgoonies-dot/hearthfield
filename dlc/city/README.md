# City Office Worker DLC Prototype

A small, vertical-slice DLC prototype using the same stack as the main project (`Rust 2021 + Bevy 0.15`).

## Prototype Goal

Build a playable office-day loop that is fun within 10-15 minutes and technically stable enough to expand.

Success criteria:
- Player can complete at least 3 in-game days.
- Each day has task intake, execution, interruptions, and end-of-day summary.
- Day outcomes affect next-day difficulty and payout.

## Core Gameplay Loop

1. Start day (08:30) at the city office desk.
2. Pull tasks from the board/inbox.
3. Complete tasks before deadlines while managing `Energy`, `Stress`, and `Focus`.
4. Handle interruptions (meetings, urgent requests, system outages).
5. Submit end-of-day report.
6. Receive salary/reputation changes, unlock next day modifiers, repeat.

## Controls (Prototype Defaults)

- `W/A/S/D` or arrow keys: move in office map.
- `E`: interact (desk, task board, NPC, coffee machine).
- `Tab`: open/close task board.
- `1/2/3`: choose response during interruption dialogs.
- `Space`: confirm selected action.
- `Esc`: pause menu.
- `F5`: restart current day (debug/dev convenience).

## Run

The prototype runs as a separate crate in this folder.

From repo root:

```bash
# run the DLC prototype
cargo run --manifest-path city_office_worker_dlc/Cargo.toml

# quick compile check
cargo check --manifest-path city_office_worker_dlc/Cargo.toml

# run DLC tests
cargo test --manifest-path city_office_worker_dlc/Cargo.toml
```

## Target Module Layout

```text
city_office_worker_dlc/
  Cargo.toml
  src/
    main.rs
    shared/
    office_time/
    tasks/
    interruptions/
    economy/
    ui/
    save/
```

## MVP Feature Slice

- Office clock with day progression.
- Task generation with deadlines and rewards.
- Interruption system with 2-3 choice outcomes.
- Basic NPC interactions (manager + coworker).
- End-of-day summary and persistence for next day.

For implementation detail, use `CONTRACT.md`. For parallel execution strategy and quality gates, use `ORCHESTRATION_PLAN.md`.
