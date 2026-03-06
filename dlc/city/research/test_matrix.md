# City Office Worker DLC Prototype - Test & Validation Matrix

Scope basis:
- Contract: `dlc/city/CONTRACT.md` (determinism/invariants/end-of-day expectations)
- Implementation: `dlc/city/src/game/resources.rs`, `dlc/city/src/game/systems.rs`

Current baseline (2026-03-03):
- `cargo check --manifest-path dlc/city/Cargo.toml` -> pass
- `cargo test --manifest-path dlc/city/Cargo.toml` -> pass (0 tests)

## Run Commands

```bash
# compile gate
cargo check --manifest-path dlc/city/Cargo.toml

# test gate (currently no automated tests yet)
cargo test --manifest-path dlc/city/Cargo.toml -- --nocapture

# manual validation run (use P/C/N keys)
RUST_LOG=info cargo run --manifest-path dlc/city/Cargo.toml
```

## Matrix

| ID | Area | Type | Test case (implementation-oriented) | Expected result |
|---|---|---|---|---|
| D1 | Deterministic simulation | Automated | Start defaults, apply events in exact order: `P, P, P, C, N(10), P`. | Exact final state every run: time `10:30`, energy `77`, inbox `14`, processed `4`, coffee `1`, waits `1`, failed `0`, `ended=false`. |
| D2 | Deterministic simulation | Automated | Set energy `10`, inbox `18`, send one `ProcessInboxEvent`. | Time still advances to `09:15`; energy remains `10`; inbox remains `18`; `failed_process_attempts += 1`. |
| D3 | Deterministic simulation | Automated | Set `DayClock.current_minute = u32::MAX - 5`, send `WaitEvent { minutes: 10 }`. | Clock saturates to `u32::MAX`; no panic (uses `saturating_add`). |
| I1 | Invariant | Automated | Repeatedly process from default until energy low. | Energy never < `0`; with defaults, 8 successes then failures at energy `4` (time still advances on failed process). |
| I2 | Invariant | Automated | Set energy `90`, send coffee event. | Energy clamps to `100` (not `115`); coffee count increments by `1`; time +20 min. |
| I3 | Invariant | Automated | Send `WaitEvent { minutes: 0 }`. | Wait clamps to minimum `1` minute; `wait_actions += 1`. |
| I4 | Invariant | Automated | Force `clock.ended = true`, then send process/coffee/wait events. | No state mutation from these handlers (time/stats/inbox/energy unchanged). |
| I5 | Contract alignment | Manual/Review | Compare default day bounds to contract. | Contract expects `08:30-18:00`; current impl is `09:00-17:00` -> drift flagged for decision. |
| H1 | Input handling | Manual | Press `P` once, then hold `P`. | One process action per key down edge (`just_pressed`), not per frame while held. |
| H2 | Input handling | Manual | Press `C` once and `N` once. | `C` triggers coffee (+20 min, +energy clamp), `N` triggers wait (+10 min by default). |
| H3 | Input handling | Manual/Automated | Trigger `P` and `C` in same frame. | Deterministic chain order: process first, then coffee. From defaults -> time +35 min, inbox `17`, energy ends at `100`. |
| E1 | End-of-day logic | Automated | Set clock `16:59`, inbox > 0, send wait `1`. | `EndOfDayEvent` emitted once with `finished_minute=17:00`; `clock.ended=true`. |
| E2 | End-of-day logic | Automated | Set inbox `1`, energy >= 12, process once at `09:00`. | Day ends early at `09:15` with `remaining_items=0`, `processed_items=1`. |
| E3 | End-of-day logic | Automated | After E1/E2, call update/check systems additional frames. | No second end-of-day event for same day (`clock.ended` guard). |
| E4 | End-of-day logic | Automated | Inspect emitted summary fields after controlled scenario. | Event fields match `DayStats` + worker energy exactly (no recomputation drift). |

## Suggested Minimum First Automation Batch

1. Add headless tests for `D1`, `I2`, `I3`, `E2`, `E3`.
2. Keep `H1/H2` as manual smoke until input simulation harness is added.
3. Treat `I5` as contract-gate check before adding persistence/economy layers.
