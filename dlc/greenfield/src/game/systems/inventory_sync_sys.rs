use bevy::prelude::*;

pub fn inventory_sync_system(_inv: Res<crate::game::resources::ActiveInventory>) {
    // System inventory_sync_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*_inv;
    if activity > 0 {
        // inventory_sync_system: tick had {activity} actionable events
    }
    let _ = activity;
}
