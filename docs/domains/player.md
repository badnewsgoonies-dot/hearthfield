# Domain Spec: Player

## Scope
`src/player/` — `mod.rs`, `movement.rs`, `camera.rs`, `tools.rs`, `tool_anim.rs`, `interaction.rs`, `interact_dispatch.rs`, `item_use.rs`, `spawn.rs`

## Responsibility
Player entity spawning, 4-directional movement, collision, tool use, camera following, interaction dispatching (F key), stamina management, and tool selection.

## Shared Contract Types (import from `crate::shared`)
- `Player` (Component marker)
- `PlayerMovement` (Component — facing, is_moving, speed, move_cooldown, anim_state)
- `PlayerState` (Resource — stamina, max_stamina, health, max_health, equipped_tool, tools, gold, current_map)
- `PlayerInput` (Resource — move_axis, interact, tool_use, tool_secondary, tool_next, tool_prev, tool_slot)
- `PlayerAnimState` (Idle, Walk, ToolUse)
- `Facing` (Up, Down, Left, Right)
- `ToolKind`, `ToolTier`
- `Inventory`, `InventorySlot`
- `InputBlocks`, `InteractionClaimed`
- `Interactable`, `InteractionKind`
- `LogicalPosition`, `YSorted`
- `GameState`
- Events: `ToolUseEvent`, `StaminaDrainEvent`, `ToolImpactEvent`, `MapTransitionEvent`, `EatFoodEvent`
- Constants: `TILE_SIZE` (16.0), `PIXEL_SCALE` (3.0), `MAX_STAMINA` (100.0), `MAX_HEALTH` (100.0)
- Functions: `grid_to_world_center()`, `world_to_grid()`, `tool_stamina_cost()`, `watering_can_area()`

## Quantitative Targets
- Player speed: 80.0 pixels/second
- 6 tools: Hoe, WateringCan, Axe, Pickaxe, FishingRod, Scythe
- Starting gold: 500
- Starting map: `MapId::PlayerHouse` at grid (8, 8)
- Stamina costs: base 4.0 per tool use × tier multiplier (Basic 1.0, Copper 0.85, Iron 0.7, Gold 0.55, Iridium 0.4)
- Tool selection: 1-9 hotkeys, [ ] for prev/next
- Interaction range: 1 tile in facing direction

## Constants & Formulas
- `speed = 80.0` pixels/sec
- Stamina cost: `base_cost * tier.stamina_multiplier()`
- Camera: lerp follow with smoothing factor
- Grid interaction target: `player_grid_pos + facing_delta(facing)`

## Key Systems
1. `player_movement` — read `PlayerInput.move_axis`, apply velocity, collision check
2. `camera_follow` — lerp camera to player position
3. `tool_use_system` — on `PlayerInput.tool_use`, determine tool, fire `ToolUseEvent`
4. `tool_animation` — advance tool animation frames, fire `ToolImpactEvent` at impact frame
5. `interaction_dispatch` — on `PlayerInput.interact`, find nearest `Interactable`, dispatch
6. `stamina_drain_handler` — listen to `StaminaDrainEvent`, reduce `PlayerState.stamina`
7. `player_spawn` — spawn player entity with `Player`, `PlayerMovement`, `LogicalPosition`, `YSorted`

## Does NOT Handle
- Farming logic (farming domain listens to `ToolUseEvent`)
- Fishing state machine (fishing domain triggers on rod cast)
- Mining combat (mining domain handles attack in mine context)
- NPC dialogue (npcs domain handles `DialogueStartEvent`)
- Inventory UI (ui domain handles inventory screen)
- Tool upgrades (economy/blacksmith handles upgrade logic)
