# Spec Gap Analysis (GAME_SPEC.md vs current implementation)

Scope: base game in `src/` (not DLC crates). Percentages are implementation estimates against the 12 spec domains.

## 1) Calendar & Time (`calendar`)
Estimated implemented: **90%**

Rationale:
- 4 seasons x 28 days, year counter, 6:00 AM to 2:00 AM rollover, day-end + season-change events are implemented.
- Time pause behavior is implemented via state gating (time runs in `Playing`, pauses outside it), including dialogue/cutscene/menu contexts.
- Festival days exist and are date-bound to 1 per season.

Specced but not built:
- None obvious in core mechanics.

Built but not specced:
- Extra weather states/handling detail (`Stormy`, `Snowy`, weather particles, lightning behavior).
- Extra late-night warning toasts.

Built wrong vs spec:
- Festival themes differ from spec names/content: implemented festivals are Egg Festival/Luau/Harvest Festival/Winter Star, while spec lists Spring Dance/Summer Luau/Fall Harvest/Winter Star.

## 2) Player (`player`)
Estimated implemented: **85%**

Rationale:
- 4-direction movement, tool-use animation for all six tools, stamina/energy system, 36-slot inventory (12+24), and collision handling are present.
- Equipped tool cycling and hotbar selection exist.

Specced but not built:
- Full 12-slot direct number-key selection is not present (direct mapping is 1-9).

Built but not specced:
- Extra systems: low-stamina warning toasts, tool-upgrade lockouts during blacksmith upgrade queue, extra interaction dispatch layer.

Built wrong vs spec:
- Default tool-cycle bindings are `[`/`]` (KeyBindings defaults), not `Q`/`E` as specified.

## 3) Farm & Crops (`farming`)
Estimated implemented: **95%**

Rationale:
- Tilling, watering, planting, growth stages over days, daily water requirement behavior, withering/death progression, harvesting, seasonal crop gating, and all 15 spec crops are implemented.
- Sprinklers and scarecrow/crow logic are implemented.

Specced but not built:
- None obvious.

Built but not specced:
- Multiple sprinkler tiers (basic/quality/iridium) and richer farm object placement/removal flow.

Built wrong vs spec:
- None major found.

## 4) Animals (`animals`)
Estimated implemented: **80%**

Rationale:
- Chicken/cow/sheep/cat/dog behavior exists, including feed/pet happiness loop, trough interaction, product generation, barn/coop gating/upgrades, and 0-255 happiness modeling.

Specced but not built:
- Explicit “outside bonus” behavior is not clearly implemented as a separate mechanic.

Built but not specced:
- Additional animal kinds beyond spec baseline (goat/duck/rabbit/pig/horse scaffolding).
- Product quality tiers based on happiness.

Built wrong vs spec:
- Baby-to-adult age threshold is **5 days** in code, not 7 days.

## 5) World & Maps (`world`)
Estimated implemented: **70%**

Rationale:
- Tilemap maps, map transitions (edge + doors), collision/walkability, seasonal visual changes/tints, breakable world objects, and seasonal forageables are implemented.
- Required map set (Farm/Town/Beach/Forest/Mine Entrance) exists.

Specced but not built:
- None major functionally.

Built but not specced:
- Additional interior maps (PlayerHouse/GeneralStore/AnimalShop/Blacksmith), weather FX, day-night overlay, chest/furniture systems.

Built wrong vs spec:
- Map dimensions do not match spec targets: implementation uses smaller maps (e.g., Farm 32x24 vs spec 64x64; Town 28x22 vs 48x48; Beach 20x14 vs 32x32; Forest 22x18 vs 40x40; Mine Entrance 14x12 vs 24x24).

## 6) NPCs & Dialogue (`npcs`)
Estimated implemented: **78%**

Rationale:
- 10 named NPCs, schedule system with time/day/weather/festival variants, gifting preference tiers, friendship points/hearts, birthday +8x gift multiplier, and 2 marriage candidates are present.
- NPC movement to schedule waypoints is implemented.

Specced but not built:
- True dialogue tree structure (greeting -> topic selection -> goodbye) is not implemented; dialogue is mostly line selection, not branching topics.

