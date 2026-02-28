use bevy::prelude::*;
use crate::shared::*;
use super::UnfedDays;

// ─────────────────────────────────────────────────────────────────────────────
// Day-end processing
//
// Listens for DayEndEvent and processes every animal entity:
//
//   1. Track consecutive unfed days (UnfedDays component).
//   2. Adjust happiness:
//        - Fed today:   +5  (capped at u8::MAX = 255)
//        - Not fed:     -12 (floors at 0)
//        - Petted today: +5 on top of the above
//   3. Reset daily flags (fed_today, petted_today).
//   4. Age babies → adults after 5 days.
//   5. Generate product_ready (+ PendingProductQuality) for adult animals
//      that were fed and are not blocked by a starvation streak.
//
// Happiness quality thresholds (deterministic — no RNG):
//   happiness >= 230 → Iridium
//   happiness >= 200 → Gold
//   happiness >= 128 → Silver
//   happiness  < 128 → Normal
//
// Starvation block: if an animal goes 3+ consecutive days without food it
// will not produce anything until the day it is fed again (that very day it
// is fed, the block is lifted and production resumes).
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks how many days have passed since a sheep last produced wool.
/// Sheep produce wool every 3 days.
#[derive(Component, Debug, Clone)]
pub struct SheepWoolCooldown {
    pub days_since_last_wool: u8,
}

