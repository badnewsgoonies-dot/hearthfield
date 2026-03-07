# Animal Implementation Status

## Changes Made

### src/animals/spawning.rs

**1. `item_to_animal` function** — Added 10 new mappings:
- Plain IDs: `goat`, `duck`, `rabbit`, `pig`, `horse`
- Prefixed IDs: `animal_chicken`, `animal_cow`, `animal_sheep`, `animal_cat`, `animal_dog`, `animal_goat`, `animal_duck`, `animal_rabbit`, `animal_pig`, `animal_horse`

**2. `WanderAi` speed match** — Added 5 missing arms to make the match exhaustive:
- `Goat => 18.0`, `Duck => 18.0`, `Rabbit => 26.0`, `Pig => 14.0`, `Horse => 30.0`

### src/data/shops.rs

**3. AnimalShop listings** — Added 7 new entries after `animal_sheep`:
| Item         | Price   |
|--------------|---------|
| animal_goat  | 2,000g  |
| animal_duck  | 1,200g  |
| animal_rabbit| 4,000g  |
| animal_pig   | 8,000g  |
| animal_horse | 10,000g |
| animal_cat   | 500g    |
| animal_dog   | 500g    |

## Status: ✅ Complete
