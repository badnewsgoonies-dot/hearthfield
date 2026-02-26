use bevy::prelude::*;
use crate::shared::*;
use super::FloatingFeedback;

// ─────────────────────────────────────────────────────────────────────────────
// Floating feedback text (hearts, "Yum!", etc.)
// ─────────────────────────────────────────────────────────────────────────────

/// Convenience function called from other submodules to spawn a floating text
/// entity that drifts upward and fades out.
pub fn spawn_floating_text(
    commands: &mut Commands,
    position: Vec3,
    text: &str,
    color: Color,
) {
    commands.spawn((
        FloatingFeedback {
            lifetime: Timer::from_seconds(1.2, TimerMode::Once),
            velocity: Vec2::new(0.0, 18.0),
        },
        Text2d::new(text.to_string()),
        TextFont {
            font_size: 8.0,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
    ));
}

// ─────────────────────────────────────────────────────────────────────────────
// System: update / despawn floating feedback
// ─────────────────────────────────────────────────────────────────────────────

pub fn update_floating_feedback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut FloatingFeedback, &mut TextColor)>,
) {
    for (entity, mut transform, mut feedback, mut color) in query.iter_mut() {
        feedback.lifetime.tick(time.delta());

        // Drift upward.
        let dt = time.delta_secs();
        transform.translation.x += feedback.velocity.x * dt;
        transform.translation.y += feedback.velocity.y * dt;

        // Fade out by adjusting alpha based on remaining time fraction.
        let fraction_remaining =
            1.0 - feedback.lifetime.elapsed_secs() / feedback.lifetime.duration().as_secs_f32();
        let current = color.0;
        color.0 = current.with_alpha(fraction_remaining.max(0.0));

        if feedback.lifetime.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: sync ECS animal components → AnimalState resource
//
// Other domains (save, UI) read AnimalState. We keep it up to date by
// rebuilding the Vec<Animal> every frame from the ECS query.  This is cheap
// since the number of animals is small (< 20 typically).
// ─────────────────────────────────────────────────────────────────────────────

pub fn sync_animal_state_resource(
    mut animal_state: ResMut<AnimalState>,
    animal_query: Query<&Animal>,
) {
    animal_state.animals = animal_query.iter().cloned().collect();
}