/// Per-animal component recording what quality product will be collected.
/// Written at day-end; read and removed by the collection handler in products.rs.
#[derive(Component, Debug, Clone, Default)]
pub struct PendingProductQuality {
    pub quality: ItemQuality,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: derive quality tier from happiness (deterministic, no RNG)
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the `ItemQuality` corresponding to an animal's happiness value.
///
/// Thresholds:
/// - >= 230 → Iridium  (very happy, maximally cared for)
/// - >= 200 → Gold     (happy)
/// - >= 128 → Silver   (content)
/// -  < 128 → Normal   (neglected but alive)
pub fn quality_from_happiness(happiness: u8) -> ItemQuality {
    if happiness >= 230 {
        ItemQuality::Iridium
    } else if happiness >= 200 {
        ItemQuality::Gold
    } else if happiness >= 128 {
        ItemQuality::Silver
    } else {
        ItemQuality::Normal
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main day-end system
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_day_end_for_animals(
    mut commands: Commands,
    mut day_end_events: EventReader<DayEndEvent>,
    mut animal_query: Query<(
        Entity,
        &mut Animal,
        Option<&SheepWoolCooldown>,
        Option<&mut UnfedDays>,
    )>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for _event in day_end_events.read() {
        for (entity, mut animal, wool_cd, unfed_days_opt) in animal_query.iter_mut() {
            // ── 1. Track consecutive unfed days ──────────────────────────────
            //
            // Snapshot the previous count before any resets so we can
            // correctly determine whether the production block should fire.
            let prev_unfed_count: u8 = unfed_days_opt
                .as_ref()
                .map(|ud| ud.count)
                .unwrap_or(0);

            let new_unfed_count: u8 = if animal.fed_today {
                // Animal was fed — reset the starvation streak.
                0
            } else {
                // Not fed — increment, capping at u8::MAX to avoid wrap.
                prev_unfed_count.saturating_add(1)
            };

            // Persist the updated count back into the ECS component.
            if let Some(mut ud) = unfed_days_opt {
                ud.count = new_unfed_count;
            } else {
                commands
                    .entity(entity)
                    .insert(UnfedDays { count: new_unfed_count });
            }

            // ── 2. Happiness adjustments ─────────────────────────────────────
            //
            // All adjustments use saturating arithmetic so happiness stays
            // in [0, 255] — the valid range of a u8.
            if animal.fed_today {
                // Fed today: +5 happiness.
                animal.happiness = animal.happiness.saturating_add(5);
            } else {
                // Not fed: -12 happiness (midpoint of the 10-15 range).
                animal.happiness = animal.happiness.saturating_sub(12);
            }

            if animal.petted_today {
                // Petting gives an additional +5.
                animal.happiness = animal.happiness.saturating_add(5);
            }

            // Warn via toast when an animal's happiness drops into danger zones.
            if !animal.fed_today {
                if new_unfed_count == 3 {
                    toast_writer.send(ToastEvent {
                        message: format!(
                            "{} hasn't eaten in 3 days and stopped producing!",
                            animal.name
                        ),
                        duration_secs: 4.0,
                    });
                } else if new_unfed_count == 1 && animal.happiness < 50 {
                    toast_writer.send(ToastEvent {
                        message: format!("{} is hungry and unhappy...", animal.name),
                        duration_secs: 3.0,
                    });
                }
            }

            // ── 3. Reset daily flags ─────────────────────────────────────────
            animal.fed_today = false;
            animal.petted_today = false;

            // ── 4. Aging: baby → adult after 5 days ─────────────────────────
            animal.days_old = animal.days_old.saturating_add(1);
            if animal.age == AnimalAge::Baby && animal.days_old >= 5 {
                animal.age = AnimalAge::Adult;
                info!(
                    "Animal '{}' ({:?}) has grown into an adult!",
                    animal.name, animal.kind
                );
                toast_writer.send(ToastEvent {
                    message: format!(
                        "{} the {:?} has grown into an adult!",
                        animal.name, animal.kind
                    ),
                    duration_secs: 4.0,
                });
            }

            // ── 5. Product generation ─────────────────────────────────────────
            //
            // Conditions for generating a product:
            //   a) Animal is an Adult (babies never produce).
            //   b) Animal was fed today — `new_unfed_count == 0` captures this
            //      because we only reset the count when fed_today was true.
            //   c) Not blocked by a starvation streak. A streak of 3+ unfed days
            //      blocks production; feeding the animal resets the streak
            //      (new_unfed_count == 0) so the block is lifted the same day
            //      the animal is finally fed.
            //   d) Happiness > 0 (a completely miserable animal refuses to
            //      produce even if technically fed).

            let fed_today_this_cycle = new_unfed_count == 0;
            // Block fires when the animal was not fed today AND had already been
            // starved for 3+ consecutive days before today.
            let production_blocked = !fed_today_this_cycle && prev_unfed_count >= 3;

            if animal.age == AnimalAge::Adult
                && fed_today_this_cycle
                && !production_blocked
                && animal.happiness > 0
            {
                // Quality is based on post-adjustment happiness: the animal's
                // happiness after today's feeding/petting bonuses are applied.
                let quality = quality_from_happiness(animal.happiness);

                match animal.kind {
                    AnimalKind::Chicken => {
                        // Chickens produce an egg every day.
                        animal.product_ready = true;
                        commands
                            .entity(entity)
                            .insert(PendingProductQuality { quality });
                    }
                    AnimalKind::Cow => {
                        // Cows produce milk every day.
                        animal.product_ready = true;
                        commands
                            .entity(entity)
                            .insert(PendingProductQuality { quality });
                    }
                    AnimalKind::Sheep => {
                        // Sheep produce wool every 3 days.
                        let days = wool_cd
                            .map(|c| c.days_since_last_wool)
                            .unwrap_or(3); // default 3 → produce immediately first time
                        if days >= 3 {
                            animal.product_ready = true;
                            commands
                                .entity(entity)
                                .insert(PendingProductQuality { quality });
                            // Reset the wool cooldown.
                            commands.entity(entity).insert(SheepWoolCooldown {
                                days_since_last_wool: 0,
                            });
                        } else {
                            // Advance the cooldown counter by one day.
                            commands.entity(entity).insert(SheepWoolCooldown {
                                days_since_last_wool: days + 1,
                            });
                        }
                    }
                    AnimalKind::Cat | AnimalKind::Dog => {
                        // Pets are companions only — no harvestable products.
                    }
                }
            }
        }

        // ── 6. AnimalState resource sync ─────────────────────────────────────
        // sync_animal_state_resource (rendering.rs) rebuilds the Vec<Animal>
        // from ECS every frame, so no manual sync is needed here.
    }
}
