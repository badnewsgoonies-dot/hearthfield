# Worker: Wave 1-B — NPC O(n²) Fix + Farm West Edge Routing

## Objective
Fix two bugs: an O(n²) performance issue in NPC cleanup, and incorrect map transition routing.

## Bug 1: NPC cleanup O(n²) pattern
**Location:** `src/npcs/map_events.rs` lines 31-40

The code uses `Vec::contains` inside a `retain` loop, which is O(n²) worst-case.

**Fix required:**
- Convert `despawned_entities` from `Vec<Entity>` to `HashSet<Entity>`
- Replace `despawned_entities.contains(&e)` with `despawned_entities.contains(&e)` on the HashSet
- Add `use std::collections::HashSet;` if not present

**Before pattern (pseudocode):**
```rust
let despawned_entities: Vec<Entity> = ...;
map.retain(|_, e| !despawned_entities.contains(e));  // O(n*m)
```

**After pattern:**
```rust
let despawned_entities: HashSet<Entity> = ...collect();
map.retain(|_, e| !despawned_entities.contains(e));  // O(n)
```

## Bug 2: Farm west edge routes to Beach instead of Mine
**Location:** `src/player/interaction.rs` around lines 72-74

When the player walks west from the Farm map, they transition to Beach. Per the world layout and GAME_SPEC.md line 89, farm west should route toward the Mine entrance.

**Fix required:**
- Find the edge transition logic for the Farm map's west edge
- Change the destination from `MapId::Beach` to `MapId::MineEntrance`
- Verify spawn position makes sense for entering from the east

## Files to modify
1. `src/npcs/map_events.rs` — Convert Vec to HashSet for despawned entity lookup
2. `src/player/interaction.rs` — Fix farm west edge transition destination

## Validation
Run these commands before reporting done:
```bash
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

All must pass with zero errors and zero warnings.

## Do NOT modify
- Map generation logic
- Other edge transitions that are correct
- NPC schedules or dialogue
