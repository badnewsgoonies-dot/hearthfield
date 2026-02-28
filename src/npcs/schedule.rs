//! NPC schedule resolution and movement toward schedule waypoints.

use bevy::prelude::*;
use crate::shared::*;
use super::spawning::NpcMovement;

/// Given the current calendar state, return the active schedule entry for an NPC.
pub fn current_schedule_entry(calendar: &Calendar, schedule: &NpcSchedule) -> ScheduleEntry {
    let time = calendar.time_float();
    let is_weekend = matches!(
        calendar.day_of_week(),
        DayOfWeek::Saturday | DayOfWeek::Sunday
    );
    let is_raining = matches!(calendar.weather, Weather::Rainy | Weather::Stormy | Weather::Snowy);
    let is_festival = calendar.is_festival_day();

    // Priority: festival > rain > weekend > weekday
    let entries = if is_festival {
        if let Some(ref fest) = schedule.festival_override {
            fest
        } else if is_weekend {
            &schedule.weekend
        } else {
            &schedule.weekday
        }
    } else if is_raining {
        if let Some(ref rain) = schedule.rain_override {
            rain
        } else if is_weekend {
            &schedule.weekend
        } else {
            &schedule.weekday
        }
    } else if is_weekend {
        &schedule.weekend
    } else {
        &schedule.weekday
    };

    // Find the latest entry whose time <= current time
    let mut active: Option<&ScheduleEntry> = None;
    for entry in entries.iter() {
        if time >= entry.time {
            active = Some(entry);
        } else {
            break;
        }
    }

    // Fall back to last entry (end of day position) or first if time before all entries
    active
        .or_else(|| entries.last())
        .or_else(|| entries.first())
        .cloned()
        .unwrap_or(ScheduleEntry {
            time: 6.0,
            map: MapId::Town,
            x: 24,
            y: 18,
        })
}

/// System: update NPC target positions based on current schedule, then move them.
pub fn update_npc_schedules(
    calendar: Res<Calendar>,
    npc_registry: Res<NpcRegistry>,
    player_state: Res<PlayerState>,
    mut query: Query<(&Npc, &mut NpcMovement, &LogicalPosition)>,
) {
    let current_map = player_state.current_map;
    let time = calendar.time_float();
    let _ = time; // used indirectly through current_schedule_entry

    for (npc, mut movement, logical_pos) in query.iter_mut() {
        let Some(schedule) = npc_registry.schedules.get(&npc.id) else {
            continue;
        };

        let entry = current_schedule_entry(&calendar, schedule);

        // Only update target if on the right map
        if entry.map == current_map {
            let wc = grid_to_world_center(entry.x, entry.y);
            let target_x = wc.x;
            let target_y = wc.y;

            // Only set moving if not already at target
            let dx = target_x - logical_pos.0.x;
            let dy = target_y - logical_pos.0.y;
            let dist_sq = dx * dx + dy * dy;

            movement.target_x = target_x;
            movement.target_y = target_y;
            movement.is_moving = dist_sq > 4.0; // threshold: 2 pixels
        }
    }
}

/// System: move NPC entities toward their target positions (lerp / walk).
pub fn move_npcs_toward_targets(
    time: Res<Time>,
    mut query: Query<(&mut NpcMovement, &mut LogicalPosition), With<Npc>>,
) {
    let dt = time.delta_secs();

    for (mut movement, mut logical_pos) in query.iter_mut() {
        if !movement.is_moving {
            continue;
        }

        let current_x = logical_pos.0.x;
        let current_y = logical_pos.0.y;

        let dx = movement.target_x - current_x;
        let dy = movement.target_y - current_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 2.0 {
            // Snap to target
            logical_pos.0.x = movement.target_x;
            logical_pos.0.y = movement.target_y;
            movement.is_moving = false;
        } else {
            // Move at NPC speed
            let step = (movement.speed * dt).min(dist);
            let dir_x = dx / dist;
            let dir_y = dy / dist;
            logical_pos.0.x += dir_x * step;
            logical_pos.0.y += dir_y * step;
        }
    }
}

/// System: periodically re-check schedule (runs every few seconds, not every frame).
/// Uses a timer resource to avoid checking every tick.
#[derive(Resource)]
#[allow(dead_code)]
pub struct ScheduleUpdateTimer(pub Timer);

impl Default for ScheduleUpdateTimer {
    fn default() -> Self {
        // Check schedule every 5 real seconds (= 50 game minutes at 10x)
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

#[allow(dead_code)]
pub fn tick_schedule_timer(
    time: Res<Time>,
    mut timer: ResMut<ScheduleUpdateTimer>,
) -> bool {
    timer.0.tick(time.delta()).just_finished()
}
