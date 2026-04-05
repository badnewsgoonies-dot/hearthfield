use super::spawn_floating_text;
use crate::shared::*;
use bevy::prelude::*;

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

/// Map animal happiness (0–255) to an icon index in icons_happiness.png (row 1, saturated).
/// Col 0 = lowest mood, col 5 = highest mood.
fn happiness_icon_index(happiness: u8) -> usize {
    let col = match happiness {
        0..=49 => 0,
        50..=99 => 1,
        100..=149 => 2,
        150..=199 => 3,
        200..=229 => 4,
        230..=u8::MAX => 5,
    };
    6 + col // row 1 (saturated variants) = offset by 6 cols
}

pub fn handle_animal_interact(
    mut commands: Commands,
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    player_query: Query<&LogicalPosition, With<Player>>,
    mut animal_query: Query<(Entity, &mut Animal, &LogicalPosition)>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    sprite_data: Res<super::AnimalSpriteData>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    if !player_input.tool_use {
        return;
    }

    let Ok(player_lp) = player_query.get_single() else {
        return;
    };

    let player_pos = player_lp.0;

    // First, check if any in-range animal has a product ready. If so, skip
    // petting entirely — handle_product_collection will process that press.
    let any_product_ready = animal_query.iter().any(|(_, animal, animal_lp)| {
        animal.product_ready && player_pos.distance(animal_lp.0) <= INTERACT_RANGE
    });

    if any_product_ready {
        // Let the product collection system handle this Space press.
        return;
    }

    for (_entity, mut animal, animal_lp) in animal_query.iter_mut() {
        let animal_pos = animal_lp.0;
        let dist = player_pos.distance(animal_pos);

        if dist > INTERACT_RANGE {
            continue;
        }

        // Pet the animal.
        if !animal.petted_today {
            let prior_happiness = animal.happiness.get();
            animal.petted_today = true;
            // Lower-mood animals get a larger first pet boost.
            let happiness_bump = match prior_happiness {
                0..=79 => 12,
                80..=159 => 8,
                160..=219 => 4,
                220..=u8::MAX => 1,
            };
            let happiness = prior_happiness.saturating_add(happiness_bump);
            animal.happiness = Happiness::new(happiness);

            let pet_text = match (animal.kind, prior_happiness) {
                (AnimalKind::Chicken, 0..=79) => "Bawk?",
                (AnimalKind::Chicken, 80..=159) => "Bawk!",
                (AnimalKind::Chicken, 160..=u8::MAX) => "Bawk bawk!",
                (AnimalKind::Cow, 0..=79) => "Mrrrmoo...",
                (AnimalKind::Cow, 80..=159) => "Moo~",
                (AnimalKind::Cow, 160..=u8::MAX) => "Mooooh!",
                (AnimalKind::Sheep, 0..=79) => "Baa..?",
                (AnimalKind::Sheep, 80..=159) => "Baa!",
                (AnimalKind::Sheep, 160..=u8::MAX) => "Baa-aa!",
                (AnimalKind::Goat, 0..=79) => "Meh...",
                (AnimalKind::Goat, 80..=159) => "Meh!",
                (AnimalKind::Goat, 160..=u8::MAX) => "Meeeeh!",
                (AnimalKind::Duck, 0..=79) => "quack...",
                (AnimalKind::Duck, 80..=159) => "Quack!",
                (AnimalKind::Duck, 160..=u8::MAX) => "Quack-quack!",
                (AnimalKind::Rabbit, 0..=79) => "sniff...",
                (AnimalKind::Rabbit, 80..=159) => "~squeak~",
                (AnimalKind::Rabbit, 160..=u8::MAX) => "binky!",
                (AnimalKind::Pig, 0..=79) => "snorf...",
                (AnimalKind::Pig, 80..=159) => "Oink!",
                (AnimalKind::Pig, 160..=u8::MAX) => "Oink oink!",
                (AnimalKind::Horse, 0..=79) => "hnnh...",
                (AnimalKind::Horse, 80..=159) => "Neigh!",
                (AnimalKind::Horse, 160..=u8::MAX) => "Neeeigh!",
                (AnimalKind::Cat, 0..=79) => "mrrp...",
                (AnimalKind::Cat, 80..=159) => "prrr...",
                (AnimalKind::Cat, 160..=u8::MAX) => "<3",
                (AnimalKind::Dog, 0..=79) => "ruff...",
                (AnimalKind::Dog, 80..=159) => "Woof!",
                (AnimalKind::Dog, 160..=u8::MAX) => "Wag wag!",
            };

            // Heart feedback above animal.
            spawn_floating_text(
                &mut commands,
                animal_pos.extend(Z_EFFECTS) + Vec3::new(0.0, 14.0, 0.0),
                pet_text,
                Color::srgb(1.0, 0.52, 0.72),
            );

            // Floating mood icon from icons_happiness.png above the text.
            if sprite_data.loaded {
                let icon_idx = happiness_icon_index(happiness);
                let mut mood_sprite = Sprite::from_atlas_image(
                    sprite_data.happiness_image.clone(),
                    TextureAtlas {
                        layout: sprite_data.happiness_layout.clone(),
                        index: icon_idx,
                    },
                );
                mood_sprite.custom_size = Some(Vec2::splat(12.0));
                commands.spawn((
                    super::FloatingFeedback {
                        lifetime: Timer::from_seconds(1.2, TimerMode::Once),
                        velocity: Vec2::new(0.0, 8.0),
                    },
                    mood_sprite,
                    Transform::from_xyz(
                        animal_pos.x + 10.0,
                        animal_pos.y + 22.0,
                        Z_EFFECTS + 1.0,
                    ),
                    Visibility::default(),
                ));
            }

            sfx_writer.send(PlaySfxEvent {
                sfx_id: "animal_pet".to_string(),
            });
        } else {
            // Already petted today — give small feedback so player knows.
            let repeat_text = match (animal.kind, animal.happiness.get()) {
                (AnimalKind::Chicken, 0..=159) => "Another peck later?",
                (AnimalKind::Chicken, 160..=u8::MAX) => "This hen feels adored.",
                (AnimalKind::Cow, 0..=159) => "Come back for more moo time.",
                (AnimalKind::Cow, 160..=u8::MAX) => "This cow is fully content.",
                (AnimalKind::Sheep, 0..=159) => "A gentle pat later?",
                (AnimalKind::Sheep, 160..=u8::MAX) => "This sheep feels cozy.",
                (AnimalKind::Goat, 0..=159) => "Maybe another scritch soon?",
                (AnimalKind::Goat, 160..=u8::MAX) => "This goat is pleased.",
                (AnimalKind::Duck, 0..=159) => "Try another pat later.",
                (AnimalKind::Duck, 160..=u8::MAX) => "This duck feels cherished.",
                (AnimalKind::Rabbit, 0..=159) => "Soft pets again later?",
                (AnimalKind::Rabbit, 160..=u8::MAX) => "This bun is fully soothed.",
                (AnimalKind::Pig, 0..=159) => "Save a snuggle for later.",
                (AnimalKind::Pig, 160..=u8::MAX) => "This pig is happily spoiled.",
                (AnimalKind::Horse, 0..=159) => "Another brush later?",
                (AnimalKind::Horse, 160..=u8::MAX) => "This horse feels admired.",
                (AnimalKind::Cat, 0..=159) => "Maybe another pet later.",
                (AnimalKind::Cat, 160..=u8::MAX) => "This cat has been well loved.",
                (AnimalKind::Dog, 0..=159) => "More pats later, friend?",
                (AnimalKind::Dog, 160..=u8::MAX) => "This dog is already adored.",
            };
            spawn_floating_text(
                &mut commands,
                animal_pos.extend(Z_EFFECTS) + Vec3::new(0.0, 14.0, 0.0),
                repeat_text,
                Color::srgb(0.95, 0.82, 0.52),
            );
        }

        // Only interact with the closest animal (first in range).
        break;
    }
}
