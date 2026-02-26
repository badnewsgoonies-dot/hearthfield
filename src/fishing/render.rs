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
    MINIGAME_BAR_X, PROGRESS_BAR_Y, PROGRESS_BAR_WIDTH, PROGRESS_BAR_HEIGHT,
};

// ─── Colors ───────────────────────────────────────────────────────────────────

const COLOR_BG_BAR: Color = Color::srgba(0.15, 0.15, 0.15, 0.85);
const COLOR_FISH_ZONE: Color = Color::srgb(0.9, 0.35, 0.1);
const COLOR_CATCH_BAR: Color = Color::srgb(0.2, 0.85, 0.3);
const COLOR_PROGRESS_BG: Color = Color::srgb(0.25, 0.25, 0.25);
const COLOR_PROGRESS_FILL: Color = Color::srgb(0.1, 0.7, 0.95);
const COLOR_PROGRESS_FILL_NEAR: Color = Color::srgb(0.95, 0.85, 0.1); // yellow when near 100%

// ─── Z-layers ─────────────────────────────────────────────────────────────────

const Z_UI_BG: f32 = 50.0;
const Z_UI_ELEMENTS: f32 = 51.0;
const Z_UI_PROGRESS: f32 = 52.0;

// ─── OnEnter(GameState::Fishing) — spawn minigame UI ─────────────────────────

