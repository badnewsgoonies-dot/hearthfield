use bevy::prelude::*;
use crate::shared::*;
use super::spawn_floating_text;

// ─────────────────────────────────────────────────────────────────────────────
// Petting system
//
// When the player presses Space near an animal we detect it via proximity:
// once per frame we check the player entity's position against each animal's
// position.  Interaction range: 32 px (≈2 tiles).
//
// We only trigger on the frame the key is first pressed (just_pressed) so a
// held key does not spam petting.
//
// Priority: if an animal in range has product_ready == true, product
// collection (handle_product_collection in products.rs) takes priority.
// This system skips animals that have a product waiting so the player isn't
// accidentally petting while collecting.
// ─────────────────────────────────────────────────────────────────────────────

const INTERACT_RANGE: f32 = 32.0;

pub fn handle_animal_interact(
    mut commands: Commands,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    player_query: Query<&Transform, With<Player>>,
    mut animal_query: Query<(Entity, &mut Animal, &Transform)>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    if !player_input.tool_use {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    // First, check if any in-range animal has a product ready. If so, skip
    // petting entirely — handle_product_collection will process that press.
    let any_product_ready = animal_query.iter().any(|(_, animal, animal_transform)| {
        animal.product_ready
            && player_pos.distance(animal_transform.translation.truncate()) <= INTERACT_RANGE
    });

    if any_product_ready {
        // Let the product collection system handle this Space press.
        return;
    }

    for (_entity, mut animal, animal_transform) in animal_query.iter_mut() {
        let animal_pos = animal_transform.translation.truncate();
        let dist = player_pos.distance(animal_pos);

        if dist > INTERACT_RANGE {
            continue;
        }

        // Pet the animal.
        if !animal.petted_today {
            animal.petted_today = true;
            // Immediate happiness bonus (capped at u8::MAX by saturating_add).
            animal.happiness = animal.happiness.saturating_add(5);

            let pet_text = match animal.kind {
                AnimalKind::Chicken => "Bawk!",
                AnimalKind::Cow => "Moo~",
                AnimalKind::Sheep => "Baa!",
                AnimalKind::Cat => "<3",
                AnimalKind::Dog => "Woof!",
            };

            // Heart feedback above animal.
            spawn_floating_text(
                &mut commands,
                animal_transform.translation + Vec3::new(0.0, 14.0, 2.0),
                pet_text,
                Color::srgb(1.0, 0.4, 0.7),
            );

            sfx_writer.send(PlaySfxEvent {
                sfx_id: "animal_pet".to_string(),
            });
        } else {
            // Already petted today — give small feedback so player knows.
            spawn_floating_text(
                &mut commands,
                animal_transform.translation + Vec3::new(0.0, 14.0, 2.0),
                "Already happy!",
                Color::srgb(0.8, 0.8, 0.4),
            );
        }

        // Only interact with the closest animal (first in range).
        break;
    }
}
