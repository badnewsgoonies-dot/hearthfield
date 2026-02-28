//! Minigame UI rendering and bobber animation.

use bevy::prelude::*;

use crate::shared::*;
use super::{
    Bobber, MinigameRoot, MinigameBgBar, MinigameFishZone,
    MinigameCatchBar, MinigameProgressFill, MinigameProgressBg,
    FishingMinigameState, FishingState,
};
use super::minigame::{
    zone_to_screen_y,
    MINIGAME_BAR_HEIGHT, MINIGAME_BAR_WIDTH,
    PROGRESS_BAR_Y, PROGRESS_BAR_WIDTH, PROGRESS_BAR_HEIGHT,
};

// ─── Colors ───────────────────────────────────────────────────────────────────

fn color_bg_bar() -> Color {
    Color::srgba(0.15, 0.15, 0.15, 0.85)
}

fn color_fish_zone() -> Color {
    Color::srgb(0.9, 0.35, 0.1)
}

fn color_catch_bar() -> Color {
    Color::srgb(0.2, 0.85, 0.3)
}

fn color_progress_bg() -> Color {
    Color::srgb(0.25, 0.25, 0.25)
}

fn color_progress_fill() -> Color {
    Color::srgb(0.1, 0.7, 0.95)
}

fn color_progress_fill_near() -> Color {
    Color::srgb(0.95, 0.85, 0.1)
}

// ─── Z-layers ─────────────────────────────────────────────────────────────────

const Z_UI_BG: f32 = 50.0;

// ─── OnEnter(GameState::Fishing) — spawn minigame UI ─────────────────────────

