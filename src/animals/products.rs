use bevy::prelude::*;
use crate::shared::*;
use super::{ProductReadyIndicator, spawn_floating_text};
use super::day_end::PendingProductQuality;

// ─────────────────────────────────────────────────────────────────────────────
// Product collection
//
// When the player presses Space near an animal that has product_ready == true:
//   1. Collect the product — set product_ready = false.
//   2. Determine quality from PendingProductQuality (written at day-end).
//   3. Send AnimalProductEvent (cross-domain signal for stats/achievements).
//   4. Send ItemPickupEvent so the inventory domain adds the item.
//   5. Send ToastEvent showing the quality label (for non-Normal quality).
//   6. Send PlaySfxEvent for audio feedback.
//   7. Spawn floating text above the animal.
//   8. Remove PendingProductQuality from the entity (consumed).
//
// Priority: handle_animal_interact in interaction.rs checks for product_ready
// first and defers to this system, so petting and collection never conflict.
// ─────────────────────────────────────────────────────────────────────────────

const INTERACT_RANGE: f32 = 32.0;

pub fn handle_product_collection(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut animal_query: Query<(Entity, &mut Animal, &Transform, Option<&PendingProductQuality>)>,
    mut product_writer: EventWriter<AnimalProductEvent>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (entity, mut animal, animal_transform, pending_quality) in animal_query.iter_mut() {
        if !animal.product_ready {
            continue;
        }

        let animal_pos = animal_transform.translation.truncate();
        if player_pos.distance(animal_pos) > INTERACT_RANGE {
            continue;
        }

        // Map animal kind to item id and a human-readable product name.
        let (product_id, product_display) = match animal.kind {
            AnimalKind::Chicken => ("egg", "Egg"),
            AnimalKind::Cow => ("milk", "Milk"),
            AnimalKind::Sheep => ("wool", "Wool"),
            // Cats and dogs are pets — they have no harvestable product.
            // product_ready should never be true for pets, but guard anyway.
            _ => continue,
        };

        // Read the quality that was decided at day-end; fall back to Normal if
        // the component is somehow missing (e.g. save-load edge case).
        let quality = pending_quality
            .map(|pq| pq.quality)
            .unwrap_or(ItemQuality::Normal);

        // ── Emit cross-domain events ──────────────────────────────────────────

        // Signal to other domains (stats, achievements, etc.) that a product
        // was just collected from an animal.
        product_writer.send(AnimalProductEvent {
            animal_kind: animal.kind,
            product_id: product_id.to_string(),
        });

        // Add the item to the player's inventory.
        pickup_writer.send(ItemPickupEvent {
            item_id: product_id.to_string(),
            quantity: 1,
        });

        // Audio cue.
        sfx_writer.send(PlaySfxEvent {
            sfx_id: "item_pickup".to_string(),
        });

        // Quality toast — shown for every quality level so players learn the
        // system, but worded differently for Normal vs. premium qualities.
        let quality_label = match quality {
            ItemQuality::Normal => format!("Collected {}!", product_display),
            ItemQuality::Silver => format!("Silver {} collected!", product_display),
            ItemQuality::Gold => format!("Gold {} collected!", product_display),
            ItemQuality::Iridium => format!("Iridium {}! Incredible!", product_display),
        };
        toast_writer.send(ToastEvent {
            message: quality_label,
            duration_secs: 2.5,
        });

        // ── Consume the pending quality component ─────────────────────────────
        commands.entity(entity).remove::<PendingProductQuality>();

        // ── Clear the product-ready flag ──────────────────────────────────────
        animal.product_ready = false;

        // ── Floating text above the animal ────────────────────────────────────
        // The color reflects quality so players get instant visual feedback.
        let floating_label = match animal.kind {
            AnimalKind::Chicken => "Got Egg!",
            AnimalKind::Cow => "Got Milk!",
            AnimalKind::Sheep => "Got Wool!",
            _ => "Collected!",
        };

        let text_color = match quality {
            ItemQuality::Normal => Color::srgb(0.9, 0.8, 0.2),
            ItemQuality::Silver => Color::srgb(0.75, 0.85, 1.0),
            ItemQuality::Gold => Color::srgb(1.0, 0.82, 0.1),
            ItemQuality::Iridium => Color::srgb(0.8, 0.4, 1.0),
        };

        spawn_floating_text(
            &mut commands,
            animal_transform.translation + Vec3::new(0.0, 14.0, 2.0),
            floating_label,
            text_color,
        );

        // Only collect from one animal per key press — stop after the first
        // in-range animal with a ready product.
        break;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Product-ready indicator sprites
//
// When product_ready is true, a small glowing indicator is rendered above the
// animal so the player knows to come collect it.  We manage these as separate
// entities tracked via the ProductReadyIndicator component.
//
// Each frame:
//   - Move existing indicators to follow their owner animal.
//   - Despawn indicators whose owner no longer has product_ready == true.
//   - Spawn new indicators for newly-ready animals.
// ─────────────────────────────────────────────────────────────────────────────

pub fn update_product_indicators(
    mut commands: Commands,
    animal_query: Query<(Entity, &Animal, &Transform)>,
    mut indicator_query: Query<(Entity, &mut Transform, &ProductReadyIndicator), Without<Animal>>,
) {
    // Build the set of animal entities that currently have a product ready.
    let ready_entities: std::collections::HashSet<Entity> = animal_query
        .iter()
        .filter(|(_, a, _)| a.product_ready)
        .map(|(e, _, _)| e)
        .collect();

    // Update positions of existing indicators; despawn stale ones.
    let mut indicators_present: std::collections::HashSet<Entity> =
        std::collections::HashSet::new();

    for (ind_entity, mut ind_transform, indicator) in indicator_query.iter_mut() {
        if ready_entities.contains(&indicator.owner) {
            // Keep the indicator and track its owner so we don't re-spawn.
            if let Ok((_, _, animal_transform)) = animal_query.get(indicator.owner) {
                ind_transform.translation =
                    animal_transform.translation + Vec3::new(0.0, 12.0, 2.0);
            }
            indicators_present.insert(indicator.owner);
        } else {
            // Owner no longer has a ready product — despawn the indicator.
            commands.entity(ind_entity).despawn_recursive();
        }
    }

    // Spawn new indicators for animals that just became ready.
    for (entity, animal, transform) in animal_query.iter() {
        if !animal.product_ready || indicators_present.contains(&entity) {
            continue;
        }

        // Color hint: matches the floating text quality colors loosely,
        // but here we show what *kind* of product is available (not quality,
        // since quality is only revealed on collection).
        let color = match animal.kind {
            AnimalKind::Chicken => Color::srgb(1.0, 1.0, 0.5), // yellow — egg
            AnimalKind::Cow => Color::srgb(1.0, 1.0, 1.0),     // white — milk
            AnimalKind::Sheep => Color::srgb(0.8, 0.8, 1.0),   // pale blue — wool
            _ => Color::srgb(1.0, 1.0, 1.0),
        };

        commands.spawn((
            ProductReadyIndicator { owner: entity },
            Sprite {
                color,
                custom_size: Some(Vec2::new(6.0, 6.0)),
                ..default()
            },
            Transform::from_translation(
                transform.translation + Vec3::new(0.0, 12.0, 2.0),
            ),
            GlobalTransform::default(),
            Visibility::default(),
        ));
    }
}
