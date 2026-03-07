# Domain Spec: Animals

## Scope
`src/animals/` — `mod.rs`, `day_end.rs`, `feeding.rs`, `interaction.rs`, `movement.rs`, `products.rs`, `rendering.rs`, `spawning.rs`

## Responsibility
Livestock lifecycle (buy → feed → produce → sell), happiness system, baby → adult aging, product generation, petting interaction, and animal rendering.

## Shared Contract Types (import from `crate::shared`)
- `Animal` (Component — kind, name, age, days_old, happiness, fed_today, petted_today, product_ready)
- `AnimalKind` (Chicken, Cow, Sheep, Goat, Duck, Rabbit, Pig, Horse, Cat, Dog)
- `AnimalAge` (Baby, Adult)
- `AnimalState` (Resource — animals Vec, has_coop, has_barn, coop_level, barn_level)
- `Inventory`, `InventorySlot`
- `PlayerInput`, `PlayerState`
- `LogicalPosition`, `YSorted`, `GridPosition`
- Events: `DayEndEvent`, `AnimalProductEvent`, `ItemPickupEvent`
- Constants: `TILE_SIZE` (16.0), `Z_ENTITY_BASE` (100.0)

## Quantitative Targets
- 3 primary livestock: Chicken, Cow, Sheep
- Buy prices: Chicken 800g, Cow 1500g, Sheep 4000g
- Products: Egg (daily, 50g), Milk (daily, 125g), Wool (every 3 days, 340g)
- Happiness: 0-255, affects product quality
  - +10 per day if fed
  - +5 if petted
  - -20 per day if NOT fed
  - +5 if outside in Sunny weather
- Baby → Adult: 7 days
- Building capacity: Basic coop/barn = 4, Big = 8, Deluxe = 12
- Product quality tiers:
  - Happiness 0-99: Normal (1.0x)
  - Happiness 100-149: Silver (1.25x)
  - Happiness 150-199: Gold (1.5x)
  - Happiness 200-255: Iridium (2.0x)

## Constants & Formulas
- `HAPPINESS_FED_BONUS = 10`
- `HAPPINESS_PETTED_BONUS = 5`
- `HAPPINESS_UNFED_PENALTY = -20`
- `HAPPINESS_OUTDOOR_SUNNY = 5`
- `BABY_TO_ADULT_DAYS = 7`
- Product ready: adults only, fed_today == true, product_ready resets daily
- Sheep wool cycle: every 3 days (tracked via `days_old % 3`)

## Key Systems
1. `animal_day_end` — on `DayEndEvent`: decay happiness if unfed, age babies, reset daily flags
2. `feeding_system` — player places hay or auto-feed from silo, set `fed_today = true`
3. `petting_system` — player interacts with animal, `petted_today = true`, +happiness
4. `product_generation` — at morning, mark `product_ready = true` for eligible adults
5. `product_collection` — player interacts with ready animal, add product to inventory, fire `AnimalProductEvent`
6. `animal_movement` — simple random wandering within barn/coop bounds
7. `animal_rendering` — sprite based on kind, age, facing

## Does NOT Handle
- Buying animals (economy/shop domain)
- Building coop/barn (economy/buildings domain)
- Selling animal products (economy/shipping domain)
- Animal product item definitions (data domain)
- Animal shop UI (ui domain)
