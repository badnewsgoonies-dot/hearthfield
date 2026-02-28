use bevy::prelude::*;
use crate::shared::*;
use super::{PlayerSpriteData, ActionSpriteData, DistanceAnimator};

/// Starting grid position on the farm (roughly center of the farmable area).
const SPAWN_GRID_X: i32 = 10;
const SPAWN_GRID_Y: i32 = 10;

/// Spawn the player entity with all necessary components.
/// Runs once on `OnEnter(GameState::Playing)`.
pub fn spawn_player(
    mut commands: Commands,
    existing: Query<Entity, With<Player>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_data: ResMut<PlayerSpriteData>,
    mut action_data: ResMut<ActionSpriteData>,
) {
    // Guard: don't double-spawn if returning to Playing state.
    if !existing.is_empty() {
        return;
    }

    let spawn = grid_to_world_center(SPAWN_GRID_X, SPAWN_GRID_Y);
    let world_x = spawn.x;
    let world_y = spawn.y;

    // Load the character spritesheet.
    // Sheet is 192×192, laid out as a 4×4 grid of 48×48 frames:
    //   Row 0 (indices  0- 3): Walk down
    //   Row 1 (indices  4- 7): Walk up
    //   Row 2 (indices  8-11): Walk right
    //   Row 3 (indices 12-15): Walk left
    let texture = asset_server.load("sprites/character_spritesheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(48, 48), 4, 4, None, None);
    let layout_handle = layouts.add(layout);

    // Cache handles so we don't reload on re-entry (e.g. after loading screen).
    sprite_data.image = texture.clone();
    sprite_data.layout = layout_handle.clone();
    sprite_data.loaded = true;

    commands.spawn((
        // Tag
        Player,
        // Movement state
        PlayerMovement::default(),
        // Grid position for tile-based lookups
        GridPosition::new(SPAWN_GRID_X, SPAWN_GRID_Y),
        // Animated sprite — frame 0 = idle-down (first frame of Row 0).
        // The 48×48 frame at PIXEL_SCALE 3.0 renders as 144px on screen
        // because the camera uses Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE)).
        // No custom_size needed; let the atlas frame's natural dimensions drive size.
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: layout_handle,
                index: 0,
            },
        ),
        // Logical position for pixel-perfect rendering (movement writes here)
        LogicalPosition(Vec2::new(world_x, world_y)),
        // World-space transform — Y-sort system sets Z from LogicalPosition each frame.
        Transform::from_translation(Vec3::new(world_x, world_y, Z_ENTITY_BASE)),
        // Y-sort depth ordering
        YSorted,
        // Required for rendering
        Visibility::default(),
        // Distance-based walk animation
        DistanceAnimator {
            last_pos: Vec2::new(world_x, world_y),
            ..default()
        },
    ));

    // Load character_actions.png atlas (tool-use animations)
    if !action_data.loaded {
        let action_texture = asset_server.load("sprites/character_actions.png");
        let action_layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(48, 48), 2, 12, None, None,
        ));
        action_data.image = action_texture;
        action_data.layout = action_layout;
        action_data.loaded = true;
    }
}
