# Worker: Player Domain (Wave 1)

## Scope (mechanically enforced)
You may only create/modify files under: `dlc/police/src/domains/player/`

## Required reading (read BEFORE writing code)
1. `dlc/police/docs/spec.md`
2. `dlc/police/docs/domains/player.md` (CRITICAL)
3. `dlc/police/src/shared/mod.rs`
4. `dlc/police/src/main.rs`

## Interpretation contract
Before coding, extract from player.md:
- Grid-based collision (not physics)
- Placeholder colored sprites (not atlas)
- Fatigue AND stress as separate constraints (not merged)

## Required imports (from crate::shared)
- `Player`, `PlayerMovement`, `PlayerState`, `PlayerInput`, `InputContext`
- `FatigueChangeEvent`, `StressChangeEvent`, `MapTransitionEvent`
- `MapId`, `Facing`, `GridPosition`, `Equipment`
- `GameState`, `UpdatePhase`
- `TILE_SIZE`, `PIXEL_SCALE`, `MAX_FATIGUE`, `MAX_STRESS`

## Deliverables
Create `dlc/police/src/domains/player/mod.rs` containing:
- `pub struct PlayerPlugin;` implementing `Plugin`
- `read_keyboard_input` — UpdatePhase::Input, gated on Playing
  - WASD/arrows → PlayerInput.move_dir, F → interact, Escape → menu, Shift → run
- `spawn_player` — OnEnter(GameState::Playing)
  - Spawn entity with Player, PlayerMovement, GridPosition, Sprite (blue 16x16 square), Transform
  - Position at grid (16, 20) on PrecinctInterior — use TILE_SIZE * PIXEL_SCALE for world coords
- `move_player` — UpdatePhase::Simulation, gated on Playing
  - Read PlayerInput.move_dir, apply speed * delta_secs
  - Check target grid tile against collision (read CollisionMap resource if available, otherwise skip)
  - Update Transform and Facing
  - Running: speed * 1.5
- `camera_follow` — UpdatePhase::Presentation
  - Lerp camera transform to player, pixel-snap
- `apply_fatigue_stress` — UpdatePhase::Reactions
  - Read FatigueChangeEvent → apply to PlayerState.fatigue, clamp 0..MAX_FATIGUE
  - Read StressChangeEvent → apply to PlayerState.stress, clamp 0..MAX_STRESS
- `despawn_player` — OnExit(GameState::Playing)

## Quantitative targets
- Walk speed: 80.0 px/s
- Run speed: 120.0 px/s
- Spawn at grid (16, 20) which is world position (16 * 16 * 3, 20 * 16 * 3) = (768, 960)
- Player sprite: blue colored square, 16x16 pixels (will be scaled by PIXEL_SCALE via camera)

## Failure patterns to avoid
- Importing bevy_rapier or physics libraries
- Creating a sprite atlas loader
- Redefining PlayerState or PlayerInput locally
- Forgetting OnExit cleanup (despawn_player)
- Using raw Vec3 math instead of grid * TILE_SIZE * PIXEL_SCALE

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```

## When done
Write report to `dlc/police/status/workers/player.md`
