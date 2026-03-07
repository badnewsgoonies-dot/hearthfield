use super::{spawn_floating_text, FeedTrough};
use crate::shared::*;
use bevy::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Feed-trough interaction
//
// The player feeds animals by placing hay at the feed trough. We detect this
// via `ItemRemovedEvent` with item_id == "hay". This event is emitted by the
// economy/inventory domain when the player uses hay near the trough (the
// inventory domain owns the consumption logic).
//
// When hay is consumed:
//   - All barn/coop animals (Chicken, Cow, Sheep) are marked fed_today = true.
//   - A sound effect plays.
//   - Small "Yum!" floating text appears above each fed animal.
//
// The direct proximity / Space-press interaction path (player near trough with
// hay in hotbar) is handled by the player domain which sends ItemRemovedEvent.
// We listen for that event here and apply the feeding effect. This keeps the
// animals domain fully decoupled — it never reads from the inventory directly.
//
// Edge case: if the player feeds multiple times in one day (multiple hay
// removed events), subsequent feeds are harmless because fed_today is
// idempotent (setting true twice is fine).
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_feed_trough_interact(
    mut commands: Commands,
    mut removed_events: EventReader<ItemRemovedEvent>,
    mut animal_query: Query<(Entity, &mut Animal, &LogicalPosition)>,
    trough_query: Query<&FeedTrough>,
    player_query: Query<&GridPosition, With<Player>>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    let Ok(player_gp) = player_query.get_single() else {
        return;
    };

    // Check if the player is adjacent (Manhattan distance ≤ 2) to any feed trough.
    let near_trough = trough_query.iter().any(|trough| {
        (trough.grid_x - player_gp.x).abs() + (trough.grid_y - player_gp.y).abs() <= 2
    });

    for ev in removed_events.read() {
        if ev.item_id != "hay" {
            continue;
        }

        if !near_trough {
            continue;
        }

        // Hay was consumed — feed all productive barn/coop animals.
        let mut fed_count = 0u32;
        for (_entity, mut animal, animal_lp) in animal_query.iter_mut() {
            if !matches!(
                animal.kind,
                AnimalKind::Chicken
                    | AnimalKind::Cow
                    | AnimalKind::Sheep
                    | AnimalKind::Goat
                    | AnimalKind::Duck
                    | AnimalKind::Rabbit
                    | AnimalKind::Pig
            ) {
                // Companions (Cat, Dog, Horse) don't need hay.
                continue;
            }

            if !animal.fed_today {
                animal.fed_today = true;
                fed_count += 1;

                // Floating "Yum!" feedback above each newly-fed animal.
                spawn_floating_text(
                    &mut commands,
                    animal_lp.0.extend(Z_EFFECTS) + Vec3::new(0.0, 14.0, 0.0),
                    "Yum!",
                    Color::srgb(0.3, 0.9, 0.3),
                );
            }
        }

        if fed_count > 0 {
            sfx_writer.send(PlaySfxEvent {
                sfx_id: "feed_animals".to_string(),
            });
            toast_writer.send(ToastEvent {
                message: format!(
                    "Fed {} animal{}!",
                    fed_count,
                    if fed_count == 1 { "" } else { "s" }
                ),
                duration_secs: 2.0,
            });
        }
    }
}
