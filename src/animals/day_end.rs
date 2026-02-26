use bevy::prelude::*;
use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// Day-end processing
//
// Listens for DayEndEvent and:
//   - Adjusts happiness based on feeding / petting
//   - Resets daily flags
//   - Ages babies → adults (7 days)
//   - Sets product_ready for adult, happy animals
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks how many days have passed since a sheep last produced wool.
/// Sheep produce every 3 days (stored per-entity via this component).
#[derive(Component, Debug, Clone)]
pub struct SheepWoolCooldown {
    pub days_since_last_wool: u8,
}

pub fn handle_day_end_for_animals(
    mut commands: Commands,
    mut day_end_events: EventReader<DayEndEvent>,
    mut animal_query: Query<(Entity, &mut Animal, Option<&SheepWoolCooldown>)>,
    _animal_state: Res<AnimalState>,
) {
    for _event in day_end_events.read() {
        for (entity, mut animal, wool_cd) in animal_query.iter_mut() {
            // ── 1. Happiness adjustments ─────────────────────────────────────
            if animal.fed_today {
                animal.happiness = animal.happiness.saturating_add(3);
            } else {
                animal.happiness = animal.happiness.saturating_sub(10);
            }

            if animal.petted_today {
                animal.happiness = animal.happiness.saturating_add(2);
            }

            // Clamp to 0-255 (u8 already handles underflow via saturating ops).
            // No need to clamp, u8 naturally wraps — saturating_add/sub is safe.

            // ── 2. Reset daily flags ─────────────────────────────────────────
            animal.fed_today = false;
            animal.petted_today = false;

            // ── 3. Aging ─────────────────────────────────────────────────────
            animal.days_old += 1;
            if animal.age == AnimalAge::Baby && animal.days_old >= 7 {
                animal.age = AnimalAge::Adult;
                info!(
                    "Animal '{}' ({:?}) has grown into an adult!",
                    animal.name, animal.kind
                );
            }

            // ── 4. Product generation ────────────────────────────────────────
            if animal.age == AnimalAge::Adult && animal.happiness > 100 {
                match animal.kind {
                    AnimalKind::Chicken => {
                        animal.product_ready = true;
                    }
                    AnimalKind::Cow => {
                        animal.product_ready = true;
                    }
                    AnimalKind::Sheep => {
                        // Sheep produce wool every 3 days.
                        let days = wool_cd.map(|c| c.days_since_last_wool).unwrap_or(3);
                        if days >= 3 {
                            animal.product_ready = true;
                            commands.entity(entity).insert(SheepWoolCooldown {
                                days_since_last_wool: 0,
                            });
                        } else {
                            // Increment the cooldown counter.
                            commands.entity(entity).insert(SheepWoolCooldown {
                                days_since_last_wool: days + 1,
                            });
                        }
                    }
                    AnimalKind::Cat | AnimalKind::Dog => {
                        // Pets don't produce items.
                    }
                }
            }
        }

        // ── 5. Mirror entity state into AnimalState resource ─────────────────
        // (sync_animal_state_resource does a full rebuild each frame, so we
        //  don't need a duplicate update here; the resource is kept current by
        //  the rendering/sync system.)
    }
}
