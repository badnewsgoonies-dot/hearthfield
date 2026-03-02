# Domain: Blacksmith Interior

## Map: Blacksmith (12×12 grid)
Tool upgrades, ore processing. Industrial, hot, stone floors.

## Layout
- Void perimeter, door at x=5,6 y=11 (top, WoodFloor)
- **Forge Area** (bottom-right, y=1-3, x=7-10): Dirt tiles (heated), furnace furniture
- **Anvil Area** (center, y=4-6, x=4-6): WoodFloor island in stone  
- **Counter/Reception** (y=3, x=2-6): WoodFloor counter line
- **Storage** (bottom-left, y=1-3, x=1-3): Crates and barrels
- Rest of interior: Stone floor

## Furniture targets: 12-15 pieces
- Anvil: idx 19 at center
- Furnace: idx 18 along back wall
- Crates: idx 36, 37, 38
- Barrels: idx 27 (water barrel)
- Counter: idx 32, 33
- Lamp: idx 24

## Transition
from_rect: (5, 11, 2, 1) → MapId::Town, to_pos: (22, 14)

## Output: generate_blacksmith() + blacksmith_furniture()
tiles[y * width + x], y=0 bottom, y=11 top (door side).
