use bevy::prelude::*;
use crate::shared::*;

/// Syncs LogicalPosition → Transform with pixel rounding and Y-sort Z.
/// Runs in PostUpdate AFTER all movement systems.
///
/// For Y-sorted entities: rounds XY, computes Z from Y position.
/// For non-Y-sorted entities with LogicalPosition: just rounds XY, keeps Z.
pub fn sync_position_and_ysort(
    mut with_ysort: Query<(&LogicalPosition, &mut Transform), With<YSorted>>,
    mut without_ysort: Query<(&LogicalPosition, &mut Transform), Without<YSorted>>,
) {
    for (logical_pos, mut transform) in &mut with_ysort {
        transform.translation.x = logical_pos.0.x.round();
        transform.translation.y = logical_pos.0.y.round();
        transform.translation.z = Z_ENTITY_BASE - logical_pos.0.y * Z_Y_SORT_SCALE;
    }

    for (logical_pos, mut transform) in &mut without_ysort {
        transform.translation.x = logical_pos.0.x.round();
        transform.translation.y = logical_pos.0.y.round();
    }
}
