//! Rock breaking system — listens for ToolUseEvent with Pickaxe and damages mine rocks.

use bevy::prelude::*;

use crate::shared::*;
use super::components::*;

/// Pickaxe damage per tool tier.
fn pickaxe_damage(tier: ToolTier) -> u8 {
    match tier {
        ToolTier::Basic => 1,
        ToolTier::Copper => 1,
        ToolTier::Iron => 2,
        ToolTier::Gold => 3,
        ToolTier::Iridium => 4,
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

            // Check if this rock had the ladder hidden in it, or if all rocks are gone
            check_ladder_reveal(&mut active_floor, &mut ladders, rx, ry);
        }
    }
}

/// Reveal the ladder if the broken rock contained it, or if all rocks are destroyed.
fn check_ladder_reveal(
    active_floor: &mut ActiveFloor,
    ladders: &mut Query<(&MineGridPos, &mut MineLadder, &mut Sprite), Without<MineRock>>,
    broken_x: i32,
    broken_y: i32,
) {
    if active_floor.ladder_revealed {
        return;
    }

    let should_reveal = active_floor.rocks_remaining == 0;

    for (grid_pos, mut ladder, mut sprite) in ladders.iter_mut() {
        // Reveal if we broke the rock containing the ladder, or all rocks are gone
        if !ladder.revealed
            && (should_reveal || (grid_pos.x == broken_x && grid_pos.y == broken_y))
        {
            ladder.revealed = true;
            sprite.color = Color::srgb(0.65, 0.50, 0.25); // LADDER_COLOR
            active_floor.ladder_revealed = true;
        }
    }
}
