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

/// Cached sprite atlas handles for animals with real sprite assets.
#[derive(Resource, Default)]
pub struct AnimalSpriteData {
    pub loaded: bool,
    pub chicken_image: Handle<Image>,
    pub chicken_layout: Handle<TextureAtlasLayout>,
    pub cow_image: Handle<Image>,
    pub cow_layout: Handle<TextureAtlasLayout>,
    pub sheep_image: Handle<Image>,
    pub sheep_layout: Handle<TextureAtlasLayout>,
    pub goat_image: Handle<Image>,
    pub goat_layout: Handle<TextureAtlasLayout>,
    pub pig_image: Handle<Image>,
    pub pig_layout: Handle<TextureAtlasLayout>,
    pub duck_image: Handle<Image>,
    pub duck_layout: Handle<TextureAtlasLayout>,
    pub rabbit_image: Handle<Image>,
    pub rabbit_layout: Handle<TextureAtlasLayout>,
    pub dog_image: Handle<Image>,
    pub dog_layout: Handle<TextureAtlasLayout>,
    // Product indicator sprites
    pub egg_nest_image: Handle<Image>,
    pub egg_nest_layout: Handle<TextureAtlasLayout>,
    pub milk_grass_image: Handle<Image>,
    pub milk_grass_layout: Handle<TextureAtlasLayout>,
}

/// Loads all animal sprite atlases on first entry into Playing state.
pub fn load_animal_sprites(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_data: ResMut<AnimalSpriteData>,
) {
    if sprite_data.loaded {
        return;
    }

    // chicken.png: 384×64, 24 cols × 4 rows of 16×16 frames
    sprite_data.chicken_image = asset_server.load("sprites/chicken.png");
    sprite_data.chicken_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        24,
        4,
        None,
        None,
    ));

    // cow.png: 1152×192, 24 cols × 4 rows of 48×48 frames
    sprite_data.cow_image = asset_server.load("sprites/cow.png");
    sprite_data.cow_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 48),
        24,
        4,
        None,
        None,
    ));

    // sheep.png: 768×128, 24 cols × 4 rows of 32×32 frames
    sprite_data.sheep_image = asset_server.load("sprites/sheep.png");
    sprite_data.sheep_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        24,
        4,
        None,
        None,
    ));

    // goat.png: 768×192, 24 cols × 4 rows of 32×48 frames
    sprite_data.goat_image = asset_server.load("sprites/goat.png");
    sprite_data.goat_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 48),
        24,
        4,
        None,
        None,
    ));

    // pig.png: 768×128, 24 cols × 4 rows of 32×32 frames
    sprite_data.pig_image = asset_server.load("sprites/pig.png");
    sprite_data.pig_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        24,
        4,
        None,
        None,
    ));

    // duck.png: 829×128, 24 cols × 4 rows of 32×32 frames (rightmost 61px unused)
    sprite_data.duck_image = asset_server.load("sprites/duck.png");
    sprite_data.duck_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        24,
        4,
        None,
        None,
    ));

    // rabbit.png: 768×128, 24 cols × 4 rows of 32×32 frames
    sprite_data.rabbit_image = asset_server.load("sprites/rabbit.png");
    sprite_data.rabbit_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        24,
        4,
        None,
        None,
    ));

    // dog.png: 1152×416, 24 cols × 13 rows of 48×32 frames
    sprite_data.dog_image = asset_server.load("sprites/dog.png");
    sprite_data.dog_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 32),
        24,
        13,
        None,
        None,
    ));

    // egg_and_nest.png: 64×16, 4 cols × 1 row of 16×16 frames
    sprite_data.egg_nest_image = asset_server.load("sprites/egg_and_nest.png");
    sprite_data.egg_nest_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        1,
        None,
        None,
    ));

    // milk_and_grass.png: 64×16, 4 cols × 1 row of 16×16 frames
    sprite_data.milk_grass_image = asset_server.load("sprites/milk_and_grass.png");
    sprite_data.milk_grass_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        1,
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
    mut query: Query<(&WanderAi, Option<&Facing>, &mut Sprite, &mut AnimalAnimTimer)>,
) {
    for (wander, facing_opt, mut sprite, mut anim) in query.iter_mut() {
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
            // Calculate base index based on facing.
            // Layout: Row 0=Down, 1=Left, 2=Right, 3=Up
            // Each row has 6 frames, but the grid is 24 cols.
            // Wait, the grid is 24 cols, 4 rows.
            // If it's 24 cols * 4 rows, each row has 24 columns!
            // Actually, the original comment said "Each sprite sheet has 24 cols = 6 frames * 4 directions."
            // This means there is 1 row, or the columns themselves contain the directions.
            // Let's assume the directions are distributed along the 24 columns:
            // Down: 0..5, Left: 6..11, Right: 12..17, Up: 18..23
            let dir_offset = if let Some(facing) = facing_opt {
                match facing {
                    Facing::Down => 0,
                    Facing::Left => 6,
                    Facing::Right => 12,
                    Facing::Up => 18,
                }
            } else {
                0
            };

            atlas.index = dir_offset + anim.current_frame;
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
