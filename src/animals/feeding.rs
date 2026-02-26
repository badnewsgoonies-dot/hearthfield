use bevy::prelude::*;
use crate::shared::*;
use super::{FeedTrough, FloatingFeedback, spawn_floating_text};

// ─────────────────────────────────────────────────────────────────────────────
// Feed-trough interaction
//
// The player "interacts" with the trough by pressing Space while the currently
// selected hotbar slot contains "hay".  We detect this via ItemRemovedEvent
// (the economy / inventory domain removes hay and emits this event when the
// player uses hay near the trough).  We then mark all animals as fed_today.
//
// Because no direct cross-domain calls are allowed, we watch for ItemRemovedEvent
// with item_id == "hay" and apply the feeding effect.
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_feed_trough_interact(
    mut commands: Commands,
    mut removed_events: EventReader<ItemRemovedEvent>,
    mut animal_query: Query<(Entity, &mut Animal, &Transform)>,
    trough_query: Query<&FeedTrough>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for ev in removed_events.read() {
        if ev.item_id != "hay" {
            continue;
        }

        // Hay was consumed — feed all barn/coop animals.
        let mut fed_count = 0u32;
        for (entity, mut animal, transform) in animal_query.iter_mut() {
            if matches!(
                animal.kind,
                AnimalKind::Chicken | AnimalKind::Cow | AnimalKind::Sheep
            ) {
                animal.fed_today = true;
                fed_count += 1;

                // Spawn a little "+" feedback above each animal.
                spawn_floating_text(
                    &mut commands,
                    transform.translation + Vec3::new(0.0, 14.0, 2.0),
                    "Yum!",
                    Color::srgb(0.3, 0.9, 0.3),
                );
            }
        }

        if fed_count > 0 {
            sfx_writer.send(PlaySfxEvent {
                sfx_id: "feed_animals".to_string(),
            });
        }
    }
}
