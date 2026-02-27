use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::shared::*;

// ─── Sub-modules ────────────────────────────────────────────────────────────
mod cast;
mod bite;
mod fish_select;
mod minigame;
mod render;
mod resolve;
pub mod skill;
pub mod treasure;
pub mod legendaries;

// ─── Plugin ─────────────────────────────────────────────────────────────────

pub struct FishingPlugin;

impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<FishingState>()
            .init_resource::<FishingMinigameState>()
            .init_resource::<FishEncyclopedia>()
            .init_resource::<skill::FishingSkill>()
            // Internal events
            .add_event::<skill::FishingLevelUpEvent>()
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
            // Skill update runs after item pickup events fire (PostUpdate ensures
            // that ItemPickupEvent written this frame are available).
            .add_systems(
                PostUpdate,
                skill::update_fishing_skill.run_if(
                    in_state(GameState::Playing).or(in_state(GameState::Fishing)),
                ),
            )
            // Setup/teardown on state transitions
            .add_systems(OnEnter(GameState::Fishing), render::spawn_minigame_ui)
            .add_systems(OnExit(GameState::Fishing), render::despawn_minigame_ui)
            // Minigame visual updates (color feedback on progress bar)
            .add_systems(
                Update,
                render::update_progress_fill_color.run_if(in_state(GameState::Fishing)),
            )
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

// ─── Fish Encyclopedia Resource ──────────────────────────────────────────────

/// Tracks every species the player has ever caught.
#[derive(Resource, Debug, Clone, Default)]
pub struct FishEncyclopedia {
    /// fish_id → CaughtFishEntry
    pub entries: HashMap<String, CaughtFishEntry>,
}

impl FishEncyclopedia {
    /// Record a successful catch. Returns `true` if this is the first time this
    /// species has been caught (useful for triggering a "New fish!" toast).
    pub fn record_catch(&mut self, fish_id: &str, day: u32, season: Season) -> bool {
        if let Some(entry) = self.entries.get_mut(fish_id) {
            entry.times_caught += 1;
            false
        } else {
            self.entries.insert(
                fish_id.to_string(),
                CaughtFishEntry {
                    fish_id: fish_id.to_string(),
                    times_caught: 1,
                    first_caught_day: day,
                    first_caught_season: season,
                },
            );
            true
        }
    }

    /// How many unique species have been caught.
    #[allow(dead_code)]
    pub fn unique_species(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CaughtFishEntry {
    pub fish_id: String,
    pub times_caught: u32,
    pub first_caught_day: u32,
    pub first_caught_season: Season,
}

// ─── Tackle Types ────────────────────────────────────────────────────────────

/// Specific tackle items that modify minigame parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TackleKind {
    /// No tackle equipped — no modifier.
    #[default]
    None,
    /// Spinner: enlarges the fish zone by 25% (easier target).
    Spinner,
    /// Trap Bobber: slows progress drain by 50% (more forgiving).
    TrapBobber,
    /// Lead Bobber: reduces catch bar fall speed by 30% (easier to hold up).
    LeadBobber,
}

impl TackleKind {
    /// Derive tackle kind from an inventory item ID.
    #[allow(dead_code)]
    pub fn from_item_id(id: &str) -> Self {
        match id {
            "spinner" => TackleKind::Spinner,
            "trap_bobber" => TackleKind::TrapBobber,
            "lead_bobber" => TackleKind::LeadBobber,
            _ => TackleKind::None,
        }
    }

    /// User-facing description of the tackle effect.
    #[allow(dead_code)]
    pub fn effect_description(self) -> Option<&'static str> {
        match self {
            TackleKind::None => None,
            TackleKind::Spinner => Some("Spinner: larger fish zone"),
            TackleKind::TrapBobber => Some("Trap Bobber: slower progress drain"),
            TackleKind::LeadBobber => Some("Lead Bobber: slower catch bar fall"),
        }
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

#[derive(Resource, Debug)]
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
    /// Whether tackle is currently equipped (any kind).
    pub tackle_equipped: bool,
    /// Specific tackle type that is equipped (None if no tackle).
    pub tackle_kind: TackleKind,
    /// Tier of the equipped fishing rod.
    pub rod_tier: ToolTier,
}

impl Default for FishingState {
    fn default() -> Self {
        Self {
            phase: FishingPhase::Idle,
            bobber_tile: (0, 0),
            bobber_pos: Vec2::ZERO,
            bite_timer: None,
            reaction_timer: None,
            selected_fish_id: None,
            bait_equipped: false,
            tackle_equipped: false,
            tackle_kind: TackleKind::None,
            rod_tier: ToolTier::Basic,
        }
    }
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
    /// Multiplier on the progress drain rate (1.0 = normal; 0.5 = Trap Bobber).
    pub progress_drain_multiplier: f32,
    /// Multiplier on the catch bar fall speed (1.0 = normal; 0.7 = Lead Bobber).
    pub catch_fall_multiplier: f32,
    /// Total time (seconds) the catch bar was overlapping the fish zone this game.
    pub overlap_time_total: f32,
    /// Total time (seconds) the minigame has been running (excluding the ramp-up grace period).
    pub minigame_total_time: f32,
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
            progress_drain_multiplier: 1.0,
            catch_fall_multiplier: 1.0,
            overlap_time_total: 0.0,
            minigame_total_time: 0.0,
        }
    }
}

