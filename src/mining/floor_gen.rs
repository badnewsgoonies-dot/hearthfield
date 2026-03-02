//! Procedural mine floor generation.
//!
//! Each floor is a 24x24 grid. Rocks, enemies, a ladder, and mine tiles are
//! placed based on a deterministic seed derived from the floor number so that
//! re-entering the same floor (without progressing) yields the same layout.

use crate::shared::*;
use rand::prelude::*;
use rand::rngs::StdRng;

/// Width and height of every mine floor in tiles.
pub const MINE_WIDTH: i32 = 24;
pub const MINE_HEIGHT: i32 = 24;

/// Describes a single generated floor before it is spawned into the ECS.
#[derive(Debug, Clone)]
pub struct FloorBlueprint {
    pub floor: u8,
    pub rocks: Vec<RockBlueprint>,
    pub enemies: Vec<EnemyBlueprint>,
    pub ladder_pos: (i32, i32),
    /// If true, the ladder is hidden inside a rock and only revealed when
    /// that rock is destroyed (or all rocks are destroyed).
    pub ladder_hidden: bool,
    /// Index into `rocks` that contains the hidden ladder (if any).
    pub ladder_rock_index: Option<usize>,
    /// Player spawn position (near the entrance).
    pub spawn_pos: (i32, i32),
}

#[derive(Debug, Clone)]
pub struct RockBlueprint {
    pub x: i32,
    pub y: i32,
    pub health: u8,
    pub drop_item: String,
    pub drop_quantity: u8,
    /// If true, the hidden ladder is under this rock.
    pub has_ladder: bool,
}

#[derive(Debug, Clone)]
pub struct EnemyBlueprint {
    pub x: i32,
    pub y: i32,
    pub kind: MineEnemy,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,
}

/// Generate a complete floor blueprint for the given floor number.
pub fn generate_floor(floor: u8) -> FloorBlueprint {
    let mut rng = StdRng::seed_from_u64(floor as u64 * 7919 + 42);

    // --- Player spawn (bottom-center) ---
    let spawn_pos = (MINE_WIDTH / 2, 1);

    // --- Determine rock coverage (40-60%) ---
    let total_tiles = (MINE_WIDTH * MINE_HEIGHT) as usize;
    let coverage: f64 = rng.gen_range(0.40..=0.60);
    let max_rocks = (total_tiles as f64 * coverage) as usize;

    // Reserve tiles: spawn area (3x3 around spawn) and border row at bottom
    let mut occupied = std::collections::HashSet::new();
    // Keep spawn area clear (3x3)
    for dx in -1..=1 {
        for dy in -1..=1 {
            occupied.insert((spawn_pos.0 + dx, spawn_pos.1 + dy));
        }
    }
    // Also keep a small area around the very bottom row clear for entrance feel
    for x in 0..MINE_WIDTH {
        occupied.insert((x, 0));
    }

    // --- Place rocks ---
    let mut rocks = Vec::new();
    let mut attempts = 0;
    while rocks.len() < max_rocks && attempts < max_rocks * 4 {
        let x = rng.gen_range(1..MINE_WIDTH - 1);
        let y = rng.gen_range(2..MINE_HEIGHT - 1);
        if !occupied.contains(&(x, y)) {
            occupied.insert((x, y));
            let (drop_item, drop_qty, health) = rock_drop(floor, &mut rng);
            rocks.push(RockBlueprint {
                x,
                y,
                health,
                drop_item,
                drop_quantity: drop_qty,
                has_ladder: false,
            });
        }
        attempts += 1;
    }

    // --- Place ladder ---
    // Pick a random position in the upper half that isn't occupied, OR hide it in a rock.
    let hide_ladder = rng.gen_bool(0.6); // 60% chance the ladder is hidden in a rock
    let (ladder_pos, ladder_hidden, ladder_rock_index) = if hide_ladder && !rocks.is_empty() {
        // Put ladder inside a random rock in the upper half of the map
        let upper_rocks: Vec<usize> = rocks
            .iter()
            .enumerate()
            .filter(|(_, r)| r.y >= MINE_HEIGHT / 2)
            .map(|(i, _)| i)
            .collect();
        if let Some(&idx) = upper_rocks.choose(&mut rng) {
            let pos = (rocks[idx].x, rocks[idx].y);
            rocks[idx].has_ladder = true;
            (pos, true, Some(idx))
        } else if !rocks.is_empty() {
            // fallback: pick any rock
            let idx = rng.gen_range(0..rocks.len());
            let pos = (rocks[idx].x, rocks[idx].y);
            rocks[idx].has_ladder = true;
            (pos, true, Some(idx))
        } else {
            // no rocks at all (shouldn't happen), place ladder openly
            let lx = rng.gen_range(2..MINE_WIDTH - 2);
            let ly = rng.gen_range(MINE_HEIGHT / 2..MINE_HEIGHT - 2);
            ((lx, ly), false, None)
        }
    } else {
        // Place ladder openly in the upper portion
        let mut lx;
        let mut ly;
        let mut ladder_attempts = 0;
        loop {
            lx = rng.gen_range(2..MINE_WIDTH - 2);
            ly = rng.gen_range(MINE_HEIGHT / 2..MINE_HEIGHT - 2);
            ladder_attempts += 1;
            if !occupied.contains(&(lx, ly)) || ladder_attempts >= 100 {
                break;
            }
        }
        ((lx, ly), false, None)
    };

    // --- Place enemies ---
    let enemy_count = enemy_count_for_floor(floor, &mut rng);
    let mut enemies = Vec::new();
    let mut enemy_attempts = 0;
    while enemies.len() < enemy_count && enemy_attempts < enemy_count * 10 {
        let x = rng.gen_range(2..MINE_WIDTH - 2);
        let y = rng.gen_range(3..MINE_HEIGHT - 2);
        if !occupied.contains(&(x, y)) {
            occupied.insert((x, y));
            let kind = pick_enemy_kind(floor, &mut rng);
            let bp = make_enemy_blueprint(kind, floor, x, y);
            enemies.push(bp);
        }
        enemy_attempts += 1;
    }

    FloorBlueprint {
        floor,
        rocks,
        enemies,
        ladder_pos,
        ladder_hidden,
        ladder_rock_index,
        spawn_pos,
    }
}

