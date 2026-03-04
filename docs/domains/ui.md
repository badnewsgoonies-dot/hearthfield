# Domain Spec: UI System

## Scope
`src/ui/` — `mod.rs`, `hud.rs`, `inventory_screen.rs`, `shop_screen.rs`, `crafting_screen.rs`, `dialogue_box.rs`, `pause_menu.rs`, `main_menu.rs`, `building_upgrade_menu.rs`, `chest_screen.rs`, `toast.rs`, `minimap.rs`, `debug_overlay.rs`, `intro_sequence.rs`, `cutscene_runner.rs`, `transitions.rs`, `tutorial.rs`, `menu_input.rs`, `menu_kit.rs`, `audio.rs`

## Responsibility
All visual UI: HUD, menus, inventory screen, shop screen, crafting screen, dialogue box, pause menu, main menu, toast notifications, minimap, debug overlay, intro sequence, cutscene runner, screen transitions, tutorial overlays, and audio integration.

## Shared Contract Types (import from `crate::shared`)
- `GameState` (all states for screen transitions)
- `Calendar`, `Season`, `Weather` (HUD display)
- `PlayerState` (stamina bar, gold, health, equipped_tool)
- `Inventory`, `InventorySlot` (inventory screen)
- `ItemRegistry`, `ItemDef`, `ItemCategory` (item display)
- `ShopData`, `ShopListing`, `ShopId` (shop screen)
- `RecipeRegistry`, `Recipe`, `UnlockedRecipes` (crafting screen)
- `Relationships`, `NpcRegistry`, `NpcDef` (friendship display)
- `QuestLog`, `Quest`, `QuestObjective` (quest display)
- `Achievements`, `PlayStats` (achievement display)
- `StorageChest`, `QualityStack` (chest screen)
- `MineState` (mine floor display)
- `TutorialState`, `HintEvent` (tutorial overlay)
- `CutsceneQueue`, `CutsceneStep` (cutscene runner)
- `MenuTheme`, `MenuItem`, `MenuAction` (menu styling/input)
- `PlayerInput`, `InputContext` (menu navigation)
- `InputBlocks`, `InteractionClaimed`
- `DebugOverlayState` (debug toggle)
- Events: `DialogueStartEvent`, `DialogueEndEvent`, `ToastEvent`, `PlaySfxEvent`, `PlayMusicEvent`, `ShopTransactionEvent`, `GoldChangeEvent`, `EatFoodEvent`, `BuildingUpgradeEvent`, `AchievementUnlockedEvent`, `HintEvent`, `MapTransitionEvent`
- Constants: `SCREEN_WIDTH` (960.0), `SCREEN_HEIGHT` (540.0), `HOTBAR_SLOTS` (12), `TOTAL_INVENTORY_SLOTS` (36)

## Quantitative Targets
- HUD elements: time/date, gold, stamina bar, health bar, equipped tool icon, hotbar (12 slots)
- Inventory screen: 36 slots (12 hotbar + 24 backpack), item tooltip on hover
- Shop screen: buy/sell tabs, scrollable item list, price display
- Crafting screen: recipe list, ingredient check (green/red), craft button
- Dialogue box: bottom-screen, NPC portrait (64×64), text with typewriter effect, advance prompt
- Pause menu: Resume, Save, Settings, Quit
- Main menu: New Game, Continue, Settings
- Toast: slide-in notification, 3-5 second duration, stack up to 3
- Minimap: top-right corner, shows current map layout
- Transitions: fade-out/fade-in on map transitions (0.3s each)

## Key Systems
1. `hud_system` — render persistent HUD (time, gold, stamina, hotbar)
2. `inventory_screen` — toggle on E, display items, handle selection/use
3. `shop_screen` — display shop listings, handle buy/sell transactions
4. `crafting_screen` — display recipes, ingredient checking, craft execution
5. `dialogue_box` — render dialogue with typewriter text, portrait, advance on input
6. `pause_menu` — render pause overlay, handle resume/save/quit
7. `main_menu` — title screen with menu options
8. `toast_system` — display and animate toast notifications
9. `minimap_system` — render mini map overlay
10. `cutscene_runner` — execute CutsceneStep queue, manage transitions
11. `transition_system` — fade-in/fade-out on map changes
12. `menu_input_system` — translate PlayerInput to MenuAction for all menus
13. `audio_system` — play SFX and music based on events

## Does NOT Handle
- Game logic behind shop transactions (economy domain)
- Dialogue content generation (npcs domain)
- Recipe validation logic (crafting domain)
- Save/load execution (save domain)
- Inventory add/remove logic (shared types handle this)
