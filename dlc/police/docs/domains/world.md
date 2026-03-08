# World Domain Spec — Precinct

## Purpose
Tilemap rendering, collision, map transitions. Provides the spatial environment.

## Scope
`src/domains/world/` — owns map loading, tile rendering, collision map, transitions.

## What This Domain Does
- Define map data for PrecinctInterior (32x24 tiles) — the Wave 1 hub map
- Spawn tilemap entities on `OnEnter(GameState::Playing)` based on current `PlayerState.position_map`
- Populate a `CollisionMap` resource with solid tile positions
- Define transition zones (door positions that connect maps)
- Handle `MapTransitionEvent`: despawn current map, load new map, reposition player
- Render tiles using colored sprites (Wave 1 — no art assets yet)

## What This Domain Does NOT Do
- Player movement or input (player domain)
- Time/weather (calendar domain)
- UI rendering (ui domain)
- Cases, evidence, NPCs, economy

## Key Types (import from crate::shared)
- `MapId`, `TileKind`, `GridPosition`, `MapTransition`
- `MapTransitionEvent` (Event)
- `PlayerState` (Resource — read position_map)
- `UpdatePhase` — systems in `Simulation`
- Constants: `TILE_SIZE`, `PIXEL_SCALE`

## New Types (define locally in this domain)
- `CollisionMap` — Resource, `HashSet<(i32, i32)>` of solid tile positions
- `TileMapData` — struct holding tile grid for a map
- `MapTile` — component on tile entities

## Systems to Implement
1. `spawn_map` — `OnEnter(GameState::Playing)` or on `MapTransitionEvent`
   - Read PlayerState.position_map to determine which map
   - Spawn tile entities with `Sprite` (colored by TileKind) + `Transform`
   - Floor = dark gray, Wall = dark brown, Door = lighter brown, Sidewalk = light gray
   - Populate CollisionMap with Wall positions
2. `handle_map_transition` — read `MapTransitionEvent`
   - Despawn all entities with `MapTile` component
   - Update `PlayerState.position_map`
   - Call spawn_map logic for new map
   - Reposition player at transition target

## Map: PrecinctInterior (32x24)
The hub map. Layout:
- Outer walls on all edges
- Main hallway running east-west through center
- Rooms: lobby (entrance, south), case board room (west), evidence room (east),
  captain's office (northwest), break room (northeast), locker room (southeast)
- Doors connecting hallway to each room
- Transition zone at south entrance → PrecinctExterior

Spawn point: grid (16, 20) — center of lobby near entrance

## Quantitative Targets
- PrecinctInterior: 32x24 = 768 tiles
- At least 6 distinct rooms with door connections
- 1 transition zone (to PrecinctExterior — target map won't exist yet in Wave 1, that's OK)
- CollisionMap populated with all Wall tiles

## Decision Fields
- **Preferred**: hardcoded map data in Rust arrays for Wave 1
- **Tempting alternative**: RON file loading (data-driven maps)
- **Consequence**: RON loading adds complexity before we have a map editor; hardcode first, extract later
- **Drift cue**: worker creates RON parser, asset loading pipeline, or map editor

- **Preferred**: colored rectangle tiles for Wave 1
- **Tempting alternative**: sprite atlas tiles
- **Consequence**: no art assets exist yet; atlas loading is premature work
- **Drift cue**: worker references texture atlas or tries to load PNG files

## Plugin Export
```rust
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionMap>()
        // OnEnter(Playing) → spawn_map
        // Systems for map transition handling
    }
}
```

## Tests (minimum 5)
1. Map spawns correct number of tile entities (768 for 32x24)
2. CollisionMap contains all Wall tile positions
3. CollisionMap does NOT contain Floor tile positions
4. MapTransitionEvent despawns old tiles and spawns new ones
5. Player spawn point is on a walkable tile