pub fn spawn_minigame_ui(
    mut commands: Commands,
    minigame_state: Res<FishingMinigameState>,
    fishing_state: Res<FishingState>,
) {
    // Convert to "screen-space" by placing the UI in world coords at a fixed
    // position that the camera always sees. Since the camera is scaled by
    // 1/PIXEL_SCALE, world units map directly to screen pixels at 1:1 when
    // we account for the scale. We place the UI relative to the camera by
    // using a very high Z so it renders on top, and position based on known
    // screen dimensions.
    //
    // The minigame bar sits in the right portion of the screen.
    // Screen is SCREEN_WIDTH × SCREEN_HEIGHT = 960 × 540.
    // In world space (accounting for camera scale = 1/PIXEL_SCALE = 1/3):
    //   screen center = (0, 0) in view;
    //   world coords: multiply screen pixels by PIXEL_SCALE.
    //   But our camera is at scale 1/3, meaning 1 world unit = 1/3 screen pixel.
    //   So 1 screen pixel = 3 world units.
    //   To place at screen x=380 (right side), world_x = 380 * ...
    //   Actually with Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    //   the camera maps 1 screen pixel to PIXEL_SCALE world units.
    //   So world_x for screen_x=400: world_x = 400 (camera handles the rest).
    //   We just need to place sprites in unscaled screen coordinates since
    //   we'll parent them to a root node at the right position.
    //
    // Simpler: place the minigame in world coords at SCREEN_WIDTH/2 - margin.
    // We use PIXEL_SCALE to convert screen positions → world positions.

    let screen_to_world = PIXEL_SCALE; // 3.0: multiply screen coords by this

    // Bar center position in world space
    let bar_world_x = (SCREEN_WIDTH / 2.0 - 80.0) * screen_to_world;
    let bar_world_y = 0.0_f32;

    // Fish and catch zone heights in world units (the bar is MINIGAME_BAR_HEIGHT screen px tall)
    let bar_h_world = MINIGAME_BAR_HEIGHT * screen_to_world;
    let bar_w_world = MINIGAME_BAR_WIDTH * screen_to_world;

    let fish_zone_h_world = minigame_state.fish_zone_half * 2.0 * (bar_h_world / 100.0);
    let catch_bar_h_world = minigame_state.catch_bar_half * 2.0 * (bar_h_world / 100.0);

    let progress_bar_y_world = (PROGRESS_BAR_Y) * screen_to_world;
    let progress_w_world = PROGRESS_BAR_WIDTH * screen_to_world;
    let progress_h_world = PROGRESS_BAR_HEIGHT * screen_to_world;

    // Scale factor to convert zone_to_screen_y → world
    let y_scale = bar_h_world / MINIGAME_BAR_HEIGHT;

    let fish_y = zone_to_screen_y(minigame_state.fish_zone_center) * y_scale;
    let catch_y = zone_to_screen_y(minigame_state.catch_bar_center) * y_scale;

    // Spawn root entity
    commands
        .spawn((
            Sprite {
                color: Color::NONE, // transparent root
                ..default()
            },
            Transform::from_translation(Vec3::new(bar_world_x, bar_world_y, Z_UI_BG)),
            MinigameRoot,
        ))
        .with_children(|parent| {
            // Background bar
            parent.spawn((
                Sprite {
                    color: COLOR_BG_BAR,
                    custom_size: Some(Vec2::new(bar_w_world, bar_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                MinigameBgBar,
            ));

            // Fish zone (red/orange block)
            parent.spawn((
                Sprite {
                    color: COLOR_FISH_ZONE,
                    custom_size: Some(Vec2::new(bar_w_world * 0.85, fish_zone_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, fish_y, 1.0)),
                MinigameFishZone,
            ));

            // Catch bar (green block)
            parent.spawn((
                Sprite {
                    color: COLOR_CATCH_BAR,
                    custom_size: Some(Vec2::new(bar_w_world, catch_bar_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, catch_y, 1.5)),
                MinigameCatchBar,
            ));

            // Progress bar background
            parent.spawn((
                Sprite {
                    color: COLOR_PROGRESS_BG,
                    custom_size: Some(Vec2::new(progress_w_world, progress_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, progress_bar_y_world, 1.0)),
                MinigameProgressBg,
            ));

            // Progress bar fill (anchored left; scale x to represent fill fraction)
            // We offset by -half_width so it grows rightward from the left edge.
            parent.spawn((
                Sprite {
                    color: COLOR_PROGRESS_FILL,
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

/// Clean up minigame UI when leaving GameState::Fishing.
pub fn despawn_minigame_ui(
    mut commands: Commands,
    root_query: Query<Entity, With<MinigameRoot>>,
) {
    for entity in root_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Bobber animation ─────────────────────────────────────────────────────────

/// Animate the bobber with a gentle bobbing motion.
/// When a bite is pending, make it bob more aggressively (the "dip").
pub fn animate_bobber(
    mut bobber_query: Query<(&mut Transform, &mut Bobber)>,
    fishing_state: Res<FishingState>,
    time: Res<Time>,
) {
    use super::FishingPhase;

    let dt = time.delta_secs();

    for (mut transform, mut bobber) in bobber_query.iter_mut() {
        let is_bite = fishing_state.phase == FishingPhase::BitePending;

        // Faster, deeper bob when a fish has bitten
        let bob_speed = if is_bite { 4.0 } else { 1.5 };
        let bob_amplitude = if is_bite { 6.0 } else { 2.0 };

        bobber.bob_timer.tick(time.delta());

        // Use sinusoidal bobbing
        let elapsed = time.elapsed_secs();
        let bob_y = (elapsed * bob_speed).sin() * bob_amplitude;

        transform.translation.y = bobber.original_y + bob_y;
    }
}

// ─── Progress fill color update ───────────────────────────────────────────────

/// Update the progress fill color to give visual feedback as it nears 100%.
/// This runs as part of update_progress in minigame.rs via the render pass.
pub fn update_progress_fill_color(
    minigame_state: Res<FishingMinigameState>,
    mut fill_query: Query<&mut Sprite, With<MinigameProgressFill>>,
) {
    for mut sprite in fill_query.iter_mut() {
        if minigame_state.progress > 75.0 {
            // Lerp toward yellow as progress approaches 100%
            let t = (minigame_state.progress - 75.0) / 25.0;
            let r = COLOR_PROGRESS_FILL.to_srgba().red * (1.0 - t)
                + COLOR_PROGRESS_FILL_NEAR.to_srgba().red * t;
            let g = COLOR_PROGRESS_FILL.to_srgba().green * (1.0 - t)
                + COLOR_PROGRESS_FILL_NEAR.to_srgba().green * t;
            let b = COLOR_PROGRESS_FILL.to_srgba().blue * (1.0 - t)
                + COLOR_PROGRESS_FILL_NEAR.to_srgba().blue * t;
            sprite.color = Color::srgb(r, g, b);
        } else {
            sprite.color = COLOR_PROGRESS_FILL;
        }
    }
}
