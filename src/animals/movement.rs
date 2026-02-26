use bevy::prelude::*;
use rand::Rng;
use crate::shared::*;
use super::WanderAi;

// ─────────────────────────────────────────────────────────────────────────────
// Wander AI system
// Animals pick a random point inside their pen every 2-4 seconds, walk toward
// it, then idle briefly before picking the next point.
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_animal_wander(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut WanderAi, &Animal)>,
) {
    let mut rng = rand::thread_rng();

    for (mut transform, mut wander, _animal) in query.iter_mut() {
        // Advance the timer.
        wander.timer.tick(time.delta());

        if let Some(target) = wander.target {
            // Move toward target.
            let current = transform.translation.truncate();
            let delta = target - current;
            let dist = delta.length();

            if dist < 1.5 {
                // Arrived — clear target, start idle timer.
                wander.target = None;
                let idle_secs = rng.gen_range(1.5_f32..=3.5_f32);
                wander.timer = Timer::from_seconds(idle_secs, TimerMode::Once);
            } else {
                // Step toward the target, capped by speed × dt.
                let step = wander.speed * time.delta_secs();
                let movement = delta.normalize() * step.min(dist);
                transform.translation.x += movement.x;
                transform.translation.y += movement.y;

                // Clamp to pen bounds.
                transform.translation.x = transform
                    .translation
                    .x
                    .clamp(wander.pen_min.x, wander.pen_max.x);
                transform.translation.y = transform
                    .translation
                    .y
                    .clamp(wander.pen_min.y, wander.pen_max.y);

                // Flip sprite horizontally based on direction.
                // (positive x → face right; negative x → face left)
                if movement.x.abs() > 0.1 {
                    // We express flip via scale sign.
                    transform.scale.x = if movement.x > 0.0 { 1.0 } else { -1.0 };
                }
            }
        } else if wander.timer.just_finished() {
            // Pick a new wander target inside the pen.
            let tx = rng.gen_range(wander.pen_min.x..=wander.pen_max.x);
            let ty = rng.gen_range(wander.pen_min.y..=wander.pen_max.y);
            wander.target = Some(Vec2::new(tx, ty));

            // Walk timer: how long until we pick the NEXT point after arriving.
            // This is reset again when we arrive, so this just acts as a
            // safety fallback.
            let next_secs = rng.gen_range(2.0_f32..=4.0_f32);
            wander.timer = Timer::from_seconds(next_secs, TimerMode::Once);
        }
    }
}
