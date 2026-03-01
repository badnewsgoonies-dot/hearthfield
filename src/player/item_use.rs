//! Item Use Dispatcher (R key)
//!
//! When R is pressed with an item selected, dispatches based on item properties.
//! All action inputs use just_pressed (edge-triggered) — no debounce needed.

use bevy::prelude::*;
use crate::shared::*;

// Domain event imports — use pub re-exports from domain mod.rs.
use crate::crafting::PlaceMachineEvent;

pub fn dispatch_item_use(
    player_input: Res<PlayerInput>,
    player_state: Res<PlayerState>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    input_blocks: Res<InputBlocks>,
    interaction_claimed: Res<InteractionClaimed>,
    player_query: Query<(&GridPosition, &PlayerMovement), With<Player>>,
    npc_query: Query<(&Npc, &Transform)>,
    logical_pos_query: Query<&LogicalPosition, With<Player>>,
    mut eat_events: EventWriter<EatFoodEvent>,
    mut sprinkler_events: EventWriter<PlaceSprinklerEvent>,
    mut place_machine_events: EventWriter<PlaceMachineEvent>,
    mut bouquet_events: EventWriter<BouquetGivenEvent>,
    mut proposal_events: EventWriter<ProposalEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if input_blocks.is_blocked() || !player_input.tool_secondary || interaction_claimed.0 {
        return;
    }

    let slot_idx = inventory.selected_slot;
    let Some(ref slot) = inventory.slots.get(slot_idx).and_then(|s| s.as_ref()) else {
        return;
    };
    let Some(def) = item_registry.get(&slot.item_id) else {
        return;
    };
    let item_id = &slot.item_id;

    let Ok((grid_pos, movement)) = player_query.get_single() else {
        return;
    };

    // ── FOOD ──────────────────────────────────────────────────────
    if def.edible {
        // Send EatFoodEvent with buff: None — the handle_eat_food system
        // in crafting/buffs.rs resolves the buff via food_buff_for_item().
        eat_events.send(EatFoodEvent {
            item_id: item_id.clone(),
            stamina_restore: def.energy_restore,
            buff: crate::crafting::food_buff_for_item(item_id),
        });
        return;
    }

    // For placement items, compute the tile the player is facing.
    let (dx, dy) = super::facing_offset(&movement.facing);
    let target_x = grid_pos.x + dx;
    let target_y = grid_pos.y + dy;

    // ── SPRINKLERS ────────────────────────────────────────────────
    if matches!(
        item_id.as_str(),
        "sprinkler" | "quality_sprinkler" | "iridium_sprinkler"
    ) {
        if player_state.current_map != MapId::Farm {
            toast_events.send(ToastEvent {
                message: "Sprinklers can only be placed on the farm.".into(),
                duration_secs: 2.0,
            });
            return;
        }
        let kind = match item_id.as_str() {
            "quality_sprinkler" => SprinklerKind::Quality,
            "iridium_sprinkler" => SprinklerKind::Iridium,
            _ => SprinklerKind::Basic,
        };
        sprinkler_events.send(PlaceSprinklerEvent {
            kind,
            tile_x: target_x,
            tile_y: target_y,
        });
        return;
    }

    // ── MACHINES ──────────────────────────────────────────────────
    if matches!(
        item_id.as_str(),
        "furnace" | "preserves_jar" | "cheese_press" | "loom" | "keg" | "oil_maker"
    ) {
        if player_state.current_map != MapId::Farm {
            toast_events.send(ToastEvent {
                message: "Machines can only be placed on the farm.".into(),
                duration_secs: 2.0,
            });
            return;
        }
        place_machine_events.send(PlaceMachineEvent {
            item_id: item_id.clone(),
            grid_x: target_x,
            grid_y: target_y,
        });
        return;
    }

    // ── BOUQUET → NPC (dating trigger) ────────────────────────────
    if item_id == "bouquet" {
        if let Some(npc_id) = find_nearest_npc(&logical_pos_query, &npc_query) {
            bouquet_events.send(BouquetGivenEvent { npc_name: npc_id });
        } else {
            toast_events.send(ToastEvent {
                message: "No one nearby to give this to.".into(),
                duration_secs: 2.0,
            });
        }
        return;
    }

    // ── MERMAID PENDANT → NPC (proposal trigger) ──────────────────
    if item_id == "mermaid_pendant" {
        if let Some(npc_id) = find_nearest_npc(&logical_pos_query, &npc_query) {
            proposal_events.send(ProposalEvent { npc_name: npc_id });
        } else {
            toast_events.send(ToastEvent {
                message: "No one nearby to give this to.".into(),
                duration_secs: 2.0,
            });
        }
        return;
    }
}

fn find_nearest_npc(
    player_pos_query: &Query<&LogicalPosition, With<Player>>,
    npc_query: &Query<(&Npc, &Transform)>,
) -> Option<String> {
    let Ok(player_pos) = player_pos_query.get_single() else {
        return None;
    };
    let range = TILE_SIZE * 1.5;
    let mut best: Option<(f32, String)> = None;
    for (npc, tf) in npc_query.iter() {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range && (best.is_none() || d < best.as_ref().unwrap().0) {
            best = Some((d, npc.id.clone()));
        }
    }
    best.map(|(_, id)| id)
}