/// Spawn the fishing minigame UI when entering the Fishing game state.
///
/// The minigame bar is positioned in the right portion of the visible screen.
/// Since the camera uses a scale of 1/PIXEL_SCALE, we need to account for that
/// when converting screen pixel coordinates to world coordinates. With the camera
/// at 1/PIXEL_SCALE scale, 1 screen pixel = PIXEL_SCALE world units. However,
/// the camera projection already handles this — sprites placed at world coords
/// appear at their world position divided by the camera scale factor on screen.
/// In practice, we position UI relative to screen center (world origin).
pub fn spawn_minigame_ui(
    mut commands: Commands,
    minigame_state: Res<FishingMinigameState>,
) {
    // The camera scale is 1/PIXEL_SCALE. With a 960x540 screen:
    // The camera shows a region of 960*PIXEL_SCALE × 540*PIXEL_SCALE world units.
    // Screen right edge ≈ SCREEN_WIDTH/2 * PIXEL_SCALE world units from center.
    // We place the minigame bar near the right edge.
    let screen_to_world = PIXEL_SCALE;

    // Bar position: right side, vertically centered
    let bar_world_x = (SCREEN_WIDTH / 2.0 - 90.0) * screen_to_world;
    let bar_world_y = 0.0_f32;

    // Convert bar dimensions from screen-pixels to world units
    let bar_h_world = MINIGAME_BAR_HEIGHT * screen_to_world;
    let bar_w_world = MINIGAME_BAR_WIDTH * screen_to_world;

    let fish_zone_h_world = minigame_state.fish_zone_half * 2.0 * (bar_h_world / 100.0);
    let catch_bar_h_world = minigame_state.catch_bar_half * 2.0 * (bar_h_world / 100.0);

    let progress_bar_y_world = PROGRESS_BAR_Y * screen_to_world;
    let progress_w_world = PROGRESS_BAR_WIDTH * screen_to_world;
    let progress_h_world = PROGRESS_BAR_HEIGHT * screen_to_world;

    // Scale factor: zone 0-100 maps to bar height
    let y_scale = bar_h_world / MINIGAME_BAR_HEIGHT;

    let fish_y = zone_to_screen_y(minigame_state.fish_zone_center) * y_scale;
    let catch_y = zone_to_screen_y(minigame_state.catch_bar_center) * y_scale;

    // Spawn root entity (transparent container)
    commands
        .spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(bar_world_x, bar_world_y, Z_UI_BG)),
            MinigameRoot,
        ))
        .with_children(|parent| {
            // Background bar (dark, semi-transparent)
            parent.spawn((
                Sprite {
                    color: color_bg_bar(),
                    custom_size: Some(Vec2::new(bar_w_world, bar_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                MinigameBgBar,
            ));

            // Fish zone (red/orange — target to overlap)
            parent.spawn((
                Sprite {
                    color: color_fish_zone(),
                    custom_size: Some(Vec2::new(bar_w_world * 0.85, fish_zone_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, fish_y, 1.0)),
                MinigameFishZone,
            ));

            // Catch bar (green — controlled by player)
            parent.spawn((
                Sprite {
                    color: color_catch_bar(),
                    custom_size: Some(Vec2::new(bar_w_world, catch_bar_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, catch_y, 1.5)),
                MinigameCatchBar,
            ));

            // Progress bar background
            parent.spawn((
                Sprite {
                    color: color_progress_bg(),
                    custom_size: Some(Vec2::new(progress_w_world, progress_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, progress_bar_y_world, 1.0)),
                MinigameProgressBg,
            ));

            // Progress bar fill
            // Anchored to the left edge; x-scale = 0.001 to 1.0 representing 0-100%
            parent.spawn((
                Sprite {
                    color: color_progress_fill(),
                    custom_size: Some(Vec2::new(progress_w_world, progress_h_world)),
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    -progress_w_world / 2.0,
                    progress_bar_y_world,
                    2.0,
                ))
                .with_scale(Vec3::new(0.001, 1.0, 1.0)),
                MinigameProgressFill,
            ));
        });
}

/// Clean up all minigame UI entities when leaving GameState::Fishing.
pub fn despawn_minigame_ui(
    mut commands: Commands,
    root_query: Query<Entity, With<MinigameRoot>>,
) {
    for entity in root_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Bobber animation ─────────────────────────────────────────────────────────

/// Animate the bobber with gentle sinusoidal bobbing.
/// When a bite is pending, the bobber dips more aggressively to signal the player.
pub fn animate_bobber(
    mut bobber_query: Query<(&mut Transform, &mut Bobber)>,
    fishing_state: Res<FishingState>,
    time: Res<Time>,
) {
    use super::FishingPhase;

    for (mut transform, mut bobber) in bobber_query.iter_mut() {
        let is_bite = fishing_state.phase == FishingPhase::BitePending;

        // Faster, deeper bob when a fish has bitten
        let bob_speed = if is_bite { 4.0 } else { 1.5 };
        let bob_amplitude = if is_bite { 6.0 } else { 2.0 };

        bobber.bob_timer.tick(time.delta());

        let elapsed = time.elapsed_secs();
        let bob_y = (elapsed * bob_speed).sin() * bob_amplitude;

        transform.translation.y = bobber.original_y + bob_y;
    }
}

// ─── Progress fill color update ───────────────────────────────────────────────

/// Update the progress fill color to give visual feedback as it nears 100%.
/// Called every frame during the Fishing state.
pub fn update_progress_fill_color(
    minigame_state: Res<FishingMinigameState>,
    mut fill_query: Query<&mut Sprite, With<MinigameProgressFill>>,
) {
    for mut sprite in fill_query.iter_mut() {
        if minigame_state.progress > 75.0 {
            // Lerp from blue toward yellow as progress approaches 100%
            let t = (minigame_state.progress - 75.0) / 25.0;
            let base = color_progress_fill().to_srgba();
            let near = color_progress_fill_near().to_srgba();
            let r = base.red * (1.0 - t) + near.red * t;
            let g = base.green * (1.0 - t) + near.green * t;
            let b = base.blue * (1.0 - t) + near.blue * t;
            sprite.color = Color::srgb(r, g, b);
        } else {
            sprite.color = color_progress_fill();
        }
    }
}