/// Choose a rock drop based on floor depth.
fn rock_drop(floor: u8, rng: &mut StdRng) -> (String, u8, u8) {
    let roll: f64 = rng.gen();

    if floor <= 5 {
        // Floors 1-5: Stone, Copper Ore (20%)
        if roll < 0.20 {
            ("copper_ore".to_string(), rng.gen_range(1..=2), rng.gen_range(2..=3))
        } else {
            ("stone".to_string(), rng.gen_range(1..=3), 2)
        }
    } else if floor <= 10 {
        // Floors 6-10: Stone, Copper (30%), Iron (15%)
        if roll < 0.15 {
            ("iron_ore".to_string(), rng.gen_range(1..=2), rng.gen_range(3..=4))
        } else if roll < 0.45 {
            ("copper_ore".to_string(), rng.gen_range(1..=2), rng.gen_range(2..=3))
        } else {
            ("stone".to_string(), rng.gen_range(1..=3), 2)
        }
    } else if floor <= 15 {
        // Floors 11-15: Stone, Iron (30%), Gold (10%), Quartz (5%)
        if roll < 0.05 {
            ("quartz".to_string(), 1, 3)
        } else if roll < 0.15 {
            ("gold_ore".to_string(), rng.gen_range(1..=2), 4)
        } else if roll < 0.45 {
            ("iron_ore".to_string(), rng.gen_range(1..=2), 3)
        } else {
            ("stone".to_string(), rng.gen_range(1..=3), 2)
        }
    } else {
        // Floors 16-20: Stone, Gold (25%), Diamond (3%), Ruby (2%), Emerald (2%)
        if roll < 0.02 {
            ("emerald".to_string(), 1, 4)
        } else if roll < 0.04 {
            ("ruby".to_string(), 1, 4)
        } else if roll < 0.07 {
            ("diamond".to_string(), 1, 4)
        } else if roll < 0.32 {
            ("gold_ore".to_string(), rng.gen_range(1..=2), 4)
        } else {
            ("stone".to_string(), rng.gen_range(1..=3), 2)
        }
    }
}

/// How many enemies spawn on this floor.
fn enemy_count_for_floor(floor: u8, rng: &mut StdRng) -> usize {
    let base = 2;
    let extra = (floor as usize) / 4; // +1 every 4 floors
    let count = base + extra + rng.gen_range(0..=1);
    count.min(6) // cap at 6
}

