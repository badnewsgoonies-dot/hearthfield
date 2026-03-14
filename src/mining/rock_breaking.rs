//! Rock breaking system — listens for ToolUseEvent with Pickaxe and damages mine rocks.

use bevy::prelude::*;
use rand::prelude::*;

use super::components::*;
use super::rock_impact::{RockDestroyedEvent, RockHitEvent};
use super::spawning::{cave_tiles, MiningAtlases};
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
        ToolTier::Basic => 3.5,
        ToolTier::Copper => 3.0,
        ToolTier::Iron => 2.6,
        ToolTier::Gold => 2.2,
        ToolTier::Iridium => 1.8,
    }
}

/// System: handle pickaxe hits on mine rocks.
#[allow(clippy::too_many_arguments)]
pub fn handle_rock_breaking(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    mut rocks: Query<(Entity, &MineGridPos, &mut MineRock, &Transform)>,
    mut active_floor: ResMut<ActiveFloor>,
    mut ladders: Query<(&MineGridPos, &mut MineLadder, &mut Sprite), Without<MineRock>>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut rock_hit_events: EventWriter<RockHitEvent>,
    mut rock_destroyed_events: EventWriter<RockDestroyedEvent>,
    in_mine: Res<InMine>,
    atlases: Res<MiningAtlases>,
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
        // hit_rock = Some((entity, drop_item, drop_qty, grid_x, grid_y, world_x, world_y))
        let mut hit_rock = None;
        let mut hit_and_survived: Option<(Entity, f32, f32)> = None;

        for (entity, grid_pos, mut rock, transform) in rocks.iter_mut() {
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

                let wx = transform.translation.x;
                let wy = transform.translation.y;

                if rock.health == 0 {
                    // Rock is destroyed
                    let drop_item = rock.drop_item.clone();
                    let drop_qty = rock.drop_quantity;
                    hit_rock = Some((entity, drop_item, drop_qty, grid_pos.x, grid_pos.y, wx, wy));
                } else {
                    // Rock survived — queue hit feedback
                    hit_and_survived = Some((entity, wx, wy));
                }
                break;
            }
        }

        // Fire hit-survived feedback event
        if let Some((entity, wx, wy)) = hit_and_survived {
            rock_hit_events.send(RockHitEvent {
                rock_entity: entity,
                world_x: wx,
                world_y: wy,
            });
        }

        if let Some((entity, drop_item, drop_qty, rx, ry, wx, wy)) = hit_rock {
            // Fire destruction feedback event BEFORE despawning
            rock_destroyed_events.send(RockDestroyedEvent {
                world_x: wx,
                world_y: wy,
            });

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
            check_ladder_reveal(&mut active_floor, &mut ladders, rx, ry, &atlases);
        }
    }
}

/// Reveal the ladder if the broken rock contained it, probability triggers, or all rocks gone.
///
/// Ladder probability: 2% base + 4% per rock broken this floor, max 55%.
/// If the broken rock directly contains the ladder, it always reveals.
/// If all rocks are destroyed, the ladder is always revealed.
fn check_ladder_reveal(
    active_floor: &mut ActiveFloor,
    ladders: &mut Query<(&MineGridPos, &mut MineLadder, &mut Sprite), Without<MineRock>>,
    broken_x: i32,
    broken_y: i32,
    atlases: &MiningAtlases,
) {
    if active_floor.ladder_revealed {
        return;
    }

    // Always reveal if all rocks are gone
    let all_gone = active_floor.rocks_remaining == 0;

    // Probability-based reveal: 2% base + 4% per rock broken, max 55%
    let probability = (0.02 + active_floor.rocks_broken_this_floor as f64 * 0.04).min(0.55);
    let prob_reveal = rand::thread_rng().gen_bool(probability);

    for (grid_pos, mut ladder, mut sprite) in ladders.iter_mut() {
        // Reveal if: rock at ladder position was broken, all rocks gone, or probability hit
        let is_ladder_rock = grid_pos.x == broken_x && grid_pos.y == broken_y;
        if !ladder.revealed && (all_gone || is_ladder_rock || prob_reveal) {
            ladder.revealed = true;
            active_floor.ladder_revealed = true;

            // Swap the sprite to show the ladder tile from the cave atlas
            if atlases.loaded {
                *sprite = Sprite::from_atlas_image(
                    atlases.cave_image.clone(),
                    TextureAtlas {
                        layout: atlases.cave_layout.clone(),
                        index: cave_tiles::LADDER,
                    },
                );
                sprite.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
            } else {
                sprite.color = Color::srgb(0.82, 0.68, 0.32);
            }
        }
    }
}
