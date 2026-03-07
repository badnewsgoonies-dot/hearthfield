//! Rock breaking system — listens for ToolUseEvent with Pickaxe and damages mine rocks.

use bevy::prelude::*;
use rand::prelude::*;

use super::components::*;
use crate::shared::*;

/// Pickaxe damage per tool tier.
fn pickaxe_damage(tier: ToolTier) -> u8 {
    match tier {
        ToolTier::Basic => 1,
        ToolTier::Copper => 2,
        ToolTier::Iron => 3,
        ToolTier::Gold => 4,
        ToolTier::Iridium => 5,
    }
}

/// Stamina cost for a pickaxe swing.
fn pickaxe_stamina_cost(tier: ToolTier) -> f32 {
    match tier {
        ToolTier::Basic => 4.0,
        ToolTier::Copper => 3.5,
        ToolTier::Iron => 3.0,
        ToolTier::Gold => 2.5,
        ToolTier::Iridium => 2.0,
    }
}

/// System: handle pickaxe hits on mine rocks.
#[allow(clippy::too_many_arguments)]
pub fn handle_rock_breaking(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    mut rocks: Query<(Entity, &MineGridPos, &mut MineRock)>,
    mut active_floor: ResMut<ActiveFloor>,
    mut ladders: Query<(&MineGridPos, &mut MineLadder, &mut Sprite), Without<MineRock>>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    for event in tool_events.read() {
        if event.tool != ToolKind::Pickaxe {
            continue;
        }

        let damage = pickaxe_damage(event.tier);
        let stamina_cost = pickaxe_stamina_cost(event.tier);

        // Find a rock at the target position
        let mut hit_rock = None;
        for (entity, grid_pos, mut rock) in rocks.iter_mut() {
            if grid_pos.x == event.target_x && grid_pos.y == event.target_y {
                // Apply damage
                let _effective_dmg = damage.min(rock.health);
                rock.health = rock.health.saturating_sub(damage);

                sfx_events.send(PlaySfxEvent {
                    sfx_id: "mine_rock_hit".to_string(),
                });

                // Drain stamina
                stamina_events.send(StaminaDrainEvent {
                    amount: stamina_cost,
                });

                if rock.health == 0 {
                    // Rock is destroyed
                    let drop_item = rock.drop_item.clone();
                    let drop_qty = rock.drop_quantity;
                    hit_rock = Some((entity, drop_item, drop_qty, grid_pos.x, grid_pos.y));
                }
                break;
            }
        }

        if let Some((entity, drop_item, drop_qty, rx, ry)) = hit_rock {
            // Despawn the rock
            commands.entity(entity).despawn();

            // Drop loot
            pickup_events.send(ItemPickupEvent {
                item_id: drop_item,
                quantity: drop_qty,
            });

            sfx_events.send(PlaySfxEvent {
                sfx_id: "mine_rock_break".to_string(),
            });

            // Track rock count
            active_floor.rocks_remaining = active_floor.rocks_remaining.saturating_sub(1);
            active_floor.rocks_broken_this_floor += 1;

            // Check if this rock had the ladder hidden in it, or probability triggers
            check_ladder_reveal(&mut active_floor, &mut ladders, rx, ry);
        }
    }
}

/// Reveal the ladder if the broken rock contained it, probability triggers, or all rocks gone.
///
/// Ladder probability: 5% base + 2% per rock broken this floor, max 30%.
/// If the broken rock directly contains the ladder, it always reveals.
/// If all rocks are destroyed, the ladder is always revealed.
fn check_ladder_reveal(
    active_floor: &mut ActiveFloor,
    ladders: &mut Query<(&MineGridPos, &mut MineLadder, &mut Sprite), Without<MineRock>>,
    broken_x: i32,
    broken_y: i32,
) {
    if active_floor.ladder_revealed {
        return;
    }

    // Always reveal if all rocks are gone
    let all_gone = active_floor.rocks_remaining == 0;

    // Probability-based reveal: 5% base + 2% per rock broken, max 30%
    let probability = (0.05 + active_floor.rocks_broken_this_floor as f64 * 0.02).min(0.30);
    let prob_reveal = rand::thread_rng().gen_bool(probability);

    for (grid_pos, mut ladder, mut sprite) in ladders.iter_mut() {
        // Reveal if: rock at ladder position was broken, all rocks gone, or probability hit
        let is_ladder_rock = grid_pos.x == broken_x && grid_pos.y == broken_y;
        if !ladder.revealed && (all_gone || is_ladder_rock || prob_reveal) {
            ladder.revealed = true;
            sprite.color = Color::srgb(0.65, 0.50, 0.25); // LADDER_COLOR
            active_floor.ladder_revealed = true;
        }
    }
}