Built but not specced:
- Romance/marriage flow systems, spouse behaviors, emotes, daily talk tracking, daily quests/story quest hooks.

Built wrong vs spec:
- Pathing is simple direct movement/greedy behavior, not robust obstacle-aware pathfinding.

## 7) Shops & Economy (`economy`)
Estimated implemented: **93%**

Rationale:
- General store, animal shop, blacksmith, shipping bin day-end sell-through, fixed pricing, starting gold 500, and upgrade price ladder + bar requirements are implemented.
- Buy/sell transaction flow and affordability handling are present.

Specced but not built:
- None obvious.

Built but not specced:
- Building upgrade timing systems, achievements/evaluation scoring, richer stats/play-metrics.

Built wrong vs spec:
- None major found.

## 8) Crafting & Cooking (`crafting`)
Estimated implemented: **88%**

Rationale:
- Crafting screen/bench, cooking with kitchen gate, recipe unlock flows, food energy/buffs, and all four spec processing machines (furnace/preserves jar/cheese press/loom) are implemented.
- Cooking recipe count matches spec target (15).

Specced but not built:
- None obvious at baseline feature level.

Built but not specced:
- Extra machine types and outputs (keg/oil maker/mayo/tapper/bee house/recycling/crab pot).
- Expanded crafting catalog.

Built wrong vs spec:
- Crafting recipe count is **21**, not spec’s “20 initially”.

## 9) Fishing (`fishing`)
Estimated implemented: **90%**

Rationale:
- Rod cast into water, timing minigame, fish selection by season/location/time/weather, rarity tiers, bait/tackle modifiers, and integration with sell/cook/gift loops are implemented.

Specced but not built:
- None obvious.

Built but not specced:
- Fishing skill progression, encyclopedia, treasure systems, extra legendary fish framework.

Built wrong vs spec:
- Fish count exceeds baseline: **28 species** implemented vs spec “20 initially”.

## 10) Mining (`mining`)
Estimated implemented: **92%**

Rationale:
- 20-floor mine progression, ladder discovery from rock breaking, ore depth progression, gem drops, 3 enemy types, separate health, elevator every 5 floors, and stamina use are implemented.

Specced but not built:
- None obvious.

Built but not specced:
- Knockout penalties and mine-specific HUD/elevator UI state flow.

Built wrong vs spec:
- None major found.

## 11) UI System (`ui`)
Estimated implemented: **72%**

Rationale:
- HUD (time/date/gold/stamina/tool), dialogue box, shop UI, crafting menu, pause menu, map screen, and relationship screen with hearts are implemented.

Specced but not built:
- Inventory drag-and-drop behavior is not implemented; inventory is cursor/nav based.
- Pause menu lacks the spec’s in-menu Settings option (currently Resume/Save/Quit).

Built but not specced:
- Journal, stats/calendar/settings/debug overlays, touch controls, toast framework, intro/cutscene transitions.

Built wrong vs spec:
- Inventory UX differs materially from drag-and-drop requirement.

## 12) Save & Settings (`save`)
Estimated implemented: **68%**

Rationale:
- Full-state serialization is extensive, 3 save slots exist, autosave on day end/sleep exists, manual save/load and quicksave are present.

Specced but not built:
- Persistent settings for keybind remapping and window-size configuration are not implemented.
- Settings persistence itself is missing (volume/keybind display is runtime UI, not saved config).

Built but not specced:
- Extra save metadata, save-versioning, additional tracked systems/resources in save payload.

Built wrong vs spec:
- Settings implementation is partial compared to spec (volume-only overlay, no full settings model for keybind/window-size persistence).

## Cross-domain architecture gap (important)
- `GAME_SPEC.md` calls for “No direct cross-domain function calls — everything through shared resources/events/state transitions.”
- Current code has direct cross-domain imports/usages in many places (notably `save` and multiple UI/economy/player systems), so architecture is functionally integrated but not strictly adhering to the specified isolation contract.

## Overall estimate
Average across 12 domains: **~84% implemented**.

Primary remaining spec risk areas:
- UI/Settings parity
- World map scale parity
- NPC dialogue/tree/pathfinding depth
- A few concrete behavior mismatches (animal aging days, keybinding defaults, recipe/spec count alignment)