impl FishingMinigameState {
    /// Set up the minigame without skill bonuses applied (kept for compatibility).
    pub fn setup(&mut self, difficulty: f32, rod_tier: ToolTier, tackle_kind: TackleKind) {
        self.setup_with_skill(difficulty, rod_tier, tackle_kind, &skill::FishingSkill::default());
    }

    /// Set up the minigame incorporating the player's fishing skill bonuses.
    ///
    /// `FishingSkill::catch_zone_bonus` expands the catch bar so experienced
    /// anglers have an easier time.
    pub fn setup_with_skill(
        &mut self,
        difficulty: f32,
        rod_tier: ToolTier,
        tackle_kind: TackleKind,
        fishing_skill: &skill::FishingSkill,
    ) {
        let mut rng = rand::thread_rng();
        self.fish_zone_center = rng.gen_range(20.0..80.0);
        self.fish_zone_velocity = 0.0;
        self.catch_bar_center = 50.0;
        self.progress = 0.0;
        self.fish_difficulty = difficulty;
        self.elapsed = 0.0;
        self.overlap_sfx_cooldown = 0.0;
        self.space_held = false;
        self.overlap_time_total = 0.0;
        self.minigame_total_time = 0.0;

        // Fish zone size: easier fish have bigger zones (more forgiving).
        // Difficulty 0.0 → fish_zone_half = 22.0
        // Difficulty 1.0 → fish_zone_half = 8.0
        let base_fish_zone = 22.0 - difficulty * 14.0;

        // Spinner: enlarges the fish zone by 25% (makes the target bigger).
        self.fish_zone_half = match tackle_kind {
            TackleKind::Spinner => base_fish_zone * 1.25,
            _ => base_fish_zone,
        };

        // Catch bar size: base 12.5 per half (25 total).
        // Rod tier grants a small size bonus.
        let tier_bonus = match rod_tier {
            ToolTier::Basic => 1.0,
            ToolTier::Copper => 1.05,
            ToolTier::Iron => 1.10,
            ToolTier::Gold => 1.15,
            ToolTier::Iridium => 1.20,
        };
        // Generic tackle bonus (any tackle = +25% catch bar) is kept for non-Spinner tackle.
        // Spinner's benefit is the fish zone, not the catch bar.
        let catch_bar_tackle_bonus = match tackle_kind {
            TackleKind::None => 1.0,
            TackleKind::Spinner => 1.0,    // Spinner helps via fish zone instead
            TackleKind::TrapBobber => 1.0, // TrapBobber helps via drain rate instead
            TackleKind::LeadBobber => 1.25, // LeadBobber was the old generic +25%
        };
        // Apply fishing skill catch_zone_bonus on top of tackle and tier.
        let skill_bonus = 1.0 + fishing_skill.catch_zone_bonus;
        self.catch_bar_half = 12.5 * catch_bar_tackle_bonus * tier_bonus * skill_bonus;

        // Trap Bobber: slow progress drain rate (stored on minigame state for
        // update_progress to read at runtime).
        // Lead Bobber: reduce catch bar fall speed.
        // These modifiers are stored as multipliers read by the systems.
        self.progress_drain_multiplier = match tackle_kind {
            TackleKind::TrapBobber => 0.5,
            _ => 1.0,
        };
        self.catch_fall_multiplier = match tackle_kind {
            TackleKind::LeadBobber => 0.7,
            _ => 1.0,
        };

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

    /// Returns `true` if the player achieved a "perfect catch" — they kept the
    /// catch bar in the fish zone for at least 90% of the total minigame duration.
    ///
    /// Uses `minigame_total_time` (accumulated in `update_progress`) to judge
    /// the denominator, ignoring the initial grace period.
    pub fn is_perfect_catch(&self) -> bool {
        if self.minigame_total_time < 0.5 {
            // Too short to be meaningful — don't award perfect bonus
            return false;
        }
        let ratio = self.overlap_time_total / self.minigame_total_time;
        ratio >= 0.90
    }
}

// ─── Marker Components ───────────────────────────────────────────────────────

/// Marks the bobber entity.
#[derive(Component)]
pub struct Bobber {
    pub bob_timer: Timer,
    pub _bob_direction: f32,
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
