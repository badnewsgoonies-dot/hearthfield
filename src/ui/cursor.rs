//! Custom pixel-art mouse cursor system.
//!
//! Hides the OS cursor and draws a sprite that follows the mouse each frame.
//! Three visual states map to game context:
//!   - Default  → normal gameplay / menus with no special hover
//!   - Pointing → hovering over an NPC or Interactable entity in world space
//!   - Holding  → any inventory / drag state (GameState::Inventory)

use crate::shared::*;
use bevy::prelude::*;
use bevy::render::camera::Camera;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Pixel radius in world units within which the cursor "hovers" an entity.
/// World units match pixel_scale=3 so 24px world = 8px logical pixel art.
const HOVER_RADIUS: f32 = 24.0;

/// Z-layer for the cursor sprite — must be above all world and UI layers.
/// Z_WEATHER = 400, so 500 is safely on top.
const Z_CURSOR: f32 = 500.0;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Stores the three cursor image handles loaded at startup.
#[derive(Resource)]
pub struct CursorAssets {
    pub default_handle: Handle<Image>,
    pub pointing_handle: Handle<Image>,
    pub holding_handle: Handle<Image>,
}

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker for the cursor sprite entity.
#[derive(Component)]
pub struct GameCursorSprite;

// ═══════════════════════════════════════════════════════════════════════
// STARTUP SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Load cursor images and hide the OS cursor.
pub fn setup_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: Query<&mut Window>,
) {
    // Load cursor images from disk.
    let default_handle = asset_server.load("ui/cursor_default.png");
    let pointing_handle = asset_server.load("ui/cursor_pointing.png");
    let holding_handle = asset_server.load("ui/cursor_holding.png");

    commands.insert_resource(CursorAssets {
        default_handle: default_handle.clone(),
        pointing_handle,
        holding_handle,
    });

    // Hide the OS cursor so only our sprite shows.
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.visible = false;
    }

    // Spawn cursor sprite — starts hidden until the first mouse move.
    commands.spawn((
        GameCursorSprite,
        Sprite {
            // Anchor at top-left so the hotspot matches (0, 0) of the image.
            anchor: bevy::sprite::Anchor::TopLeft,
            image: default_handle,
            // Render at native pixel size (16×16 in world/pixel coords).
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_CURSOR),
        // Start invisible; update_cursor_sprite will position and show it.
        Visibility::Hidden,
    ));
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEM
// ═══════════════════════════════════════════════════════════════════════

/// Each frame:
/// 1. Determine which cursor image to show based on game state / hover.
/// 2. Convert the OS cursor position to world space via the camera.
/// 3. Move the sprite to that position.
pub fn update_cursor_sprite(
    windows: Query<&Window>,
    state: Res<State<GameState>>,
    cursor_assets: Res<CursorAssets>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    npc_query: Query<&Transform, (With<Npc>, Without<GameCursorSprite>)>,
    interactable_query: Query<&Transform, (With<Interactable>, Without<GameCursorSprite>)>,
    mut cursor_query: Query<(&mut Transform, &mut Visibility, &mut Sprite), With<GameCursorSprite>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((mut cursor_tf, mut cursor_vis, mut cursor_sprite)) = cursor_query.get_single_mut()
    else {
        return;
    };

    // If there is no cursor position (window not focused / no mouse), hide.
    let Some(screen_pos) = window.cursor_position() else {
        *cursor_vis = Visibility::Hidden;
        return;
    };

    // ── World-space position ──────────────────────────────────────────
    let world_pos = if let Ok((camera, cam_gtf)) = camera_query.get_single() {
        match camera.viewport_to_world_2d(cam_gtf, screen_pos) {
            Ok(wp) => wp,
            Err(_) => {
                // Camera not ready yet — hide cursor and retry next frame.
                *cursor_vis = Visibility::Hidden;
                return;
            }
        }
    } else {
        *cursor_vis = Visibility::Hidden;
        return;
    };

    // ── Choose cursor image ──────────────────────────────────────────
    let current_state = *state.get();

    let desired_image = if current_state == GameState::Inventory {
        // Any inventory open → holding cursor.
        cursor_assets.holding_handle.clone()
    } else if current_state == GameState::Playing {
        // Check world hover only while playing.
        let hovered = is_hovering_entity(world_pos, &npc_query, &interactable_query);
        if hovered {
            cursor_assets.pointing_handle.clone()
        } else {
            cursor_assets.default_handle.clone()
        }
    } else {
        // Menus, dialogue, shop, etc. → default arrow.
        cursor_assets.default_handle.clone()
    };

    // Only swap the handle if it changed (avoids clone each frame).
    if cursor_sprite.image != desired_image {
        cursor_sprite.image = desired_image;
    }

    // ── Position sprite ───────────────────────────────────────────────
    // The cursor sprite uses TopLeft anchor, so translation == top-left corner.
    cursor_tf.translation.x = world_pos.x;
    cursor_tf.translation.y = world_pos.y;
    cursor_tf.translation.z = Z_CURSOR;

    *cursor_vis = Visibility::Inherited;
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Returns true if the world-space cursor position is within HOVER_RADIUS
/// of any NPC or Interactable entity.
fn is_hovering_entity(
    cursor_world: Vec2,
    npc_query: &Query<&Transform, (With<Npc>, Without<GameCursorSprite>)>,
    interactable_query: &Query<&Transform, (With<Interactable>, Without<GameCursorSprite>)>,
) -> bool {
    for tf in npc_query.iter() {
        let entity_pos = tf.translation.truncate();
        if cursor_world.distance(entity_pos) <= HOVER_RADIUS {
            return true;
        }
    }
    for tf in interactable_query.iter() {
        let entity_pos = tf.translation.truncate();
        if cursor_world.distance(entity_pos) <= HOVER_RADIUS {
            return true;
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════
// CLEANUP
// ═══════════════════════════════════════════════════════════════════════

/// Restore the OS cursor when the app exits / window closes.
/// Not strictly required but polite for desktop users.
#[allow(dead_code)]
pub fn restore_os_cursor(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.visible = true;
    }
}
