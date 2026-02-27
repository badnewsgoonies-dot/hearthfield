//! Tool upgrade extensions — watering area computation and stamina cost helpers.
//!
//! These pure functions are called by the farming domain to determine which tiles
//! a watering can action covers and how much stamina it costs, based on the tool tier.
//!
//! The farming domain imports these via:
//!   `use crate::economy::tool_upgrades::{watering_can_area, tool_stamina_cost};`

use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// Watering area
// ─────────────────────────────────────────────────────────────────────────────

/// Returns all tile positions that a watering-can action covers, based on tier
/// and the direction the player is facing.
///
/// # Coverage rules
/// | Tier     | Pattern                                  |
/// |----------|------------------------------------------|
/// | Basic    | 1 tile — the target tile itself          |
/// | Copper   | 3 tiles in a line along `facing`         |
/// | Iron     | 5 tiles in a line along `facing`         |
/// | Gold     | 3 × 3 area (9 tiles) centred on target   |
/// | Iridium  | 6 × 6 area (36 tiles) centred on target  |
///
/// For line patterns the target tile is the *first* tile; the line extends
/// further in the direction the player is facing.
pub fn watering_can_area(
    tier: ToolTier,
    target_x: i32,
    target_y: i32,
    facing: Facing,
) -> Vec<(i32, i32)> {
    match tier {
        ToolTier::Basic => {
            // Single tile — current behaviour.
            vec![(target_x, target_y)]
        }

        ToolTier::Copper => {
            // 3 tiles in a line starting at target and extending along facing.
            line_tiles(target_x, target_y, facing, 3)
        }

        ToolTier::Iron => {
            // 5 tiles in a line.
            line_tiles(target_x, target_y, facing, 5)
        }

        ToolTier::Gold => {
            // 3 × 3 area centred on target.
            square_area(target_x, target_y, 1) // radius 1 → 3×3
        }

        ToolTier::Iridium => {
            // 6 × 6 area.  Bevy/grid convention: for even side lengths we can't
            // have a true centre, so we offset by −2..=3 in each axis (6 tiles).
            let half = 3_i32; // gives 6 tiles per axis: −2, −1, 0, 1, 2, 3
            let mut tiles = Vec::with_capacity(36);
            for dy in -(half - 1)..=half {
                for dx in -(half - 1)..=half {
                    tiles.push((target_x + dx, target_y + dy));
                }
            }
            tiles
        }
    }
}

/// Build a straight line of `length` tiles starting at `(sx, sy)` and walking
/// one step per tile in the direction of `facing`.
fn line_tiles(sx: i32, sy: i32, facing: Facing, length: u8) -> Vec<(i32, i32)> {
    let (dx, dy) = facing_delta(facing);
    (0..length as i32)
        .map(|i| (sx + dx * i, sy + dy * i))
        .collect()
}

/// Build a square area of side `2 * radius + 1` centred on `(cx, cy)`.
fn square_area(cx: i32, cy: i32, radius: i32) -> Vec<(i32, i32)> {
    let side = 2 * radius + 1;
    let mut tiles = Vec::with_capacity((side * side) as usize);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            tiles.push((cx + dx, cy + dy));
        }
    }
    tiles
}

/// Returns the (dx, dy) unit step for each facing direction.
/// Grid convention: +y is up (north), +x is right (east).
fn facing_delta(facing: Facing) -> (i32, i32) {
    match facing {
        Facing::Up    => (0,  1),
        Facing::Down  => (0, -1),
        Facing::Left  => (-1, 0),
        Facing::Right => (1,  0),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stamina cost
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the stamina cost for a single tool-use action at the given tier.
///
/// Better tiers are more efficient: the cost is scaled by `tier.stamina_multiplier()`.
/// The cost is charged once per action (not per tile watered), so upgrading the
/// watering can both widens the area AND reduces stamina cost.
///
/// # Example
/// ```ignore
/// // Basic watering can: 2.0 stamina.
/// assert_eq!(tool_stamina_cost(2.0, ToolTier::Basic), 2.0);
/// // Iridium watering can: 2.0 × 0.4 = 0.8 stamina.
/// assert_eq!(tool_stamina_cost(2.0, ToolTier::Iridium), 0.8);
/// ```
pub fn tool_stamina_cost(base_cost: f32, tier: ToolTier) -> f32 {
    base_cost * tier.stamina_multiplier()
}
