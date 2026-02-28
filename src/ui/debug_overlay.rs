use bevy::prelude::*;
use crate::shared::*;
use crate::player::DistanceAnimator;

/// Marker for the debug overlay root node.
#[derive(Component)]
pub struct DebugOverlayRoot;

/// Marker for the debug text.
#[derive(Component)]
pub struct DebugOverlayText;

/// Toggle debug overlay with F3.
pub fn toggle_debug_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugOverlayState>,
) {
    if keys.just_pressed(KeyCode::F3) {
        debug_state.visible = !debug_state.visible;
    }
}

/// Spawn the debug overlay UI (runs once at startup).
pub fn spawn_debug_overlay(mut commands: Commands) {
    commands.spawn((
        DebugOverlayRoot,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(8.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Visibility::Hidden,
    )).with_children(|parent| {
        parent.spawn((
            DebugOverlayText,
            Text::new("Debug"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
        ));
    });
}

/// Update debug overlay content and visibility.
pub fn update_debug_overlay(
    debug_state: Res<DebugOverlayState>,
    mut overlay_query: Query<&mut Visibility, With<DebugOverlayRoot>>,
    mut text_query: Query<&mut Text, With<DebugOverlayText>>,
    player_query: Query<(
        &LogicalPosition,
        &GridPosition,
        &PlayerMovement,
        &DistanceAnimator,
        &Sprite,
    ), With<Player>>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    let Ok(mut vis) = overlay_query.get_single_mut() else { return };

    if !debug_state.visible {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Inherited;

    let Ok(mut text) = text_query.get_single_mut() else { return };

    let mut lines = Vec::new();

    if let Ok((pos, grid, movement, anim, sprite)) = player_query.get_single() {
        let atlas_idx = sprite.texture_atlas.as_ref().map(|a| a.index).unwrap_or(0);
        let ysort_z = Z_ENTITY_BASE - pos.0.y * Z_Y_SORT_SCALE;

        lines.push(format!("Pos: ({:.1}, {:.1})", pos.0.x, pos.0.y));
        lines.push(format!("Grid: ({}, {})", grid.x, grid.y));
        lines.push(format!("Facing: {:?}", movement.facing));
        lines.push(format!("Anim: {:?}", movement.anim_state));
        lines.push(format!("Atlas: {}", atlas_idx));
        lines.push(format!("Dist frame: {}/{}", anim.current_frame, anim.frames_per_row));
        lines.push(format!("Dist budget: {:.1}", anim.distance_budget));
        lines.push(format!("Y-sort Z: {:.2}", ysort_z));
    }

    if let Ok(cam_tf) = camera_query.get_single() {
        lines.push(format!("Cam: ({:.1}, {:.1})", cam_tf.translation.x, cam_tf.translation.y));
    }

    **text = lines.join("\n");
}
