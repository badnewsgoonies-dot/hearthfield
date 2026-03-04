# Worker: ECONOMY

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/economy/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/economy.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `ShopId`, `ShopListing`, `ShopData`
- `ShippingBin`, `ShippingLog`
- `Inventory`, `InventorySlot`, `ItemId`, `ItemCategory`
- `ItemRegistry`, `ItemDef`
- `PlayerState`, `ToolKind`, `ToolTier`
- `AnimalState`
- `HouseState`, `HouseTier`
- `BuildingKind`, `BuildingTier`
- `Achievements`, `PlayStats`
- `EvaluationScore`
- `Calendar`, `Season`
- Events: `ShopTransactionEvent`, `GoldChangeEvent`, `DayEndEvent`, `CropHarvestedEvent`, `AnimalProductEvent`, `AchievementUnlockedEvent`, `BuildingUpgradeEvent`, `EvaluationTriggerEvent`, `ToastEvent`, `ItemPickupEvent`

## Deliverables
- `src/economy/mod.rs` — `EconomyPlugin`
- `src/economy/shop.rs` — Buy/sell transaction logic
- `src/economy/shipping.rs` — Shipping bin end-of-day sales
- `src/economy/gold.rs` — Gold change handler
- `src/economy/blacksmith.rs` — Tool upgrade system
- `src/economy/tool_upgrades.rs` — Upgrade timer/completion
- `src/economy/buildings.rs` — Building upgrade handler
- `src/economy/achievements.rs` — Achievement tracking
- `src/economy/play_stats.rs` — Play statistics tracking
- `src/economy/evaluation.rs` — Year-end evaluation
- `src/economy/stats.rs` — Stat aggregation

## Quantitative targets (non-negotiable)
- Tool upgrade costs: 2000/5000/10000/25000g + 5 bars each
- Tool upgrade time: 2 days
- Building costs: Coop 4000g+300wood → Big 10000g+400wood → Deluxe 20000g+500wood; Barn 6000g+350wood → 12000g+450wood → 25000g+550wood
- House: Big 10000g+450wood, Deluxe 50000g+1000wood
- Quality multipliers: Normal 1.0, Silver 1.25, Gold 1.5, Iridium 2.0
- 20+ achievements

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/economy.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
