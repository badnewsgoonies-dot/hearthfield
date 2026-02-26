mod movement;
mod tools;
mod camera;
mod interaction;
mod spawn;

use bevy::prelude::*;
use crate::shared::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // -- Local resources --
        app.init_resource::<ToolCooldown>();
        app.init_resource::<CollisionMap>();
        app.init_resource::<PlayerSpriteData>();

        // -- Spawn player when we enter Playing --
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn::spawn_player,
        );

        // -- Systems that run every frame while Playing --
        app.add_systems(
            Update,
            (
                movement::player_movement,
                movement::animate_player_sprite,
                tools::tool_cycle,
                tools::tool_use,
                tools::stamina_drain_handler,
                interaction::item_pickup_check,
                interaction::add_items_to_inventory,
                interaction::map_transition_check,
                interaction::handle_map_transition,
                camera::camera_follow_player,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // -- DayEnd handling runs regardless of sub-state so we never miss it --
        app.add_systems(
            Update,
            interaction::handle_day_end,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Local resources (player-domain only)
// ═══════════════════════════════════════════════════════════════════════════

/// Holds the loaded character spritesheet handles so spawn_player can
/// reference them without reloading on every call.
#[derive(Resource, Default)]
pub struct PlayerSpriteData {
    pub loaded: bool,
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Per-entity walk animation state.
#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
}

/// Cooldown timer to prevent tool spam.
#[derive(Resource)]
pub struct ToolCooldown {
    pub timer: Timer,
}

impl Default for ToolCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

/// Collision map for the current area. Populated by the world domain
/// via the shared FarmState / MapId. The player domain builds a local
/// lookup each time the map changes so movement checks are O(1).
#[derive(Resource, Default)]
pub struct CollisionMap {
    /// Set of (grid_x, grid_y) positions that are solid / impassable.
    pub solid_tiles: std::collections::HashSet<(i32, i32)>,
    /// Map boundaries (min_x, max_x, min_y, max_y) in grid coordinates.
    pub bounds: (i32, i32, i32, i32),
    /// Whether the collision map has been initialised for the current map.
    pub initialised: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers shared across sub-modules
// ═══════════════════════════════════════════════════════════════════════════

/// Stamina cost for each tool kind.
pub fn stamina_cost(tool: &ToolKind) -> f32 {
    match tool {
        ToolKind::Hoe => 4.0,
        ToolKind::WateringCan => 3.0,
        ToolKind::Axe => 6.0,
        ToolKind::Pickaxe => 6.0,
        ToolKind::FishingRod => 4.0,
        ToolKind::Scythe => 2.0,
    }
}

/// Convert world-space pixel coordinates to grid position.
pub fn world_to_grid(x: f32, y: f32) -> (i32, i32) {
    (
        (x / TILE_SIZE).floor() as i32,
        (y / TILE_SIZE).floor() as i32,
    )
}

/// Convert grid coordinates to world-space pixel position (tile center).
pub fn grid_to_world(gx: i32, gy: i32) -> (f32, f32) {
    (
        gx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        gy as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

/// The ordered list of tools for cycling with Q/E.
pub const TOOL_ORDER: [ToolKind; 6] = [
    ToolKind::Hoe,
    ToolKind::WateringCan,
    ToolKind::Axe,
    ToolKind::Pickaxe,
    ToolKind::FishingRod,
    ToolKind::Scythe,
];

/// Get the facing-direction offset as a grid delta.
pub fn facing_offset(facing: &Facing) -> (i32, i32) {
    match facing {
        Facing::Up => (0, 1),
        Facing::Down => (0, -1),
        Facing::Left => (-1, 0),
        Facing::Right => (1, 0),
    }
}
