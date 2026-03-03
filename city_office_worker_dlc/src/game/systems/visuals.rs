use bevy::prelude::*;

use crate::game::components::{InboxAvatar, OfficeWorker, WorkerAvatar};
use crate::game::resources::{DayClock, InboxState, OfficeRules};

pub fn update_visuals(
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    inbox: Res<InboxState>,
    worker_query: Query<&OfficeWorker>,
    mut worker_avatar_query: Query<&mut Sprite, With<WorkerAvatar>>,
    mut inbox_avatar_query: Query<(&mut Sprite, &mut Transform), With<InboxAvatar>>,
    mut clear_color: ResMut<ClearColor>,
) {
    if let Ok(worker) = worker_query.get_single() {
        let energy_ratio = (worker.energy as f32 / rules.max_energy as f32).clamp(0.0, 1.0);
        if let Ok(mut sprite) = worker_avatar_query.get_single_mut() {
            sprite.color = Color::srgb(1.0 - energy_ratio, 0.2 + (0.7 * energy_ratio), 0.25);
        }
    }

    if let Ok((mut sprite, mut transform)) = inbox_avatar_query.get_single_mut() {
        let inbox_ratio = if rules.starting_inbox_items == 0 {
            0.0
        } else {
            (inbox.remaining_items as f32 / rules.starting_inbox_items as f32).clamp(0.0, 1.0)
        };
        let scale = 0.35 + (inbox_ratio * 1.15);
        transform.scale = Vec3::splat(scale);
        sprite.color = Color::srgb(0.4 + (0.5 * inbox_ratio), 0.35 + (0.45 * inbox_ratio), 0.2);
    }

    let shift_duration = (rules.day_end_minute - rules.day_start_minute).max(1);
    let elapsed = clock.current_minute.saturating_sub(rules.day_start_minute);
    let progress = (elapsed as f32 / shift_duration as f32).clamp(0.0, 1.0);

    let tone = 0.2 - (0.11 * progress);
    clear_color.0 = Color::srgb(tone, tone * 0.95, tone * 1.1);
}
