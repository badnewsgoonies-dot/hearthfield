# Domain Spec: Economy & Shops

## Scope
`src/economy/` — `mod.rs`, `shop.rs`, `shipping.rs`, `blacksmith.rs`, `buildings.rs`, `gold.rs`, `achievements.rs`, `tool_upgrades.rs`, `evaluation.rs`, `play_stats.rs`, `stats.rs`

## Responsibility
Gold management, shop buy/sell logic, shipping bin end-of-day sales, tool upgrades at blacksmith, building upgrades (coop, barn, house, silo), achievement tracking, play statistics, and year-end evaluation.

## Shared Contract Types (import from `crate::shared`)
- `ShopId` (GeneralStore, AnimalShop, Blacksmith)
- `ShopListing` (item_id, price, season_available)
- `ShopData` (Resource — listings HashMap)
- `ShippingBin` (Resource — items Vec)
- `ShippingLog` (Resource — shipped_items HashMap)
- `Inventory`, `InventorySlot`, `ItemId`, `ItemCategory`
- `ItemRegistry`, `ItemDef`
- `PlayerState` (gold, tools)
- `ToolKind`, `ToolTier`
- `AnimalState` (has_coop, has_barn, coop_level, barn_level)
- `HouseState`, `HouseTier`
- `BuildingKind`, `BuildingTier`
- `Achievements` (Resource — unlocked, progress)
- `PlayStats` (Resource — all counters)
- `EvaluationScore` (Resource — total_points, categories, evaluated, candles_lit)
- `Calendar`, `Season`
- Events: `ShopTransactionEvent`, `GoldChangeEvent`, `DayEndEvent`, `CropHarvestedEvent`, `AnimalProductEvent`, `AchievementUnlockedEvent`, `BuildingUpgradeEvent`, `EvaluationTriggerEvent`, `ToastEvent`, `ItemPickupEvent`

## Quantitative Targets
- Starting gold: 500
- Tool upgrade costs (per tier): Basic→Copper 2000g, Copper→Iron 5000g, Iron→Gold 10000g, Gold→Iridium 25000g
- Tool upgrade bars needed: 5 per tier
- Tool upgrade time: 2 days at blacksmith
- Building upgrade costs:
  - Coop: Basic 4000g+300wood, Big 10000g+400wood, Deluxe 20000g+500wood
  - Barn: Basic 6000g+350wood, Big 12000g+450wood, Deluxe 25000g+550wood
  - House: Big 10000g+450wood, Deluxe 50000g+1000wood (enables kitchen/nursery)
- Shipping: items in bin sold at base_sell_price at day end
- Achievement count: 20+ trackable achievements
- Year-end evaluation: score categories (earnings, relationships, farming, exploration), candles 0-4

## Constants & Formulas
- Quality sell multiplier: Normal 1.0, Silver 1.25, Gold 1.5, Iridium 2.0
- Shipping revenue: `sum(item.sell_price * quality_multiplier * quantity)`
- Evaluation candles: 0-3 points = 1 candle, 4-7 = 2, 8-11 = 3, 12+ = 4
- Achievement progress tracked via event listening (crops_harvested, fish_caught, etc.)

## Key Systems
1. `gold_change_handler` — listen to `GoldChangeEvent`, update `PlayerState.gold`
2. `shop_transaction` — handle buy/sell, validate gold, update inventory, fire events
3. `shipping_sell` — on `DayEndEvent`, sell all items in `ShippingBin`, add gold, log to `ShippingLog`
4. `tool_upgrade_system` — process upgrade requests, deduct gold+bars, track 2-day timer
5. `building_upgrade_system` — handle `BuildingUpgradeEvent`, deduct resources
6. `achievement_tracking` — listen to various events, increment `PlayStats`, check unlock conditions
7. `evaluation_system` — on `EvaluationTriggerEvent`, calculate score, set candles

## Does NOT Handle
- Shop UI rendering (ui domain)
- Item definitions (data domain)
- Player movement to shop locations (player domain)
- NPC shop dialogue (npcs domain)
- Shop map generation (world domain)
