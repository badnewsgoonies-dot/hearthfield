use bevy::prelude::*;
use crate::shared::*;
use super::{ProductReadyIndicator, FloatingFeedback, spawn_floating_text};

// ─────────────────────────────────────────────────────────────────────────────
// Product collection
//
// When player presses Space near an animal that has product_ready == true,
// we emit AnimalProductEvent and ItemPickupEvent and clear product_ready.
// ─────────────────────────────────────────────────────────────────────────────

const INTERACT_RANGE: f32 = 32.0;

pub fn handle_product_collection(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut animal_query: Query<(Entity, &mut Animal, &Transform)>,
    mut product_writer: EventWriter<AnimalProductEvent>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (_entity, mut animal, animal_transform) in animal_query.iter_mut() {
        if !animal.product_ready {
            continue;
        }

        let animal_pos = animal_transform.translation.truncate();
        if player_pos.distance(animal_pos) > INTERACT_RANGE {
            continue;
        }

        // Determine the product item id.
        let product_id = match animal.kind {
            AnimalKind::Chicken => "egg",
            AnimalKind::Cow => "milk",
            AnimalKind::Sheep => "wool",
            _ => continue, // pets don't produce
        };

        // Emit cross-domain events.
        product_writer.send(AnimalProductEvent {
            animal_kind: animal.kind,
            product_id: product_id.to_string(),
        });

        pickup_writer.send(ItemPickupEvent {
            item_id: product_id.to_string(),
            quantity: 1,
        });

        // Clear the flag.
        animal.product_ready = false;

        // Visual and audio feedback.
        let label = match animal.kind {
            AnimalKind::Chicken => "Got Egg!",
            AnimalKind::Cow => "Got Milk!",
            AnimalKind::Sheep => "Got Wool!",
            _ => "Collected!",
        };

        spawn_floating_text(
            &mut commands,
            animal_transform.translation + Vec3::new(0.0, 14.0, 2.0),
            label,
            Color::srgb(0.9, 0.8, 0.2),
        );

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "item_pickup".to_string(),
        });

        // Only collect from one animal per key press.
        break;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Product-ready indicator sprites
//
// When product_ready is true, a small glowing indicator is rendered above the
// animal.  We manage these as separate child-like entities tracked via the
// ProductReadyIndicator component.
// ─────────────────────────────────────────────────────────────────────────────

pub fn update_product_indicators(
    mut commands: Commands,
    animal_query: Query<(Entity, &Animal, &Transform)>,
    mut indicator_query: Query<(Entity, &mut Transform, &ProductReadyIndicator), Without<Animal>>,
) {
    // Build a set of entities with product_ready == true.
    let ready_entities: std::collections::HashSet<Entity> = animal_query
        .iter()
        .filter(|(_, a, _)| a.product_ready)
        .map(|(e, _, _)| e)
        .collect();

    // Update or remove existing indicators.
    let mut handled: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for (ind_entity, mut ind_transform, indicator) in indicator_query.iter_mut() {
        if ready_entities.contains(&indicator.owner) {
            // Move indicator to sit above the owner animal.
            if let Ok((_, _, animal_transform)) = animal_query.get(indicator.owner) {
                ind_transform.translation = animal_transform.translation + Vec3::new(0.0, 12.0, 2.0);
            }
            handled.insert(indicator.owner);
        } else {
            // Animal no longer has product — despawn indicator.
            commands.entity(ind_entity).despawn_recursive();
        }
    }

    // Spawn new indicators for newly-ready animals.
    for (entity, animal, transform) in animal_query.iter() {
        if animal.product_ready && !handled.contains(&entity) {
            let color = match animal.kind {
                AnimalKind::Chicken => Color::srgb(1.0, 1.0, 0.5),
                AnimalKind::Cow => Color::srgb(1.0, 1.0, 1.0),
                AnimalKind::Sheep => Color::srgb(0.8, 0.8, 1.0),
                _ => Color::srgb(1.0, 1.0, 1.0),
            };

            commands.spawn((
                ProductReadyIndicator { owner: entity },
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(6.0, 6.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation + Vec3::new(0.0, 12.0, 2.0)),
                GlobalTransform::default(),
                Visibility::default(),
            ));
        }
    }
}
