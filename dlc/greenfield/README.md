# Greenfield DLC

A Hearthfield DLC sibling crate. Theme: **tower-defense farming survival**.
The player tends a small green field and defends growing crops from
critters that drift in to eat them.

## Build & run

```
cargo run -p greenfield_dlc
```

## Controls

- `WASD` move the farmer around the field
- `Space` attack — strike the nearest critter (3 hits to dispatch)
- `Esc` pause

## Gameplay loop

- Crops grow through stages: `Seed -> Sprout -> Sapling -> Mature -> Harvest`
  (typed transitions via `ironclad::game_lifecycle` — same lifecycle
  pattern as Hearthfield's tool, soil, and animal progressions)
- Critters spawn at the edges and drift toward the nearest crop
- When a critter touches the farmer, the farmer takes 5 HP damage
  (with a 0.5s hit cooldown so it doesn't spam)
- Attacking a critter accumulates damage; 3 hits resolves combat
  and drops loot + XP + score, despawning the critter
- HUD shows live HP / Score / Level / XP

## Append-only removal replay

Addressed waves, enemy removals, and restorations form one append-only history.
`W` appends `WaveSpawned(seed, generation)` before spawning; combat appends an
`EnemyRemoved` receipt with the exact stable enemy key and full visual transform;
`U` appends `EnemyRestored` with a new stable key and never erases the removal.

The cold replayer certifies a deliberately bounded state quotient: live enemies
introduced by recorded addressed waves, plus witnessed restorations, including
their stable keys, wave parents, transforms, sprite colours, sizes, and flip flags.
It does not claim to reconstruct timing, player input, Bevy entity handles, or
external enemies that were never removed. Torn receipt sequences, mismatched
removal snapshots, unknown restorations, and duplicate restorations refuse by name.

## Position in the Hearthfield universe

Greenfield is a workspace sibling of `hearthfield`, `dlc/city`,
`dlc/lifeline`, `dlc/pilot`, and `dlc/police`. It shares the
Bevy 0.15 engine version and the project-wide asset pipeline. It
does NOT depend on the hearthfield host crate (DLCs are siblings,
not dependents) but it uses the workspace-shared `ironclad`
proc-macro crate for typed lifecycle progressions, matching the
pattern in `src/shared/lifecycle_types.rs`.

## Substrate provenance

This DLC was filled in from an empty scaffold by the GC-OS substrate
across 10 chain-resident cork.compose iterations. Each iteration is
queryable by compose_id, signed, and replayable byte-identically.
See the `substrate/greenfield-slice-v1` branch's commit history for
the full lineage.
