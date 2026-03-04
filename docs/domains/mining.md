# Domain Spec: Mining

## Scope
`src/mining/` — `mod.rs`, `floor_gen.rs`, `rock_breaking.rs`, `combat.rs`, `ladder.rs`, `transitions.rs`, `movement.rs`, `spawning.rs`, `components.rs`, `hud.rs`

## Responsibility
Mine floor generation, rock breaking and ore drops, combat with monsters, ladder discovery and floor transitions, elevator system, and mine HUD.

## Shared Contract Types (import from `crate::shared`)
- `MineState` (Resource — current_floor, deepest_floor_reached, elevator_floors)
- `MineRock` (Component — health, drop_item, drop_quantity)
- `MineMonster` (Component — kind, health, max_health, damage, speed)
- `MineEnemy` (GreenSlime, Bat, RockCrab)
- `PlayerState` (health, max_health, stamina, equipped_tool)
- `PlayerInput` (attack, tool_use)
- `Inventory`, `ItemId`
- `MapId` (Mine, MineEntrance)
- `GameState` (Mining state)
- `GridPosition`, `LogicalPosition`, `YSorted`
- Events: `MapTransitionEvent`, `StaminaDrainEvent`, `ItemPickupEvent`, `ToolUseEvent`, `MonsterSlainEvent`, `PlaySfxEvent`, `ToastEvent`
- Constants: `TILE_SIZE` (16.0), `Z_ENTITY_BASE` (100.0), `MAX_HEALTH` (100.0)

## Quantitative Targets
- 20 mine floors
- Elevator: unlocks every 5 floors (5, 10, 15, 20)
- Rock types per floor depth:
  - Floors 1-5: Stone (100%), Copper ore (30%)
  - Floors 6-10: Stone (60%), Copper (40%), Iron ore (20%)
  - Floors 11-15: Stone (40%), Iron (40%), Gold ore (20%), gems (5%)
  - Floors 16-20: Stone (30%), Gold (30%), Iridium ore (10%), gems (10%)
- Rocks per floor: 8-15 (increases with depth)
- Monster spawn per floor:
  - Floors 1-5: 1-2 GreenSlime
  - Floors 6-10: 2-3 GreenSlime + 1 Bat
  - Floors 11-15: 2-3 Bat + 1-2 RockCrab
  - Floors 16-20: 2-4 mixed
- Monster stats:
  - GreenSlime: HP 20, DMG 5, Speed 30
  - Bat: HP 15, DMG 8, Speed 50
  - RockCrab: HP 40, DMG 12, Speed 15
- Gems: quartz (40%), amethyst (25%), emerald (15%), ruby (12%), diamond (8%)
- Stamina cost: 4 per pickaxe swing × tier multiplier
- Ladder: 5% per rock broken to reveal ladder

## Constants & Formulas
- Rock health: 3 (stone) to 6 (ore), pickaxe tier reduces hits needed
- Pickaxe damage: Basic 1, Copper 2, Iron 3, Gold 4, Iridium 5
- Combat damage to player: `monster.damage - (defense_buff * 0.5)`
- Combat damage from player: `weapon_damage * (1.0 + attack_buff * 0.15)`
- Ladder probability: `0.05 + (rocks_broken_this_floor * 0.02)`, max 0.3
- Elevator unlock: `floor % 5 == 0` and `floor > deepest_floor_reached`

## Key Systems
1. `floor_generation` — procedurally generate rocks, monsters, ladder position
2. `rock_breaking` — on pickaxe use, damage rock, drop items on destroy
3. `combat_system` — player attacks (bump), monster AI (chase/attack), damage calculation
4. `ladder_system` — reveal ladder, transition to next floor on interact
5. `elevator_system` — at floor 0, show elevator menu for unlocked floors
6. `mine_transition` — handle entering/exiting mine, floor transitions
7. `mine_hud` — display current floor, health bar, enemy count

## Does NOT Handle
- Pickaxe upgrades (economy/blacksmith domain)
- Ore smelting (crafting/machines domain)
- Gem/ore item definitions (data domain)
- Monster drop definitions (data domain)
- Main game HUD (ui domain)
