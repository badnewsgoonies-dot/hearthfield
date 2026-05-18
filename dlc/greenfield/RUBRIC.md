# Greenfield → Hearthfield-DLC Validation Rubric

The goal of the substrate iterations is to push Greenfield from "empty
shell that compiles" toward "real, integrated Hearthfield DLC". Each
iteration should advance the score below. If an iteration doesn't move
the score, it isn't a step toward the goal — it's drift.

## Scoring (0-100 total)

Each criterion is checked at the END of the iteration via `cargo check`
output, file inspection, or the chain envelope.

### Foundation (max 20)

- [F1] greenfield_dlc compiles green (cargo check rc=0)            10 pts
- [F2] Greenfield is in `[workspace] members` of root Cargo.toml    5 pts
- [F3] Bevy version + feature set matches host                     5 pts

### Sibling-DLC parity with city (max 30)

- [S1] serde + serde_json deps present                              5 pts
- [S2] ironclad path dep present                                    5 pts
- [S3] Coherent state machine (<= 8 thematic states, no
       BigBatch/McpScale/Variant noise)                            10 pts
- [S4] SystemSet ordering configured via `.configure_sets(...)`
       with `.chain()`                                              5 pts
- [S5] Cargo `description` field identifies it as Hearthfield DLC   5 pts

### Theme coherence (max 20)

- [T1] README documents theme + gameplay loop + position in
       Hearthfield universe                                         5 pts
- [T2] Window title mentions Hearthfield                            3 pts
- [T3] At least one `#[game_lifecycle(...)]` typestate matching
       the host's lifecycle_types.rs pattern                        7 pts
- [T4] Clear color / visual theme matches Hearthfield's farming
       palette (greens, earth tones)                                5 pts

### Integration with host (max 30) — the actual stretch goal

- [I1] Cargo path dep: `hearthfield = { path = "../.." }`           5 pts
- [I2] At least one `use hearthfield::shared::...` import           5 pts
- [I3] Greenfield uses `UpdatePhase` from `shared::schedule` instead
       of redefining its own `GreenfieldSet`                        5 pts
- [I4] Greenfield consumes at least one shared event (e.g.
       `DayEndEvent`, `ToolUseEvent`, `CropHarvestedEvent`)         5 pts
- [I5] Greenfield writes at least one shared event (e.g. emits
       a `CropHarvestedEvent` when a Mature crop is harvested)      5 pts
- [I6] Greenfield reads at least one shared resource (e.g.
       `Calendar`, `PlayerState`, or `Inventory`)                   5 pts

## Iteration scorecard

| ver  | foundation | sibling | theme | integration | total | what it changed |
|------|------------|---------|-------|-------------|-------|-----------------|
| b0   | 15 / 20    |  0 / 30 |  0/20 |  0 / 30     | 15    | baseline: empty Greenfield shell |
| b9   | 20 / 20    |  5 / 30 |  3/20 |  0 / 30     | 28    | gameplay slice, no theme/deps |
| b11  | 20 / 20    | 30 / 30 | 20/20 |  0 / 30     | 70    | sibling parity + theme — but no host integration |
| b12  | 20 / 20    | 30 / 30 | 20/20 |  0 / 30     | 70    | red — env-missing alsa-sys / libudev-sys (chain audit captured) |
| b13  | 20 / 20    | 30 / 30 | 20/20 | 25 / 30     | 95    | I1+I2+I4+I5+I6 land green; only I3 (UpdatePhase) left |
| b14  | 20 / 20    | 30 / 30 | 20/20 | 30 / 30     | **100** | I3: GreenfieldSet dropped, UpdatePhase from shared adopted |

**Greenfield is fully validated as a real Hearthfield DLC.** Every
criterion in the rubric is met. The chain has the full audit trail
from b0 (empty shell) to b14 (full integration).

## Iteration discipline rule

If an iteration's authored cork program won't move at least one
unchecked rubric box, the iteration is not authored. The substrate
chain is small. We don't burn an iteration on drift.

This file is authoritative. The substrate reads it as `extra_context`
when authoring the next cork program, and the score after each
cargo-check verdict is recorded back in the same row.
