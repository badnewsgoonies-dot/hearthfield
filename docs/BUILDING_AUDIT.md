# CATEGORY AUDIT: Buildings & Exteriors

## Current State Assessment

### How buildings work right now

Every building exterior is rendered in ONE of two ways:
1. **Composite sprite** — a single PNG (farmhouse.png, barn.png, chicken_house.png) stretched to fit a grid footprint, tinted with a `roof_tint` color
2. **Tile-by-tile** — wall tiles from house_walls.png + roof tiles from house_roof.png assembled per-tile (legacy path, still in code)

**The problem:** Almost every building in Town uses the SAME farmhouse.png with different color tints. The General Store, Animal Shop, NPC houses, and Library are all the farmhouse sprite with different tints. The Blacksmith and Tavern use barn.png. There are only 4 building images total: Farmhouse, Barn, ChickenHouse, Well.

### Door system
- Doors ARE animated (door_anim.png: 224×32 = 7 frames × 32px)
- DoorAnimTimer cycles the sprite frames at 0.4s intervals
- Player walks ONTO the door tile → MapTransition fires → warps to interior
- The door swings open/closed automatically via the animation system
- Door is rendered at 2×TILE_SIZE (32×32 visual)

### What exists per building

| Feature | Player House | Gen Store | Animal Shop | Blacksmith | NPC House 1 | NPC House 2 | Library | Tavern |
|---------|-------------|-----------|-------------|------------|-------------|-------------|---------|--------|
| Unique sprite | farmhouse.png | farmhouse.png | farmhouse.png | barn.png | farmhouse.png | farmhouse.png | farmhouse.png | barn.png |
| Color tint | ✓ brown | ✓ warm | ✓ cool blue | ✓ dark metal | ✓ green | ✓ golden | ✓ blue-grey | ✓ warm orange |
| Door anim | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Chimney smoke | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Candles (interior) | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Surrounding objects | trees | bush×2 | bush×1 | — | — | — | — | — |
| Fence | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Garden/planters | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Sign/mailbox | ✗ | building sign | building sign | building sign | ✗ | ✗ | ✗ | ✗ |
| Window glow (ext) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Unique character | tint only | tint only | tint only | tint only | tint only | tint only | tint only | tint only |

### Farm buildings

| Feature | Barn | Chicken Coop | Shipping Bin | Crafting Bench |
|---------|------|-------------|-------------|----------------|
| Sprite | barn.png | chicken_house.png | shipping_bin.png | crafting_bench.png |
| Door | ✓ | ✓ | N/A | N/A |
| Fence/paddock | ✗ | ✗ | ✗ | N/A |
| Unique surroundings | ✗ | ✗ | ✗ | ✗ |

---

## Gap Analysis (what's missing for "alive" buildings)

### CRITICAL (every building should have these)
1. **Window glow at night** — exterior windows should emit warm light after 6PM
2. **Unique surroundings** — each building needs 2-3 objects that say what it IS
   - Blacksmith: anvil outside, coal pile, metal scraps
   - General Store: crates, barrels, "OPEN" sign
   - Library: bookshelf visible through window, reading bench
   - Tavern: lanterns, outdoor table, barrel
   - NPC houses: garden, mailbox, unique plant/decor
3. **Building signs** — currently only 3 buildings have signs. All should.

### HIGH (makes buildings feel unique)
4. **Per-building exterior objects** that match the NPC who lives there:
   - Margaret's house (baker): flour sacks, bread cooling rack
   - Old Tom's dock area: fishing gear, nets, boat
   - Elena's blacksmith: already has forge, add exterior anvil
   - Lily's flower shop: flower boxes, potted plants
   - Doc's clinic: herb garden, mortar visible
5. **Fence/garden parcels** around NPC houses — even a simple 3-tile fence changes everything
6. **Chimney smoke on ALL buildings that should have it** — currently only 3 of 8

### MEDIUM (polish)
7. **Door mat / welcome mat** colored sprite in front of each door
8. **Exterior lighting** — lantern objects near doors that glow at night
9. **Weather effects per building** — snow on roofs in winter, puddles near doors in rain
10. **Seasonal decor** — flower boxes in spring/summer, harvest decor in fall, wreaths in winter

---

## Dispatch Plan

### Wave 1: Window glow system (affects ALL buildings)
One system that adds warm yellow overlay sprites to building footprints after 6PM.
Uses existing is_indoor_map() check inverted + building positions from town_buildings()/farm_buildings().

### Wave 2: Per-building exterior objects (Town buildings)
Add unique WorldObjectKind variants or positioned sprites near each building entrance.
8 buildings × 2-4 objects each = ~20-30 new placed objects.

### Wave 3: Per-building exterior objects (Farm buildings)  
Barn paddock fence, coop yard, path decorations around house.

### Wave 4: Missing chimney smoke + building signs
Extend chimney_positions() to cover all 8 town buildings.
Extend spawn_building_signs() to cover NPC houses.

### Wave 5: Seasonal exterior decor
Swap/add decorative objects by season around building entrances.

---

## Other Categories Pending Audit

- **Water bodies**: ponds, rivers, ocean, mountain lake — animation, edge rendering, reflections
- **Trees**: seasonal variation, unique species per biome, fruit trees visual stages
- **Paths**: autotiling (currently hardcoded to crossroads index 5), worn dirt near buildings
- **Fences**: exist as tileset but not placed anywhere on exterior maps
- **Interior maps**: furniture, wall decoration, NPC-specific items
