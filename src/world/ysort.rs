use bevy::prelude::*;
use crate::shared::*;

/// Syncs LogicalPosition → Transform with pixel rounding and Y-sort Z.
/// Runs in PostUpdate AFTER all movement systems.
///
/// For Y-sorted entities with LogicalPosition: rounds XY, computes Z from logical Y.
/// For non-Y-sorted entities with LogicalPosition: just rounds XY, keeps Z.
/// For Y-sorted entities WITHOUT LogicalPosition (static objects): computes Z from current Y.
pub fn sync_position_and_ysort(
    mut ysorted_with_pos: Query<
        (&LogicalPosition, &mut Transform),
        With<YSorted>,
    >,
    mut not_ysorted_with_pos: Query<
        (&LogicalPosition, &mut Transform),
        Without<YSorted>,
    >,
    mut ysorted_no_pos: Query<
        &mut Transform,
        (With<YSorted>, Without<LogicalPosition>),
    >,
) {
    // Moving Y-sorted entities: pixel-snap XY, compute Z from logical Y
    for (logical_pos, mut transform) in &mut ysorted_with_pos {
        transform.translation.x = logical_pos.0.x.round();
        transform.translation.y = logical_pos.0.y.round();
        transform.translation.z = Z_ENTITY_BASE - logical_pos.0.y * Z_Y_SORT_SCALE;
    }

    // Non-Y-sorted entities with LogicalPosition: pixel-snap XY, keep Z
    for (logical_pos, mut transform) in &mut not_ysorted_with_pos {
        transform.translation.x = logical_pos.0.x.round();
        transform.translation.y = logical_pos.0.y.round();
    }

    // Static Y-sorted entities (no LogicalPosition): compute Z from current Y
    // Do NOT touch XY — these were placed correctly at spawn
    for mut transform in &mut ysorted_no_pos {
        transform.translation.z = Z_ENTITY_BASE - transform.translation.y * Z_Y_SORT_SCALE;
    }
}
