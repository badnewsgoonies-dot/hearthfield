use super::WanderAi;
use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ─────────────────────────────────────────────────────────────────────────────
// Wander AI system
// Animals pick a random point inside their pen every 2-4 seconds, walk toward
// it, then idle briefly before picking the next point.
// ─────────────────────────────────────────────────────────────────────────────

fn stop_radius_for(kind: AnimalKind) -> f32 {
    match kind {
        AnimalKind::Chicken => 0.75,
        AnimalKind::Cow => 2.5,
        AnimalKind::Rabbit => 0.6,
        AnimalKind::Dog => 1.25,
        _ => 1.5,
    }
}

fn idle_secs_for(kind: AnimalKind, rng: &mut impl Rng) -> f32 {
    match kind {
        AnimalKind::Chicken => rng.gen_range(0.35_f32..=0.9_f32),
        AnimalKind::Cow => rng.gen_range(2.8_f32..=5.0_f32),
        AnimalKind::Rabbit => rng.gen_range(0.15_f32..=0.55_f32),
        AnimalKind::Dog => rng.gen_range(0.4_f32..=1.0_f32),
        _ => rng.gen_range(1.5_f32..=3.5_f32),
    }
}

fn retarget_secs_for(kind: AnimalKind, rng: &mut impl Rng) -> f32 {
    match kind {
        AnimalKind::Chicken => rng.gen_range(0.6_f32..=1.4_f32),
        AnimalKind::Cow => rng.gen_range(3.5_f32..=6.0_f32),
        AnimalKind::Rabbit => rng.gen_range(0.25_f32..=0.8_f32),
        AnimalKind::Dog => rng.gen_range(0.5_f32..=1.2_f32),
        _ => rng.gen_range(2.0_f32..=4.0_f32),
    }
}

pub fn handle_animal_wander(
    time: Res<Time>,
    mut query: Query<(
        &mut LogicalPosition,
        &mut Transform,
        &mut WanderAi,
        &Animal,
        Option<&mut Facing>,
    )>,
) {
    let mut rng = rand::thread_rng();

    for (mut logical_pos, mut transform, mut wander, animal, facing_opt) in query.iter_mut() {
        // Advance the timer.
        wander.timer.tick(time.delta());

        if let Some(target) = wander.target {
            // Move toward target.
            let current = logical_pos.0;
            let delta = target - current;
            let dist = delta.length();
            let stop_radius = stop_radius_for(animal.kind);

            if dist < stop_radius {
                // Arrived — clear target, start idle timer.
                wander.target = None;
                let idle_secs = idle_secs_for(animal.kind, &mut rng);
                wander.timer = Timer::from_seconds(idle_secs, TimerMode::Once);
            } else {
                // Step toward the target, capped by speed × dt.
                let step = wander.speed * time.delta_secs();
                let movement = delta.normalize() * step.min(dist);
                logical_pos.0.x += movement.x;
                logical_pos.0.y += movement.y;

                // Clamp to pen bounds.
                logical_pos.0.x = logical_pos.0.x.clamp(wander.pen_min.x, wander.pen_max.x);
                logical_pos.0.y = logical_pos.0.y.clamp(wander.pen_min.y, wander.pen_max.y);

                // Update facing direction based on movement.
                if let Some(mut facing) = facing_opt {
                    if movement.x.abs() > movement.y.abs() {
                        *facing = if movement.x > 0.0 {
                            Facing::Right
                        } else {
                            Facing::Left
                        };
                    } else if movement.y.abs() > 0.1 {
                        *facing = if movement.y > 0.0 {
                            Facing::Up
                        } else {
                            Facing::Down
                        };
                    }
                } else {
                    // Fallback for animals without Facing (should not happen now, but just in case)
                    if movement.x.abs() > 0.1 {
                        transform.scale.x = if movement.x > 0.0 { 1.0 } else { -1.0 };
                    }
                }
            }
        } else if wander.timer.just_finished() {
            // Pick a new wander target inside the pen.
            let tx = rng.gen_range(wander.pen_min.x..=wander.pen_max.x);
            let ty = rng.gen_range(wander.pen_min.y..=wander.pen_max.y);
            wander.target = Some(Vec2::new(tx, ty));

            let next_secs = retarget_secs_for(animal.kind, &mut rng);
            wander.timer = Timer::from_seconds(next_secs, TimerMode::Once);
        }
    }
}
