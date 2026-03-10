use super::{DistanceAnimator, PlayerSpriteData};
use crate::shared::*;
use bevy::prelude::*;

/// Starting grid position inside the player's house (center of living room).
const SPAWN_GRID_X: i32 = 8;
const SPAWN_GRID_Y: i32 = 8;
/// Match the NPC character scale so the player doesn't tower over other
/// people or read like a tree-sized sprite.
const PLAYER_RENDER_SIZE: f32 = 24.0;

/// Spawn the player entity with all necessary components.
/// Runs once on `OnEnter(GameState::Playing)`.
pub fn spawn_player(
    mut commands: Commands,
    existing: Query<Entity, With<Player>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_data: ResMut<PlayerSpriteData>,
    mut player_state: ResMut<PlayerState>,
) {
    // Guard: don't double-spawn if returning to Playing state.
    if !existing.is_empty() {
        return;
    }

    player_state.current_map = MapId::PlayerHouse;

    let spawn = grid_to_world_center(SPAWN_GRID_X, SPAWN_GRID_Y);
    let world_x = spawn.x;
    let world_y = spawn.y;

    // Load the character spritesheet.
    // Sheet is 192×192, laid out as a 4×4 grid of 48×48 frames:
    //   Row 0 (indices  0- 3): Walk down
    //   Row 1 (indices  4- 7): Walk left
    //   Row 2 (indices  8-11): Walk right
    //   Row 3 (indices 12-15): Walk up
    let texture = asset_server.load("sprites/character_spritesheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(48, 48), 4, 4, None, None);
    let layout_handle = layouts.add(layout);

    // Cache handles so we don't reload on re-entry (e.g. after loading screen).
    sprite_data.image = texture.clone();
    sprite_data.layout = layout_handle.clone();

    // Load action sprite sheet.
    // character_actions.png is 2x12 grid of 48x48.
    let action_tex = asset_server.load("sprites/character_actions.png");
    let action_layout = TextureAtlasLayout::from_grid(UVec2::new(48, 48), 2, 12, None, None);
    let action_layout_handle = layouts.add(action_layout);
    sprite_data.action_image = action_tex;
    sprite_data.action_layout = action_layout_handle;

    sprite_data.loaded = true;

    commands.spawn((
        // Tag
        Player,
        // Movement state
        PlayerMovement::default(),
        // Grid position for tile-based lookups
        GridPosition::new(SPAWN_GRID_X, SPAWN_GRID_Y),
        // Animated sprite — frame 0 = idle-down (first frame of Row 0).
        // Downscale the 48px source frame to the same character scale used by NPCs.
        {
            let mut s = Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: layout_handle,
                    index: 0,
                },
            );
            s.anchor = bevy::sprite::Anchor::BottomCenter;
            s.custom_size = Some(Vec2::splat(PLAYER_RENDER_SIZE));
            s
        },
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_render_size_matches_character_scale() {
        assert_eq!(PLAYER_RENDER_SIZE, 24.0);
    }
}
