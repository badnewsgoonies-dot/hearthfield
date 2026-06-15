# Procedural maps — the generative-seedling channel

A map you can address by a single number. `MapId::Procedural(u64)` adds a generated-map channel
alongside the 18 authored RON maps. The `u64` **is the seed**, and the seed **is the address**:
`generate_procedural_map(seed, ..)` is a pure function of it, so the id self-addresses the map.
Nothing is stored — the full artifact (tiles, objects, forageables, the way home) is *grown* from the
coordinate on demand. Generation is retrieval.

## Drive it (in-game)

- **F6** — grow and enter a fresh map. Each press uses a new seed, so you cycle through biomes.
- **Walk off the south edge** — return to the Farm. (Procedural maps carry a single south-edge
  transition back to `MapId::Farm`.)
- Press **F6** again from anywhere for another one.

The HUD names the biome you're standing in ("The Wilds — Beach", …) and the music matches it, so each
press visibly and audibly announces the map it grew.

## Biomes

The seed picks one of four kinds of place (`seed % 4`), each with its own tile palette, object set,
and density:

| biome  | ground            | objects                         |
|--------|-------------------|---------------------------------|
| Meadow | grass, small pond | sparse trees / bushes / stumps  |
| Forest | grass, dirt       | dense trees / pines             |
| Beach  | sand band + water | palm trees / driftwood / docks  |
| Rocky  | grass + stone     | rocks / large rocks / logs      |

## Guarantees

- **Deterministic.** Same seed → byte-identical map (splitmix64 RNG, pure function of the seed).
- **Always traversable.** A central crossing spine (Path, bridging over water) plus a
  connectivity-repair pass means **every** walkable cell is reachable from the spawn. Verified at
  100% across 200 seeds (0 failures), all biomes.
- **Always escapable.** The south-edge transition returns to the Farm; the spawn sits on the spine.

## How it wires in (for maintainers)

- `MapId::Procedural(u64)` — `src/shared/mod.rs`.
- `src/world/procedural.rs` — `generate_procedural_map(seed, w, h, id) -> MapData` + tests
  (`deterministic_and_valid`, `fully_traversable_from_spawn`).
- `load_map_data` (`src/world/map_data.rs`) intercepts `Procedural(seed)` and generates instead of
  reading a RON file.
- `debug_enter_procedural_map` (`src/world/mod.rs`, F6) inserts the grown map into `MapRegistry`
  (so the registry-driven edge transition resolves) and fires a `MapTransitionEvent`. Only the
  current procedural map is retained, so the registry doesn't grow across presses.
- The new variant is handled everywhere `MapId` is matched (names, audio, bounds, spawn, edges,
  `generate_map`) — procedural maps are treated as outdoor.

## Substrate note

This is the **GENERATIVE SEEDLING** channel from the GCOS model, instantiated in a real game: content
enters the field not as a recorded artifact (a RON file in the DATA BANK) but as a *seed decoded into a
full artifact in a defined grammar/constraint field*. The map field here is uniform over a fixed tile
alphabet, so it's the **addressable** regime — `seed → map` is an address lookup, not a search and not
free synthesis. The generator can't lie about what it returns: the seed determines the artifact
exactly, and the same seed always unranks to the same map.
