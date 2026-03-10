# Sailing & Island Feature Spec

## Overview
Player can craft a boat, launch from a dock on the Beach map, sail across water tiles, fish from the boat, and sail to a new Coral Island map accessible only by water.

## New Types (contract extension in shared/mod.rs)

```rust
// Add to GameState enum:
Sailing,  // Player is in boat, water is walkable

// New resource:
pub struct BoatMode {
    pub active: bool,
    pub stamina_drain_per_tile: f32,  // 0.5 stamina per water tile moved
}
```

## New WorldObjectKind variants (maps.rs)
```rust
Dock,       // Wooden pier — player boards/disembarks boat here
PalmTree,   // Tropical tree for island
Coral,      // Decorative coral on island beach
Driftwood,  // Harvestable beach wood
```

## New MapId
```rust
CoralIsland,  // Tropical island accessible by boat from Beach
```

## Architecture — How Sailing Works

1. Player crafts "boat" item (30 wood + 5 hardwood + 3 rope)
2. Player walks to a Dock object on Beach map, presses F to interact
3. If player has "boat" in inventory → BoatMode.active = true, GameState stays Playing
4. WorldMap::is_walkable checks BoatMode: if active, Water=walkable, non-Water/non-Dock=solid (inverted)
5. Player moves on water tiles normally. Camera follows. Sprite changes to boat sprite.
6. Reaching Dock object while in boat → F to disembark → BoatMode.active = false
7. Sailing to south edge of Beach map → MapTransitionEvent to CoralIsland (north dock)

## Fishing from Boat
The existing fishing system already validates target_tile == Water.
When player is in boat (on water), adjacent tiles are also water → casting works naturally.
Only change needed: allow FishingRod tool use while BoatMode.active (currently may be blocked by tool anim).

## Coral Island Map (30x22)
- Sandy tropical island surrounded by water
- North dock (Dock objects) — arrival/departure point
- Interior: PalmTree grove, open sand clearing, small freshwater pond
- Southeast: rocky tide pools (Rock, Coral objects + unique forage)
- West: sandy beach with Driftwood, shell forage points
- 15+ PalmTree, 8+ Coral, 5+ Driftwood, 4+ Rock
- 8+ forage points (shells, sea glass, tropical herbs)
- Surrounded by Water tiles on all edges except north dock area
- edges: north = Some((Beach, Fixed(10, 12))) — sail back to beach

## Beach Map Changes
- Add 2 Dock objects at south edge (around x=9-10, y=12-13)
- Add south edge transition: when in boat, south edge → CoralIsland
- edges.south: Some((CoralIsland, Fixed(15, 1))) — only accessible while BoatMode.active

## Crafting Recipe
```
"boat" => Recipe {
    result: "boat".into(),
    result_count: 1,
    ingredients: vec![("wood", 30), ("hardwood", 5), ("rope", 3)],
    category: CraftingCategory::Crafting,
}
```
Note: "rope" item may need to be added to ItemRegistry if not present.

## File Changes Required
1. src/shared/mod.rs — BoatMode resource, GameState::Sailing (or just use Playing), MapId::CoralIsland
2. src/world/maps.rs — WorldObjectKind::{Dock, PalmTree, Coral, Driftwood}, generate_coral_island(), MapId::CoralIsland in spawn/generate
3. src/world/map_data.rs — CoralIsland in registries, edges, filename
4. src/world/mod.rs — is_walkable boat check, CoralIsland outdoor match, dock sprite rendering
5. src/world/objects.rs — Dock/PalmTree/Coral/Driftwood sprite rendering, drop tables, colors, sizes
6. src/player/interaction.rs — dock boarding/disembarking interaction, CoralIsland bounds
7. src/player/mod.rs — boat movement visual (optional: swap sprite)
8. src/crafting/recipes.rs — boat recipe
9. src/fishing/cast.rs — ensure casting works while BoatMode.active
10. src/ui/audio.rs — CoralIsland music
11. src/ui/hud.rs — CoralIsland display name
12. src/ui/map_screen.rs — CoralIsland label
13. assets/maps/coral_island.ron — new map data
14. assets/maps/beach.ron — add dock objects, update edges
