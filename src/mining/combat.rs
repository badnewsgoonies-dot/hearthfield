//! Combat system for the mine.
//!
//! - Enemies move toward the player each tick.
//! - When adjacent, enemies attack the player (reducing health).
//! - The player attacks by using the pickaxe (or approaching with action key)
//!   on an adjacent enemy — we listen for ToolUseEvent with Pickaxe targeting
//!   an enemy's grid cell.
//! - Dead enemies drop loot.

use bevy::prelude::*;
use rand::prelude::*;

use crate::shared::*;
use super::components::*;
use super::floor_gen::{MINE_WIDTH, MINE_HEIGHT};

/// Player combat damage based on pickaxe tier (doubles as weapon).
fn player_attack_damage(tier: ToolTier) -> f32 {
    match tier {
        ToolTier::Basic => 10.0,
        ToolTier::Copper => 15.0,
        ToolTier::Iron => 20.0,
        ToolTier::Gold => 30.0,
        ToolTier::Iridium => 50.0,
    }
}

/// System: player attacks an enemy with the pickaxe.
pub fn handle_player_attack(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    mut enemies: Query<(Entity, &MineGridPos, &mut MineMonster)>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    for event in tool_events.read() {
        if event.tool != ToolKind::Pickaxe {
            continue;
        }

        let damage = player_attack_damage(event.tier);
        let mut killed = None;

        for (entity, grid_pos, mut monster) in enemies.iter_mut() {
            if grid_pos.x == event.target_x && grid_pos.y == event.target_y {
                monster.health -= damage;

                sfx_events.send(PlaySfxEvent {
                    sfx_id: "mine_enemy_hit".to_string(),
                });

                if monster.health <= 0.0 {
                    killed = Some((entity, monster.kind));
                }
                break;
            }
        }

        if let Some((entity, kind)) = killed {
            commands.entity(entity).despawn();

            sfx_events.send(PlaySfxEvent {
                sfx_id: "mine_enemy_die".to_string(),
            });

            // Drop loot based on enemy type
            let (item_id, qty) = enemy_loot(kind);
            pickup_events.send(ItemPickupEvent {
                item_id,
                quantity: qty,
            });
        }
    }
}

/// Determine loot dropped by a killed enemy.
fn enemy_loot(kind: MineEnemy) -> (String, u8) {
    let mut rng = rand::thread_rng();
    match kind {
        MineEnemy::GreenSlime => {
            let roll: f64 = rng.gen();
            if roll < 0.3 {
                ("slime_jelly".to_string(), rng.gen_range(1..=2))
            } else if roll < 0.5 {
                ("copper_ore".to_string(), 1)
            } else {
                ("stone".to_string(), rng.gen_range(1..=3))
            }
        }
        MineEnemy::Bat => {
            let roll: f64 = rng.gen();
            if roll < 0.25 {
                ("bat_wing".to_string(), 1)
            } else if roll < 0.5 {
                ("iron_ore".to_string(), 1)
            } else if roll < 0.7 {
                ("copper_ore".to_string(), rng.gen_range(1..=2))
            } else {
                ("stone".to_string(), rng.gen_range(1..=2))
            }
        }
        MineEnemy::RockCrab => {
            let roll: f64 = rng.gen();
            if roll < 0.2 {
                ("crab_shell".to_string(), 1)
            } else if roll < 0.45 {
                ("gold_ore".to_string(), 1)
            } else if roll < 0.65 {
                ("iron_ore".to_string(), rng.gen_range(1..=2))
            } else {
                ("stone".to_string(), rng.gen_range(2..=4))
            }
        }
    }
}

