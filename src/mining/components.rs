//! Mining-local components and marker types.
//!
//! These are ECS components used only within the mining domain.
//! Cross-domain types live in `crate::shared`.

use bevy::prelude::*;

/// Marker for all entities belonging to the current mine floor.
/// Used for bulk despawning when changing floors or leaving the mine.
#[derive(Component, Debug)]
pub struct MineFloorEntity;

/// Marker for mine floor tile sprites.
#[derive(Component, Debug)]
pub struct MineTile;

/// Marker for the ladder entity.
#[derive(Component, Debug)]
pub struct MineLadder {
    pub revealed: bool,
}

/// Marker for the mine entrance / exit tile at the bottom.
#[derive(Component, Debug)]
pub struct MineExit;

/// Grid position specifically for mine entities (mirrors shared::GridPosition
/// but we'll just use the shared one).
/// We use this tag to identify which mine grid cell something occupies.
#[derive(Component, Debug, Clone, Copy)]
pub struct MineGridPos {
    pub x: i32,
    pub y: i32,
}

/// Tracks enemy movement cooldown so they don't move every frame.
#[derive(Component, Debug)]
pub struct EnemyMoveTick {
    pub timer: Timer,
}

/// Tracks enemy attack cooldown.
#[derive(Component, Debug)]
pub struct EnemyAttackCooldown {
    pub timer: Timer,
}

/// Resource tracking the current floor's state.
#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub struct ActiveFloor {
    pub floor: u8,
    pub total_rocks: usize,
    pub rocks_remaining: usize,
    pub ladder_revealed: bool,
    /// Player grid position in the mine.
    pub player_grid_x: i32,
    pub player_grid_y: i32,
    /// Is the floor fully spawned?
    pub spawned: bool,
}

impl Default for ActiveFloor {
    fn default() -> Self {
        Self {
            floor: 0,
            total_rocks: 0,
            rocks_remaining: 0,
            ladder_revealed: false,
            player_grid_x: 12,
            player_grid_y: 1,
            spawned: false,
        }
    }
}

/// Resource: when set to true, the mine systems should generate and spawn a new floor.
#[derive(Resource, Debug, Default)]
pub struct FloorSpawnRequest {
    pub pending: bool,
    pub floor: u8,
}

/// Tracks whether we are currently inside the mine (to gate systems).
#[derive(Resource, Debug, Default)]
pub struct InMine(pub bool);

/// Invincibility frames after taking damage.
#[derive(Resource, Debug)]
pub struct PlayerIFrames {
    pub timer: Timer,
}

impl Default for PlayerIFrames {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

/// The elevator UI state (when player is choosing a floor at the elevator).
#[derive(Resource, Debug, Default)]
pub struct ElevatorUiOpen(pub bool);
