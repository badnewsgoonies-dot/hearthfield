use crate::shared::*;
use bevy::prelude::*;

/// Syncs LogicalPosition → Transform with pixel rounding and Y-sort Z.
/// Runs in PostUpdate AFTER all movement systems.
///
/// For Y-sorted entities with LogicalPosition: rounds XY, computes Z from logical Y.
/// For non-Y-sorted entities with LogicalPosition: just rounds XY, keeps Z.
/// For Y-sorted entities WITHOUT LogicalPosition (static objects): computes Z from current Y.
pub fn sync_position_and_ysort(
    mut query: Query<
        (&mut Transform, Option<&LogicalPosition>, Has<YSorted>),
        Or<(With<LogicalPosition>, With<YSorted>)>,
    >,
) {
    for (mut transform, logical_pos, is_ysorted) in &mut query {
        if let Some(logical_pos) = logical_pos {
            // Any entity with LogicalPosition: pixel-snap XY.
            transform.translation.x = logical_pos.0.x.round();
            transform.translation.y = logical_pos.0.y.round();
            if is_ysorted {
                transform.translation.z = Z_ENTITY_BASE - logical_pos.0.y * Z_Y_SORT_SCALE;
            }
        } else if is_ysorted {
            // Static Y-sorted entities (no LogicalPosition): compute Z from current Y.
            // Do NOT touch XY — these were placed correctly at spawn.
            transform.translation.z = Z_ENTITY_BASE - transform.translation.y * Z_Y_SORT_SCALE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consolidated_pass_preserves_rounding_and_z_rules() {
        let mut app = App::new();
        app.add_systems(Update, sync_position_and_ysort);

        let ysorted_with_pos = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                LogicalPosition(Vec2::new(10.49, 20.51)),
                YSorted,
            ))
            .id();

        let not_ysorted_with_pos = app
            .world_mut()
            .spawn((
                Transform::from_xyz(1.0, 2.0, 77.0),
                LogicalPosition(Vec2::new(3.6, 4.4)),
            ))
            .id();

        let ysorted_no_pos = app
            .world_mut()
            .spawn((Transform::from_xyz(5.0, 6.75, 0.0), YSorted))
            .id();

        app.update();

        let tf = app.world().entity(ysorted_with_pos).get::<Transform>().unwrap();
        assert_eq!(tf.translation.x, 10.0);
        assert_eq!(tf.translation.y, 21.0);
        assert_eq!(
            tf.translation.z,
            Z_ENTITY_BASE - 20.51 * Z_Y_SORT_SCALE
        );

        let tf = app
            .world()
            .entity(not_ysorted_with_pos)
            .get::<Transform>()
            .unwrap();
        assert_eq!(tf.translation.x, 4.0);
        assert_eq!(tf.translation.y, 4.0);
        assert_eq!(tf.translation.z, 77.0);

        let tf = app.world().entity(ysorted_no_pos).get::<Transform>().unwrap();
        assert_eq!(tf.translation.x, 5.0);
        assert_eq!(tf.translation.y, 6.75);
        assert_eq!(tf.translation.z, Z_ENTITY_BASE - 6.75 * Z_Y_SORT_SCALE);
    }
}
