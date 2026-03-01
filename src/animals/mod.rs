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
    pub cat_image: Handle<Image>,
    pub cat_layout: Handle<TextureAtlasLayout>,
    pub dog_image: Handle<Image>,
    pub dog_layout: Handle<TextureAtlasLayout>,
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

    // Sheep, cat, dog: reuse character_spritesheet.png with tint colors.
    // character_spritesheet.png: 192x256, 12 cols x 16 rows of 16x16 frames.
    let sheet = asset_server.load("sprites/character_spritesheet.png");
    let sheet_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        12,
        16,
        None,
        None,
    ));
    sprite_data.sheep_image = sheet.clone();
    sprite_data.sheep_layout = sheet_layout.clone();
    sprite_data.cat_image = sheet.clone();
    sprite_data.cat_layout = sheet_layout.clone();
    sprite_data.dog_image = sheet.clone();
    sprite_data.dog_layout = sheet_layout;

    sprite_data.loaded = true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct AnimalPlugin;

impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AnimalSpriteData>()
            // ── startup / loading ────────────────────────────────────────────
            .add_systems(OnEnter(GameState::Playing), (load_animal_sprites, setup_feed_trough))

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
