# dead_code audit: fishing + economy

Scope audited per request:
- `src/fishing/mod.rs`
- `src/fishing/legendaries.rs`
- `src/fishing/skill.rs`
- `src/economy/blacksmith.rs`
- `src/economy/evaluation.rs`
- `src/economy/gold.rs`
- `src/economy/shop.rs`

Method used for each item:
- `grep -rn 'item_name' src/ --include='*.rs'`
- Rule: remove `#[allow(dead_code)]` only if item is used outside its own definition.

## Results

### src/fishing/mod.rs
1. `unique_species` — NOT used outside definition (`grep` hit only function definition) → KEPT annotation
2. `CaughtFishEntry` — used outside definition (in `FishEncyclopedia.entries` and `record_catch`) → REMOVED annotation
3. `from_item_id` — NOT used outside definition → KEPT annotation
4. `effect_description` — NOT used outside definition → KEPT annotation
5. `setup` (`FishingMinigameState::setup`) — NOT used outside definition → KEPT annotation

### src/fishing/legendaries.rs
1. `legendary_fish_defs` — used outside definition (tests call it) → REMOVED annotation
2. `legendary_display_name` — used outside definition (`legendary_fish_defs`) → REMOVED annotation
3. `legendary_sell_price` — used outside definition (`legendary_fish_defs`, tests) → REMOVED annotation

### src/fishing/skill.rs
1. `apply_catch_zone` — used outside definition (tests call it) → REMOVED annotation
2. `new_level` field — used outside definition (`track_fishing_level_up`, event send site) → REMOVED annotation

### src/economy/blacksmith.rs
1. `upgrade_status` — NOT used outside definition → KEPT annotation
2. `ToolUpgradeCompleteEvent` — used outside definition (event reader/writer and economy module registration) → REMOVED annotation
3. `UpgradeEntry` — used outside definition (`list_available_upgrades` return type) → REMOVED annotation
4. `list_available_upgrades` — NOT used outside definition → KEPT annotation

### src/economy/evaluation.rs
1. `re_evaluate` annotation #1 — NOT used outside definition → KEPT annotation
2. `re_evaluate` annotation #2 (duplicate) — NOT used outside definition → KEPT annotation

### src/economy/gold.rs
1. `format_gold` — used outside definition (tests call it) → REMOVED annotation

### src/economy/shop.rs
1. `ActiveListing` — used outside definition (`ActiveShop.listings`, `build_listings` return/type construction) → REMOVED annotation

## Net change summary
- Removed unnecessary `#[allow(dead_code)]`: 9
- Left necessary `#[allow(dead_code)]` in place: 8
- Code bodies untouched; only annotation lines were removed where justified.
