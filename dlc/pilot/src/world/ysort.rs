//! Y-sort rendering system.
//!
//! Entities at higher Y (further from camera) render behind entities at lower Y.
//! Within each Z-layer, the Transform.z is adjusted by a small offset derived
//! from the entity's Y position to produce correct overlap ordering.

use crate::shared::*;
use bevy::prelude::*;

// ─── Z-layer constants ──────────────────────────────────────────────────

/// Floor tiles and terrain.
pub const Z_FLOOR: f32 = Z_GROUND;
/// Ground decorations (flowers, markings).
pub const Z_FLOOR_DECOR: f32 = Z_GROUND_DECOR;
/// Static objects (furniture, signs).
pub const Z_OBJECTS_LAYER: f32 = Z_OBJECTS;
/// NPCs / crew members.
pub const Z_NPC: f32 = 40.0;
/// Player sprite.
pub const Z_PLAYER_LAYER: f32 = Z_PLAYER;
/// UI overlay elements rendered in world-space.
pub const Z_UI: f32 = Z_UI_OVERLAY;

// ─── Marker component ───────────────────────────────────────────────────

/// Tag an entity for automatic Y-sort within its Z-layer.
#[derive(Component)]
pub struct YSorted {
    /// The base Z-layer this entity belongs to.
    pub layer: f32,
}

impl Default for YSorted {
    fn default() -> Self {
        Self {
            layer: Z_OBJECTS_LAYER,
        }
    }
}

// ─── System ──────────────────────────────────────────────────────────────

/// Adjusts `Transform.translation.z` based on Y position within its layer.
///
/// Entities with a *higher* screen-Y (lower world-Y, i.e. further away) get a
/// *smaller* z offset so they render behind closer entities.
///
/// The offset is clamped to a small range (0.0 – 9.0) so it stays within the
/// layer and never bleeds into the next layer.
pub fn ysort_update(mut query: Query<(&YSorted, &mut Transform)>) {
    for (ysorted, mut transform) in &mut query {
        // World-Y is negative-down in our coordinate system.
        // Normalise to 0..1 range assuming a reasonable world extent.
        let y = transform.translation.y;
        // Map Y from roughly -2000..2000 to 0..1
        let normalized = ((y + 2000.0) / 4000.0).clamp(0.0, 1.0);
        // Closer entities (low Y / high normalized) get higher z within layer
        let offset = (1.0 - normalized) * 9.0;
        transform.translation.z = ysorted.layer + offset;
    }
}