/// System: enemy AI movement toward the player.
/// Enemies move one tile toward the player on their move timer tick.
pub fn enemy_ai_movement(
    time: Res<Time>,
    mut enemies: Query<(&mut MineGridPos, &mut Transform, &MineMonster, &mut EnemyMoveTick)>,
    rocks: Query<&MineGridPos, (With<MineRock>, Without<MineMonster>)>,
    active_floor: Res<ActiveFloor>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 || !active_floor.spawned {
        return;
    }

    let player_x = active_floor.player_grid_x;
    let player_y = active_floor.player_grid_y;

    // Build a set of occupied tiles (rocks block enemy movement)
    let rock_positions: std::collections::HashSet<(i32, i32)> = rocks
        .iter()
        .map(|p| (p.x, p.y))
        .collect();

    for (mut grid_pos, mut transform, _monster, mut move_tick) in enemies.iter_mut() {
        move_tick.timer.tick(time.delta());
        if !move_tick.timer.just_finished() {
            continue;
        }

        // Simple greedy pathfinding: move in the axis that reduces Manhattan distance most
        let dx = player_x - grid_pos.x;
        let dy = player_y - grid_pos.y;

        // Try to move toward the player. Prioritize the larger axis difference.
        let (primary_dx, primary_dy, secondary_dx, secondary_dy) = if dx.abs() >= dy.abs() {
            (dx.signum(), 0, 0, dy.signum())
        } else {
            (0, dy.signum(), dx.signum(), 0)
        };

        let candidates = [
            (grid_pos.x + primary_dx, grid_pos.y + primary_dy),
            (grid_pos.x + secondary_dx, grid_pos.y + secondary_dy),
        ];

        for (nx, ny) in candidates {
            // Check bounds (stay within walkable area)
            if nx < 1 || nx >= MINE_WIDTH - 1 || ny < 0 || ny >= MINE_HEIGHT - 1 {
                continue;
            }
            // Don't walk into rocks
            if rock_positions.contains(&(nx, ny)) {
                continue;
            }
            // Don't walk onto the exact player tile (they attack from adjacent)
            if nx == player_x && ny == player_y {
                // Stay adjacent, don't step onto the player
                break;
            }
            // Move here
            grid_pos.x = nx;
            grid_pos.y = ny;
            transform.translation.x = nx as f32 * TILE_SIZE;
            transform.translation.y = ny as f32 * TILE_SIZE;
            break;
        }
    }
}

/// System: enemies attack the player when adjacent.
pub fn enemy_attack_player(
    time: Res<Time>,
    mut enemies: Query<(&MineGridPos, &MineMonster, &mut EnemyAttackCooldown)>,
    active_floor: Res<ActiveFloor>,
    mut player_state: ResMut<PlayerState>,
    mut iframes: ResMut<PlayerIFrames>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 || !active_floor.spawned {
        return;
    }

    // Tick iframes
    iframes.timer.tick(time.delta());

    // If player still has invincibility, skip
    if !iframes.timer.finished() {
        return;
    }

    let px = active_floor.player_grid_x;
    let py = active_floor.player_grid_y;

    for (grid_pos, monster, mut cooldown) in enemies.iter_mut() {
        cooldown.timer.tick(time.delta());
        if !cooldown.timer.just_finished() {
            continue;
        }

        // Check adjacency (Manhattan distance == 1)
        let dist = (grid_pos.x - px).abs() + (grid_pos.y - py).abs();
        if dist <= 1 {
            // Attack!
            player_state.health = (player_state.health - monster.damage).max(0.0);

            sfx_events.send(PlaySfxEvent {
                sfx_id: "player_hurt".to_string(),
            });

            // Grant brief invincibility to prevent multi-hit stacking
            iframes.timer = Timer::from_seconds(0.5, TimerMode::Once);

            // If player dies, we'll handle that in a separate system
            break; // Only take damage from one enemy per frame
        }
    }
}

/// System: check if the player's health has reached zero (knockout).
/// On knockout, exit the mine, set health to a fraction, and lose some gold.
pub fn check_player_knockout(
    mut player_state: ResMut<PlayerState>,
    mut mine_state: ResMut<MineState>,
    mut active_floor: ResMut<ActiveFloor>,
    mut in_mine: ResMut<InMine>,
    mut map_events: EventWriter<MapTransitionEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    if !in_mine.0 {
        return;
    }

    if player_state.health <= 0.0 {
        // Knockout! Player wakes up at mine entrance with reduced health/gold.
        sfx_events.send(PlaySfxEvent {
            sfx_id: "player_knockout".to_string(),
        });

        // Lose 10% of gold (min 0)
        let gold_loss = (player_state.gold as f32 * 0.10) as i32;
        if gold_loss > 0 {
            gold_events.send(GoldChangeEvent {
                amount: -gold_loss,
                reason: "Knocked out in the mine".to_string(),
            });
        }

        // Restore partial health
        player_state.health = player_state.max_health * 0.5;

        // Reset mine state
        mine_state.current_floor = 0;
        in_mine.0 = false;
        active_floor.spawned = false;

        // Transition back to mine entrance
        map_events.send(MapTransitionEvent {
            to_map: MapId::MineEntrance,
            to_x: 7,
            to_y: 4,
        });
    }
}
