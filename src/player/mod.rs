mod camera;
pub mod interact_dispatch;
mod interaction;
pub mod item_use;
mod movement;
mod spawn;
pub mod tool_anim;
mod tools;

use crate::shared::*;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // -- Local resources --
        app.init_resource::<ToolCooldown>();
        app.init_resource::<CollisionMap>();
        app.init_resource::<CameraSnap>();
        app.init_resource::<PlayerSpriteData>();
        app.init_resource::<ActionSpriteData>();

        // -- Spawn player when we enter Playing --
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn::spawn_player, interaction::grant_starter_items),
        );

        // -- Interaction dispatchers: run BEFORE all legacy F-key systems --
        app.add_systems(
            Update,
            interact_dispatch::dispatch_world_interaction
                .before(interaction::item_pickup_check)
                .in_set(UpdatePhase::Intent)
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            item_use::dispatch_item_use
                .in_set(UpdatePhase::Intent)
                .run_if(in_state(GameState::Playing)),
        );

        // -- Systems that run every frame while Playing --
        app.add_systems(
            Update,
            (
                // tool_use sets ToolUse anim state; must run before movement reads it
                tools::tool_use.before(movement::player_movement),
                movement::player_movement,
                movement::footstep_sfx.after(movement::player_movement),
                movement::animate_player_sprite
                    .after(movement::player_movement)
                    .after(tool_anim::animate_tool_use),
                tool_anim::animate_tool_use.after(movement::player_movement),
                tool_anim::handle_tool_impact_sfx.after(tool_anim::animate_tool_use),
                tools::tool_cycle,
                tools::stamina_drain_handler,
                tools::stamina_low_warning,
                interaction::item_pickup_check,
                interaction::add_items_to_inventory,
                interaction::map_transition_check,
                interaction::handle_map_transition,
                interaction::check_stamina_consequences,
            )
                .in_set(UpdatePhase::Simulation)
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            camera::camera_follow_player
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::Playing)),
        );

        // -- DayEnd handling: only runs in Playing state to ensure world/NPC
        // handlers (also gated on Playing) process the MapTransitionEvent --
        app.add_systems(
            Update,
            interaction::handle_day_end
                .in_set(UpdatePhase::Reactions)
                .run_if(in_state(GameState::Playing)),
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

/// Holds the loaded character_actions.png atlas handles for tool animations.
#[derive(Resource, Default)]
pub struct ActionSpriteData {
    pub loaded: bool,
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Drives walk-cycle frames based on distance traveled, not time.
/// Prevents "ice skating" when speed changes from buffs.
#[derive(Component, Debug)]
pub struct DistanceAnimator {
    /// Position at last frame advance. Compared against LogicalPosition.
    pub last_pos: Vec2,
    /// Accumulated distance since last frame change.
    pub distance_budget: f32,
    /// World-space pixels of movement needed to advance one frame.
    pub pixels_per_frame: f32,
    /// Number of frames per animation row.
    pub frames_per_row: usize,
    /// Current frame within the row (0-based).
    pub current_frame: usize,
}

impl Default for DistanceAnimator {
    fn default() -> Self {
        Self {
            last_pos: Vec2::ZERO,
            distance_budget: 0.0,
            pixels_per_frame: 6.0,
            frames_per_row: 4,
            current_frame: 0,
        }
    }
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

/// Collision map for the current area.
#[derive(Resource, Default)]
pub struct CollisionMap {
    pub solid_tiles: std::collections::HashSet<(i32, i32)>,
    pub bounds: (i32, i32, i32, i32),
    pub initialised: bool,
}

/// When set, camera_follow_player snaps instantly instead of lerping.
/// Uses a 2-frame countdown so the snap survives system ordering races
/// between the world map loader and the camera on the transition frame.
#[derive(Resource, Default)]
pub struct CameraSnap {
    /// Frames remaining before snap fires. 0 = no snap pending.
    /// Set to 2 by handle_map_transition; camera decrements each frame
    /// and snaps when it reaches 1 (ensuring WorldMap is updated).
    pub frames_remaining: u8,
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
