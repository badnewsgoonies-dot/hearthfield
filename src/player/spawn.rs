use bevy::prelude::*;
use crate::shared::*;

/// Starting grid position on the farm (roughly center of the farmable area).
const SPAWN_GRID_X: i32 = 10;
const SPAWN_GRID_Y: i32 = 10;

/// Spawn the player entity with all necessary components.
/// Runs once on `OnEnter(GameState::Playing)`.
pub fn spawn_player(
    mut commands: Commands,
    existing: Query<Entity, With<Player>>,
) {
    // Guard: don't double-spawn if returning to Playing state.
    if !existing.is_empty() {
        return;
    }

    let world_x = SPAWN_GRID_X as f32 * TILE_SIZE + TILE_SIZE * 0.5;
    let world_y = SPAWN_GRID_Y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

    commands.spawn((
        // Tag
        Player,
        // Movement state
        PlayerMovement::default(),
        // Grid position for tile-based lookups
        GridPosition::new(SPAWN_GRID_X, SPAWN_GRID_Y),
        // Placeholder sprite — a blue square
        Sprite {
            color: Color::srgb(0.2, 0.5, 0.8),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        // World-space transform. Z = 10 so the player draws above terrain.
        Transform::from_translation(Vec3::new(world_x, world_y, 10.0)),
        // Required for rendering
        Visibility::default(),
    ));
}
