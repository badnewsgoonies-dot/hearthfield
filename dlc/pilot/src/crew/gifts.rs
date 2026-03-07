//! Gift-giving system for crew relationships.

use crate::shared::*;
use bevy::prelude::*;

pub fn handle_gift_given(
    mut events: EventReader<GiftGivenEvent>,
    crew_registry: Res<CrewRegistry>,
    mut relationships: ResMut<Relationships>,
    mut inventory: ResMut<Inventory>,
    mut toast_events: EventWriter<ToastEvent>,
    mut friendship_events: EventWriter<FriendshipChangeEvent>,
) {
    for ev in events.read() {
        let Some(member) = crew_registry.members.get(&ev.npc_id) else {
            continue;
        };

        // Check already gifted today
        if relationships
            .gifts_given_today
            .get(&ev.npc_id)
            .copied()
            .unwrap_or(false)
        {
            toast_events.send(ToastEvent {
                message: format!("{} already received a gift today.", member.name),
                duration_secs: 2.0,
            });
            continue;
        }

        // Remove item from inventory
        if !inventory.remove_item(&ev.item_id, 1) {
            continue;
        }

        // Determine gift reaction
        let (amount, reaction) = if ev.item_id == member.favorite_gift {
            (15, "Loved")
        } else if ev.item_id == member.disliked_gift {
            (-10, "Hated")
        } else {
            (5, "Liked")
        };

        relationships.add_friendship(&ev.npc_id, amount);
        relationships
            .gifts_given_today
            .insert(ev.npc_id.clone(), true);

        friendship_events.send(FriendshipChangeEvent {
            npc_id: ev.npc_id.clone(),
            amount,
        });

        toast_events.send(ToastEvent {
            message: format!("{}: {} it!", member.name, reaction),
            duration_secs: 3.0,
        });
    }
}

pub fn reset_daily_gifts(
    mut day_end_events: EventReader<DayEndEvent>,
    mut relationships: ResMut<Relationships>,
) {
    for _ev in day_end_events.read() {
        relationships.gifts_given_today.clear();
    }
}
