# Player Domain Spec — Precinct

## Purpose
Player entity: spawning, movement, collision, input reading, fatigue/stress application.

## Scope
`src/domains/player/` — owns player entity and movement.

## What This Domain Does
- Spawn the player entity on `OnEnter(GameState::Playing)` at precinct interior spawn point
- Read `PlayerInput` resource and apply 4-directional movement (speed = 80.0 px/s)
- Collision detection against solid tiles (check before moving)
- Apply `FatigueChangeEvent` and `StressChangeEvent` to `PlayerState`
- Clamp fatigue to 0..MAX_FATIGUE and stress to 0..MAX_STRESS
- Emit `MapTransitionEvent` when player reaches a transition zone
- Camera follow (pixel-snapped lerp to player position)

## What This Domain Does NOT Do
- Input capture from keyboard (that's handled inline or by a thin input system)
- UI rendering
- Case/evidence/NPC/economy logic

## Key Types (import from crate::shared)
- `Player` (Component), `PlayerMovement` (Component)
- `PlayerState` (Resource), `PlayerInput` (Resource), `InputContext` (Resource)
- `FatigueChangeEvent`, `StressChangeEvent`, `MapTransitionEvent`
- `MapId`, `Facing`, `GridPosition`
- `UpdatePhase` — input reading in `Input`, movement in `Simulation`
- Constants: `TILE_SIZE`, `PIXEL_SCALE`, `MAX_FATIGUE`, `MAX_STRESS`

## Systems to Implement
1. `read_keyboard_input` — `UpdatePhase::Input`, gated on `GameState::Playing`
   - WASD/arrows → PlayerInput.move_dir
   - F/E → interact, Escape → menu, Tab → notebook, R → radio
   - Shift → run
2. `spawn_player` — `OnEnter(GameState::Playing)`, spawn entity with:
   - `Player` component, `PlayerMovement::default()`, `GridPosition`
   - `Sprite` (placeholder color square for now — Wave 1 has no sprites)
   - `Transform` at spawn position (precinct interior: grid 5, 5)
3. `move_player` — `UpdatePhase::Simulation`, gated on `GameState::Playing`
   - Read PlayerInput.move_dir, multiply by speed * delta
   - Check collision before applying (grid-based: target tile must not be solid)
   - Update Transform, update Facing
   - If running: speed * 1.5, fatigue cost 1 per 5 game-minutes
4. `camera_follow` — `UpdatePhase::Presentation`
   - Lerp camera to player position, pixel-snap (round to nearest pixel)
5. `apply_fatigue_stress` — `UpdatePhase::Reactions`
   - Read FatigueChangeEvent, apply to PlayerState.fatigue, clamp
   - Read StressChangeEvent, apply to PlayerState.stress, clamp
6. `check_map_transitions` — `UpdatePhase::Simulation`
   - If player position overlaps a transition zone → emit MapTransitionEvent

## Quantitative Targets
- Player speed: 80.0 px/s walk, 120.0 px/s run
- Spawn position: grid (5, 5) on MapId::PrecinctInterior
- Fatigue clamp: 0.0 to 100.0
- Stress clamp: 0.0 to 100.0

## Decision Fields
- **Preferred**: grid-based collision (check target tile before moving)
- **Tempting alternative**: continuous physics collision
- **Consequence**: physics collision is overkill for a tile-based game, adds complexity
- **Drift cue**: worker imports bevy_rapier or writes continuous AABB checks

- **Preferred**: placeholder colored sprite for Wave 1
- **Tempting alternative**: implement full sprite atlas system now
- **Consequence**: atlas system is premature when no art assets exist yet
- **Drift cue**: worker spends time on atlas loading, animation frames

## Plugin Export
```rust
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // OnEnter(Playing) → spawn_player
        // Update systems in appropriate UpdatePhases
    }
}
```

## Tests (minimum 5)
1. Player spawns at correct position on GameState::Playing
2. Movement updates transform when input provided
3. Player does NOT move into solid tiles
4. Fatigue changes apply and clamp correctly
5. Stress changes apply and clamp correctly
