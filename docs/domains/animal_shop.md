# Domain: Animal Shop Interior

## Map: AnimalShop (12×12 grid)  
Sells animals and animal supplies. Rustic barn feel.

## Layout
- Void perimeter, door at x=5,6 y=11 (top, WoodFloor)
- **Feed/Hay Storage** (bottom-left, y=1-3, x=1-4): Dirt floor tiles, hay bales
- **Sales Counter** (y=3, x=4-8): Stone tiles
- **Display Area** (right side): Shelves with animal supplies
- **Customer area** (y=4-10): Open WoodFloor
- Scattered hay bales and barrels for atmosphere

## Furniture targets: 12-15 pieces
- Hay bales: idx 45, 46, 47
- Barrels: idx 27, 28
- Crates: idx 36
- Shelves: idx 9, 10, 11  
- Counter: idx 32, 33
- Plant: idx 22

## Transition
from_rect: (5, 11, 2, 1) → MapId::Town, to_pos: (22, 3)

## Output: generate_animal_shop() + animal_shop_furniture()
tiles[y * width + x], y=0 bottom, y=11 top (door side).
