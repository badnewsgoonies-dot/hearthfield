use crate::shared::*;
use bevy::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Sub-modules
// ─────────────────────────────────────────────────────────────────────────────
mod day_end;
mod feeding;
mod interaction;
mod movement;
mod products;
mod rendering;
mod spawning;

pub use day_end::*;
pub use feeding::*;
pub use interaction::*;
pub use movement::*;
pub use products::*;
pub use rendering::*;
pub use spawning::*;

// ─────────────────────────────────────────────────────────────────────────────
// Private ECS components (internal to the animals domain)
// ─────────────────────────────────────────────────────────────────────────────

/// Marks an entity as the feed-trough object on the farm.
#[derive(Component, Debug, Clone)]
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

/// Timer component driving animal walk-cycle frame advancement.
/// Attached to all animal kinds. Atlas animals (chicken, cow) cycle sprite frames;
/// non-atlas animals use the timer elapsed time to drive a vertical bob animation.
#[derive(Component, Debug, Clone)]
pub struct AnimalAnimTimer {
    pub timer: Timer,
    /// Number of frames per animation row (atlas animals) or bob phases (non-atlas).
    pub frame_count: usize,
    /// Current frame index within the row.
    pub current_frame: usize,
}

/// Cached sprite atlas handles for animals with real sprite assets (chicken, cow).
#[derive(Resource, Default)]
pub struct AnimalSpriteData {
    pub loaded: bool,
    pub chicken_image: Handle<Image>,
    pub chicken_layout: Handle<TextureAtlasLayout>,
    pub cow_image: Handle<Image>,
    pub cow_layout: Handle<TextureAtlasLayout>,
}

/// Loads chicken and cow sprite atlases on first entry into Playing state.
pub fn load_animal_sprites(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_data: ResMut<AnimalSpriteData>,
) {
    if sprite_data.loaded {
        return;
    }

    // chicken.png: 64x32, 4 cols x 2 rows of 16x16 frames
    sprite_data.chicken_image = asset_server.load("sprites/chicken.png");
    sprite_data.chicken_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        2,
        None,
        None,
    ));

    // cow.png: 96x64, 3 cols x 2 rows of 32x32 frames
    sprite_data.cow_image = asset_server.load("sprites/cow.png");
    sprite_data.cow_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        3,
        2,
        None,
        None,
    ));

    sprite_data.loaded = true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Animal sprite animation
// ─────────────────────────────────────────────────────────────────────────────

/// System: animate animal sprites by cycling atlas frames when the animal is
/// moving (has a wander target). When idle, snaps back to frame 0 and resets
/// the timer so the bob offset returns to zero for non-atlas animals.
/// Non-atlas animals (no TextureAtlas) have their vertical bob applied by
/// `bob_non_atlas_animals` in PostUpdate after position sync.
pub fn animate_animal_sprites(
    time: Res<Time>,
    mut query: Query<(&WanderAi, &mut Sprite, &mut AnimalAnimTimer)>,
) {
    for (wander, mut sprite, mut anim) in query.iter_mut() {
        let is_moving = wander.target.is_some();

        if is_moving {
            anim.timer.tick(time.delta());
            if anim.timer.just_finished() {
                anim.current_frame = (anim.current_frame + 1) % anim.frame_count;
            }
        } else {
            anim.current_frame = 0;
            anim.timer.reset();
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = anim.current_frame;
        }
    }
}

/// System: apply a vertical bob (±1 px) to non-atlas animals while they move.
/// Runs in PostUpdate after `sync_position_and_ysort` has written the base
/// translation from LogicalPosition, so the offset is applied on top cleanly.
pub fn bob_non_atlas_animals(
    mut query: Query<(&WanderAi, &Sprite, &AnimalAnimTimer, &mut Transform)>,
) {
    for (wander, sprite, anim, mut transform) in query.iter_mut() {
        if sprite.texture_atlas.is_some() {
            continue;
        }
        if wander.target.is_some() {
            let elapsed = anim.timer.elapsed_secs();
            let period = anim.timer.duration().as_secs_f32();
            if period > 0.0 {
                let bob = (elapsed / period * std::f32::consts::TAU).sin();
                transform.translation.y += bob;
            }
        }
        // When idle the timer was reset (elapsed = 0, sin = 0) so no bob is added.
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimalSpriteData>()
            // ── startup / loading ────────────────────────────────────────────
            .add_systems(
                OnEnter(GameState::Playing),
                (load_animal_sprites, setup_feed_trough),
            )
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
                    animate_animal_sprites,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // ── bob non-atlas animals after position sync ────────────────────
            .add_systems(
                PostUpdate,
                bob_non_atlas_animals
                    .after(crate::world::ysort::sync_position_and_ysort)
                    .run_if(in_state(GameState::Playing)),
            )
            // ── day-end processing ───────────────────────────────────────────
            .add_systems(
                Update,
                handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
            );
    }
}
