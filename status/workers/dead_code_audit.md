# Dead Code Audit: `src/shared/mod.rs`

Audited all 43 `#[allow(dead_code)]` annotations by searching `src/**/*.rs` for each annotated item name.

## Removed `#[allow(dead_code)]` (used outside own definition/impl)

1. `ToolTier::upgrade_cost_gold` (`ui/shop_screen.rs`, tests in `shared/mod.rs`)
2. `ToolTier::upgrade_bars_needed` (`ui/shop_screen.rs`)
3. `ToolTier::upgrade_bar_item` (`ui/shop_screen.rs`)
4. `MapTransition` struct (`world/mod.rs`, `world/maps.rs`, etc.)
5. `ItemRemovedEvent.quantity` (`economy/*`, `fishing/*`, `npcs/gifts.rs`, etc.)
6. `ToolImpactEvent` struct (`player/tool_anim.rs`, `main.rs`)
7. `ItemQuality::next` (tests in `shared/mod.rs`)
8. `StaminaSource` enum (`player/interaction.rs`)
9. `QuestPostedEvent.quest` (`npcs/quests.rs`)
10. `impl BuildingTier` (`ui/building_upgrade_menu.rs`, `economy/buildings.rs`, `economy/blacksmith.rs`)
11. `AchievementUnlockedEvent` struct (`economy/achievements.rs`, `main.rs`)
12. `ScreenTransitionEvent` struct (`main.rs`)
13. `CutsceneStep` enum (`ui/intro_sequence.rs`, `ui/cutscene_runner.rs`, `calendar/mod.rs`)
14. `MenuTheme` struct (`ui/menu_kit.rs`, `ui/main_menu.rs`, `ui/pause_menu.rs`, `main.rs`)

## Kept `#[allow(dead_code)]` (no confirmed usage outside own definition/impl)

1. `GameState::Mining`
2. `ToolTier::upgrade_days`
3. `Npc.name`
4. `MineMonster.max_health`
5. `MineMonster.speed`
6. `ShopTransactionEvent.shop_id`
7. `ShopTransactionEvent.total_cost`
8. `GiftGivenEvent.preference`
9. `PlayMusicEvent.fade_in`
10. `SaveData`
11. `SEASONS_PER_YEAR`
12. `MAX_STAMINA`
13. `MAX_HEALTH`
14. `BACKPACK_SLOTS`
15. `TOTAL_INVENTORY_SLOTS`
16. `FRIENDSHIP_PER_HEART`
17. `MAX_HEARTS`
18. `MAX_FRIENDSHIP`
19. `ConsumeItemEvent.quality`
20. `AnimalPurchaseEvent`
21. `QuestCompletedEvent.reward_gold`
22. `HintEvent.hint_id`
23. `BuildingUpgradeEvent.from_tier`
24. `BuildingUpgradeEvent.cost_gold`
25. `BuildingUpgradeEvent.cost_materials`
26. `impl InputBlocks`
27. `TransitionStyle`
28. `MenuCursor`
29. `impl MenuCursor`
