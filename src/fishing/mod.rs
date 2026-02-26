use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

// ─── Sub-modules ────────────────────────────────────────────────────────────
mod cast;
mod bite;
mod fish_select;
mod minigame;
mod render;
mod resolve;

pub use cast::*;
pub use bite::*;
pub use fish_select::*;
pub use minigame::*;
pub use render::*;
pub use resolve::*;

// ─── Plugin ─────────────────────────────────────────────────────────────────

pub struct FishingPlugin;

impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<FishingState>()
            .init_resource::<FishingMinigameState>()
            // Systems that run in Playing state (cast detection)
            .add_systems(
                Update,
                (
                    cast::handle_tool_use_for_fishing,
                    cast::update_bite_timer,
                    cast::handle_bite_reaction_window,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Minigame systems that run in Fishing state
            .add_systems(
                Update,
                (
                    minigame::update_fish_zone,
                    minigame::update_catch_bar,
                    minigame::update_progress,
                    minigame::check_minigame_result,
                )
                    .chain()
                    .run_if(in_state(GameState::Fishing)),
            )
            // Setup/teardown on state transitions
            .add_systems(OnEnter(GameState::Fishing), render::spawn_minigame_ui)
            .add_systems(OnExit(GameState::Fishing), render::despawn_minigame_ui)
            // Bobber animation runs in both Playing (while waiting for bite) and Fishing
            .add_systems(
                Update,
                render::animate_bobber.run_if(
                    in_state(GameState::Playing).or(in_state(GameState::Fishing)),
                ),
            )
            // Escape key to cancel fishing (in Playing state while waiting)
            .add_systems(
                Update,
                cast::handle_cancel_fishing.run_if(in_state(GameState::Playing)),
            );
    }
}

// ─── Fishing State Resource ──────────────────────────────────────────────────

/// Phase of the fishing sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FishingPhase {
    #[default]
    Idle,
    /// Bobber is in the water, waiting for a bite.
    WaitingForBite,
    /// Fish has bitten — player must press Space within the reaction window.
    BitePending,
    /// Active minigame.
    Minigame,
}

#[derive(Resource, Debug, Default)]
pub struct FishingState {
    pub phase: FishingPhase,
    /// Tile coords where the bobber landed.
    pub bobber_tile: (i32, i32),
    /// World position of the bobber.
    pub bobber_pos: Vec2,
    /// Counts down until a bite occurs.
    pub bite_timer: Option<Timer>,
    /// Window in which player can react to the bite.
    pub reaction_timer: Option<Timer>,
    /// The fish that has been selected for this cast.
    pub selected_fish_id: Option<ItemId>,
    /// Whether bait is currently equipped.
    pub bait_equipped: bool,
    /// Whether tackle is currently equipped.
    pub tackle_equipped: bool,
    /// Tier of the equipped fishing rod.
    pub rod_tier: ToolTier,
}

impl FishingState {
    pub fn reset(&mut self) {
        self.phase = FishingPhase::Idle;
        self.bite_timer = None;
        self.reaction_timer = None;
        self.selected_fish_id = None;
    }
}

// ─── Minigame State Resource ─────────────────────────────────────────────────

/// All runtime state for the fishing minigame.
#[derive(Resource, Debug)]
pub struct FishingMinigameState {
    /// Fish zone center position, 0.0 (bottom) to 100.0 (top).
    pub fish_zone_center: f32,
    /// Current velocity of the fish zone.
    pub fish_zone_velocity: f32,
    /// Direction timer — fish changes direction occasionally.
    pub direction_change_timer: Timer,
    /// Catch bar center position, 0.0 to 100.0.
    pub catch_bar_center: f32,
    /// Catch bar half-height (25 base, boosted by tackle).
    pub catch_bar_half: f32,
    /// Fish zone half-height (based on difficulty — easier fish have bigger zones).
    pub fish_zone_half: f32,
    /// Progress: 0.0 → 100.0. Fills when overlapping, drains when not.
    pub progress: f32,
    /// Difficulty of the current fish (0.0–1.0).
    pub fish_difficulty: f32,
    /// Whether Space is currently held.
    pub space_held: bool,
    /// Time since minigame started (for sfx pacing etc.)
    pub elapsed: f32,
    /// Quick sound pulse for overlap
    pub overlap_sfx_cooldown: f32,
}

impl Default for FishingMinigameState {
    fn default() -> Self {
        Self {
            fish_zone_center: 50.0,
            fish_zone_velocity: 0.0,
            direction_change_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            catch_bar_center: 50.0,
            catch_bar_half: 12.5,
            fish_zone_half: 15.0,
            progress: 0.0,
            fish_difficulty: 0.5,
            space_held: false,
            elapsed: 0.0,
            overlap_sfx_cooldown: 0.0,
        }
    }
}

impl FishingMinigameState {
    pub fn setup(&mut self, difficulty: f32, rod_tier: ToolTier, tackle_equipped: bool) {
        let mut rng = rand::thread_rng();
        self.fish_zone_center = rng.gen_range(20.0..80.0);
        self.fish_zone_velocity = 0.0;
        self.catch_bar_center = 50.0;
        self.progress = 0.0;
        self.fish_difficulty = difficulty;
        self.elapsed = 0.0;
        self.overlap_sfx_cooldown = 0.0;
        self.space_held = false;

        // Fish zone size: easier fish have bigger zones (more forgiving)
        // Difficulty 0.0 → fish_zone_half = 22.0
        // Difficulty 1.0 → fish_zone_half = 8.0
        self.fish_zone_half = 22.0 - difficulty * 14.0;

        // Catch bar size: base 12.5 per half (25 total)
        // Tackle adds 25% → 15.625 half
        let tackle_bonus = if tackle_equipped { 1.25 } else { 1.0 };
        let tier_bonus = match rod_tier {
            ToolTier::Basic => 1.0,
            ToolTier::Copper => 1.05,
            ToolTier::Iron => 1.10,
            ToolTier::Gold => 1.15,
            ToolTier::Iridium => 1.20,
        };
        self.catch_bar_half = 12.5 * tackle_bonus * tier_bonus;

        // Reset timer with randomized first direction change
        let first_change = rng.gen_range(0.8..2.5);
        self.direction_change_timer = Timer::from_seconds(first_change, TimerMode::Once);
    }

    /// Returns true if the catch bar is overlapping the fish zone.
    pub fn is_overlapping(&self) -> bool {
        let catch_lo = self.catch_bar_center - self.catch_bar_half;
        let catch_hi = self.catch_bar_center + self.catch_bar_half;
        let fish_lo = self.fish_zone_center - self.fish_zone_half;
        let fish_hi = self.fish_zone_center + self.fish_zone_half;
        catch_hi > fish_lo && catch_lo < fish_hi
    }
}

// ─── Marker Components ───────────────────────────────────────────────────────

/// Marks the bobber entity.
#[derive(Component)]
pub struct Bobber {
    pub bob_timer: Timer,
    pub bob_direction: f32,
    pub original_y: f32,
}

/// Marks the fishing minigame root UI container.
#[derive(Component)]
pub struct MinigameRoot;

/// The dark background bar for the minigame.
#[derive(Component)]
pub struct MinigameBgBar;

/// The fish zone (red/orange block).
#[derive(Component)]
pub struct MinigameFishZone;

/// The catch bar (green block).
#[derive(Component)]
pub struct MinigameCatchBar;

/// The progress bar fill.
#[derive(Component)]
pub struct MinigameProgressFill;

/// Progress bar background.
#[derive(Component)]
pub struct MinigameProgressBg;
