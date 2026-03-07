# HUD & Touch Audit — `src/ui/hud.rs` + `src/ui/touch.rs` + `src/ui/mod.rs`

---

## 1. HUD Elements

| Element | Marker Component | Position | Data Source | Data Shown |
|---|---|---|---|---|
| Time display | `HudTimeText` | Top-bar left | `Calendar` (season, day, year, hour, minute) | `"Spring 1, Year 1 - 6:00 AM"` |
| Weather icon | `HudWeatherIcon` | Top-bar left, next to time | `Calendar.weather` → `weather_icons.png` atlas | Sprite frame per weather type |
| Weather text | `HudWeatherText` | Top-bar left, after icon | `Calendar.weather` | Label + color: Sunny/Rainy/Stormy/Snowy |
| Tool name | `HudToolText` | Top-bar center | `PlayerState.equipped_tool` + `PlayerState.tools` | `"Copper Hoe"` (tier + tool name) |
| Gold | `HudGoldText` | Top-bar right | `PlayerState.gold` | `"500 G"` |
| Stamina bar | `HudStaminaFill` | Top-bar right | `PlayerState.stamina / max_stamina` | Width % + gradient color (green→yellow→red) |
| Hotbar slots (×HOTBAR_SLOTS) | `HotbarSlot`, `HotbarItemIcon`, `HotbarItemText`, `HotbarQuantityText` | Bottom center | `Inventory.slots` + `ItemRegistry` | Item icon sprite, item name (fallback text if atlas unloaded), quantity `"x3"`, key number label, selection highlight |
| Map name | `HudMapName` | Bottom-left absolute | `PlayerState.current_map` | Map display name, fades in on transition then fades out over 0.8 s |
| Tutorial objective | `HudObjective` | Top-left absolute (below top bar) | `TutorialState.current_objective` | `"> <objective text>"` |
| Interaction prompt | `HudInteractionPrompt` | Centered above hotbar | Nearby `Interactable`, `ChestMarker`, `Npc` entities + `Inventory` selected slot | `"[F] Talk to Lily"` or `"[F] Talk to Lily | [R] Give Gift"` or `"[F] Storage"` etc. |

---

## 2. Touch Controls

| Status | Detail |
|---|---|
| **`src/ui/touch.rs` does not exist** | File is absent from the repository entirely |
| No touch module declared | `src/ui/mod.rs` has no `mod touch` declaration |
| No touch systems registered | `UiPlugin::build()` registers zero touch-related systems |
| No touch resources/events | No touch input types found in the UI plugin |

| Gameplay Action | Keyboard Binding | Touch Coverage |
|---|---|---|
| Move player | WASD / Arrow keys | ❌ None |
| Use tool | Space / Left-click | ❌ None |
| Interact / Talk | F | ❌ None |
| Give gift | R | ❌ None |
| Open inventory | I / Tab | ❌ None |
| Open pause menu | Escape | ❌ None |
| Hotbar selection | 1–9 keys | ❌ None |
| Cycle hotbar | Q/E or scroll | ❌ None |
| Run / sprint | Shift | ❌ None |
| Confirm menu | Enter / Space | ❌ None |
| Cancel menu | Escape | ❌ None |

---

## 3. Bugs

| # | Category | Location | Description |
|---|---|---|---|
| B1 | Missing file | `src/ui/touch.rs` | File does not exist; no touch controls implemented for any gameplay action |
| B2 | Hardcoded key labels | `hud.rs:update_interaction_prompt` | Prompt always shows `[F]` and `[R]` keyboard labels; no fallback for controller or touch players |
| B3 | Unverified binding | `hud.rs:update_interaction_prompt` | Prompt displays `[R] Give Gift` but the R key binding is defined outside these files — its existence cannot be confirmed from files in scope |
| B4 | Weather atlas row skip | `hud.rs:update_weather_icon` | Comment lists Row 1 as "cloud/overcast" but no `Weather` variant maps to indices 4–7; Row 1 of the atlas is entirely unused |
| B5 | No change detection | `hud.rs:update_hotbar`, `update_hotbar_icons` | Neither system gates on `inventory.is_changed()`; both query and iterate every frame unconditionally |
| B6 | Missing system | `src/ui/mod.rs` | No systems registered for touch input in `UiPlugin`; `GameState::Mining` and `GameState::Fishing` states also have no `menu_cancel_transitions` guard (cannot keyboard-escape from them) |
