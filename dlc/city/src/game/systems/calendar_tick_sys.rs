use bevy::prelude::*;
use crate::game::resources::CalendarState;

pub fn calendar_tick_system(time: Res<Time>, mut cal: ResMut<CalendarState>) {
    cal.current_minute = cal.current_minute.saturating_add(time.delta_secs() as u32);

}
