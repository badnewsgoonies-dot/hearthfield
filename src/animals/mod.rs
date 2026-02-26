use bevy::prelude::*;
use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// Sub-modules
// ─────────────────────────────────────────────────────────────────────────────
mod spawning;
mod movement;
mod feeding;
mod products;
mod day_end;
mod interaction;
mod rendering;

pub use spawning::*;
pub use movement::*;
pub use feeding::*;
pub use products::*;
pub use day_end::*;
pub use interaction::*;
pub use rendering::*;

// ─────────────────────────────────────────────────────────────────────────────
// Private ECS components (internal to the animals domain)
// ─────────────────────────────────────────────────────────────────────────────

/// Marks an entity as the feed-trough object on the farm.
#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct FeedTrough {
    pub grid_x: i32,
    pub grid_y: i32,
}

/// Wander AI timer: fires every 2-4 seconds to pick a new direction.
#[derive(Component, Debug, Clone)]
pub struct WanderAi {
    pub timer: Timer,
    /// Target world-space position we are walking toward (None = idle).
    pub target: Option<Vec2>,
    /// Pen boundaries in world space (min_x, min_y, max_x, max_y).
    pub pen_min: Vec2,
    pub pen_max: Vec2,
    /// Movement speed in pixels/sec.
    pub speed: f32,
}

/// Floating text or heart effect spawned on petting / product collection.
#[derive(Component, Debug, Clone)]
pub struct FloatingFeedback {
    pub lifetime: Timer,
    pub velocity: Vec2,
}

/// Tag on an entity that is the visual indicator for "product ready".
#[derive(Component, Debug)]
pub struct ProductReadyIndicator {
    pub owner: Entity,
}

/// Tracks how many consecutive days this animal has gone without being fed.
/// Reset to 0 when fed_today is true at day-end. After 3+ days the animal
/// refuses to produce anything until it is fed again.
#[derive(Component, Debug, Clone, Default)]
pub struct UnfedDays {
    pub count: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── startup / loading ────────────────────────────────────────────
            .add_systems(OnEnter(GameState::Playing), setup_feed_trough)

            // ── purchase detection ───────────────────────────────────────────
            .add_systems(
                Update,
                (
                    handle_animal_purchase,
                    handle_animal_wander,
                    handle_animal_interact,
                    handle_feed_trough_interact,
                    handle_product_collection,
                    update_floating_feedback,
                    sync_animal_state_resource,
                    update_product_indicators,
                )
                    .run_if(in_state(GameState::Playing)),
            )

            // ── day-end processing ───────────────────────────────────────────
            .add_systems(
                Update,
                handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
            );
    }
}
