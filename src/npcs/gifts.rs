//! Gift system: handle GiftGivenEvent, apply friendship points, send response dialogue.

use bevy::prelude::*;
use crate::shared::*;
use super::dialogue::build_gift_response_lines;

/// System: process gift-given events, apply friendship changes, send response dialogue.
pub fn handle_gifts(
    mut gift_reader: EventReader<GiftGivenEvent>,
    mut relationships: ResMut<Relationships>,
    npc_registry: Res<NpcRegistry>,
    item_registry: Res<ItemRegistry>,
    calendar: Res<Calendar>,
    mut dialogue_writer: EventWriter<DialogueStartEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for gift_event in gift_reader.read() {
        let npc_id = &gift_event.npc_id;
        let item_id = &gift_event.item_id;

        // Check if NPC has already received a gift today
        if relationships.gifted_today.get(npc_id).copied().unwrap_or(false) {
            // Send polite decline dialogue
            let _npc_name = npc_registry.npcs.get(npc_id)
                .map(|d| d.name.as_str())
                .unwrap_or("them");
            let decline_lines = vec![
                format!("You've already given me a gift today. That's very generous, but once a day is enough!"),
            ];
            let portrait_index = npc_registry.npcs.get(npc_id).map(|d| d.portrait_index);
            dialogue_writer.send(DialogueStartEvent {
                npc_id: npc_id.clone(),
                lines: decline_lines,
                portrait_index,
            });
            next_state.set(GameState::Dialogue);
            continue;
        }

        // Look up the NPC definition
        let Some(npc_def) = npc_registry.npcs.get(npc_id) else {
            continue;
        };

        // Determine gift preference
        let preference = npc_def.gift_preferences.get(item_id)
            .copied()
            .unwrap_or(GiftPreference::Neutral);

        // Check if today is the NPC's birthday
        let is_birthday = calendar.season == npc_def.birthday_season
            && calendar.day == npc_def.birthday_day;

        // Calculate friendship points
        let base_points = preference_to_points(preference);
        let multiplier = if is_birthday { 8 } else { 1 };
        let total_points = base_points * multiplier;

        // Apply friendship change
        relationships.add_friendship(npc_id, total_points);

        // Mark as gifted today
        relationships.gifted_today.insert(npc_id.clone(), true);

        // Look up item name for dialogue
        let item_name = item_registry.get(item_id)
            .map(|d| d.name.as_str())
            .unwrap_or(item_id.as_str());

        // Build gift response dialogue
        let response_lines = build_gift_response_lines(
            npc_id,
            &npc_def.name,
            preference,
            item_name,
            is_birthday,
        );

        let portrait_index = Some(npc_def.portrait_index);
        dialogue_writer.send(DialogueStartEvent {
            npc_id: npc_id.clone(),
            lines: response_lines,
            portrait_index,
        });

        next_state.set(GameState::Dialogue);
    }
}

/// Convert a GiftPreference to friendship point delta (positive or negative).
fn preference_to_points(preference: GiftPreference) -> i32 {
    match preference {
        GiftPreference::Loved    =>  80,
        GiftPreference::Liked    =>  45,
        GiftPreference::Neutral  =>  20,
        GiftPreference::Disliked => -20,
        GiftPreference::Hated    => -40,
    }
}

/// System: handle player pressing G (or the configured gift key) while in dialogue
/// with an NPC, using the selected hotbar item as the gift.
///
/// This system checks: player is adjacent to an NPC, presses G, has selected item.
/// If so, it emits a GiftGivenEvent and removes one of the item from inventory.
pub fn handle_gift_input(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(&Npc, &Transform)>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    relationships: Res<Relationships>,
    mut gift_writer: EventWriter<GiftGivenEvent>,
    mut item_removed_writer: EventWriter<ItemRemovedEvent>,
    current_state: Res<State<GameState>>,
    interaction_claimed: Res<InteractionClaimed>,
) {
    // Only in Playing state
    if *current_state.get() != GameState::Playing {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    if !player_input.interact {
        return;
    }

    if interaction_claimed.0 {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let interaction_range = TILE_SIZE * 1.5;

    // Find closest NPC
    let mut closest_npc_id: Option<String> = None;
    let mut closest_dist = f32::MAX;
    for (npc, npc_transform) in npc_query.iter() {
        let npc_pos = npc_transform.translation.truncate();
        let dist = player_pos.distance(npc_pos);
        if dist <= interaction_range && dist < closest_dist {
            closest_dist = dist;
            closest_npc_id = Some(npc.id.clone());
        }
    }

    let Some(npc_id) = closest_npc_id else {
        return;
    };

    // Already gifted today?
    if relationships.gifted_today.get(&npc_id).copied().unwrap_or(false) {
        return;
    }

    // Get selected item from hotbar
    let selected_slot = inventory.selected_slot;
    let (item_id, item_def) = {
        let slot = inventory.slots.get(selected_slot).and_then(|s| s.as_ref());
        match slot {
            Some(s) => {
                let id = s.item_id.clone();
                let def = item_registry.get(&id).cloned();
                (id, def)
            }
            None => return, // no item selected
        }
    };

    // Don't allow gifting tools or special items
    if let Some(ref def) = item_def {
        if matches!(def.category, ItemCategory::Tool | ItemCategory::Special) {
            return;
        }
    }

    // Remove one item from inventory
    inventory.try_remove(&item_id, 1);
    item_removed_writer.send(ItemRemovedEvent {
        item_id: item_id.clone(),
        quantity: 1,
    });

    // Emit gift event
    gift_writer.send(GiftGivenEvent {
        npc_id,
        item_id,
        preference: GiftPreference::Neutral, // will be looked up in handle_gifts
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_to_points_loved() {
        assert_eq!(preference_to_points(GiftPreference::Loved), 80);
    }

    #[test]
    fn test_preference_to_points_liked() {
        assert_eq!(preference_to_points(GiftPreference::Liked), 45);
    }

    #[test]
    fn test_preference_to_points_neutral() {
        assert_eq!(preference_to_points(GiftPreference::Neutral), 20);
    }

    #[test]
    fn test_preference_to_points_disliked() {
        assert_eq!(preference_to_points(GiftPreference::Disliked), -20);
    }

    #[test]
    fn test_preference_to_points_hated() {
        assert_eq!(preference_to_points(GiftPreference::Hated), -40);
    }

    #[test]
    fn test_birthday_multiplier_calculation() {
        // Simulate birthday logic: base_points * 8
        let base = preference_to_points(GiftPreference::Loved);
        let birthday_total = base * 8;
        assert_eq!(birthday_total, 640);
    }

    #[test]
    fn test_non_birthday_multiplier() {
        let base = preference_to_points(GiftPreference::Liked);
        let normal_total = base * 1;
        assert_eq!(normal_total, 45);
    }
}