/// Pick an enemy type appropriate for the floor depth.
fn pick_enemy_kind(floor: u8, rng: &mut StdRng) -> MineEnemy {
    let roll: f64 = rng.gen();
    if floor < 5 {
        MineEnemy::GreenSlime
    } else if floor < 10 {
        if roll < 0.6 {
            MineEnemy::GreenSlime
        } else {
            MineEnemy::Bat
        }
    } else {
        if roll < 0.35 {
            MineEnemy::GreenSlime
        } else if roll < 0.65 {
            MineEnemy::Bat
        } else {
            MineEnemy::RockCrab
        }
    }
}

/// Build an EnemyBlueprint with stats scaled to floor depth.
fn make_enemy_blueprint(kind: MineEnemy, floor: u8, x: i32, y: i32) -> EnemyBlueprint {
    let f = floor as f32;
    match kind {
        MineEnemy::GreenSlime => EnemyBlueprint {
            x,
            y,
            kind,
            health: 20.0 + f,
            max_health: 20.0 + f,
            damage: 5.0 + f / 2.0,
            speed: 24.0, // slow
        },
        MineEnemy::Bat => EnemyBlueprint {
            x,
            y,
            kind,
            health: 15.0 + f,
            max_health: 15.0 + f,
            damage: 8.0 + f / 2.0,
            speed: 48.0, // fast
        },
        MineEnemy::RockCrab => EnemyBlueprint {
            x,
            y,
            kind,
            health: 40.0 + f,
            max_health: 40.0 + f,
            damage: 12.0 + f / 2.0,
            speed: 16.0, // very slow but tanky
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use std::collections::HashSet;

    #[test]
    fn generate_floor_produces_valid_output_for_key_floors() {
        for floor in [1_u8, 5, 10, 15, 20] {
            let bp = generate_floor(floor);

            assert!(
                (0..MINE_WIDTH).contains(&bp.ladder_pos.0),
                "ladder x out of bounds on floor {floor}: {:?}",
                bp.ladder_pos
            );
            assert!(
                (0..MINE_HEIGHT).contains(&bp.ladder_pos.1),
                "ladder y out of bounds on floor {floor}: {:?}",
                bp.ladder_pos
            );

            assert!(
                !bp.rocks.is_empty(),
                "expected >0 rocks on floor {floor}, got 0"
            );

            let mut enemy_positions = HashSet::new();
            for enemy in &bp.enemies {
                assert!(
                    (0..MINE_WIDTH).contains(&enemy.x),
                    "enemy x out of bounds on floor {floor}: ({}, {})",
                    enemy.x,
                    enemy.y
                );
                assert!(
                    (0..MINE_HEIGHT).contains(&enemy.y),
                    "enemy y out of bounds on floor {floor}: ({}, {})",
                    enemy.x,
                    enemy.y
                );
                assert!(
                    enemy_positions.insert((enemy.x, enemy.y)),
                    "duplicate enemy position on floor {floor}: ({}, {})",
                    enemy.x,
                    enemy.y
                );
            }
        }
    }

    #[test]
    fn enemy_count_for_floor_is_reasonable() {
        for seed in 0_u64..100 {
            let mut rng_floor_1 = StdRng::seed_from_u64(seed);
            let floor_1_count = enemy_count_for_floor(1, &mut rng_floor_1);
            assert!(
                (1..=3).contains(&floor_1_count),
                "floor 1 enemy count out of expected small range: {floor_1_count}"
            );

            let mut rng_floor_20 = StdRng::seed_from_u64(seed);
            let floor_20_count = enemy_count_for_floor(20, &mut rng_floor_20);
            assert!(
                floor_20_count > floor_1_count,
                "floor 20 enemy count should be larger than floor 1 ({floor_20_count} <= {floor_1_count})"
            );
        }
    }

    #[test]
    fn ladder_position_safety_bound_holds_across_many_floors() {
        for floor in 1_u8..=100 {
            let bp = generate_floor(floor);
            assert!(
                (0..MINE_WIDTH).contains(&bp.ladder_pos.0),
                "ladder x out of bounds on floor {floor}: {:?}",
                bp.ladder_pos
            );
            assert!(
                (0..MINE_HEIGHT).contains(&bp.ladder_pos.1),
                "ladder y out of bounds on floor {floor}: {:?}",
                bp.ladder_pos
            );
        }
    }
}
