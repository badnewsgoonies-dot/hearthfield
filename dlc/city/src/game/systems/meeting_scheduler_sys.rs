use bevy::prelude::*;
use crate::game::resources::{CalendarState, MeetingQueue};
use crate::game::events::MeetingScheduledEvent;

pub fn meeting_scheduler_system(mut queue: ResMut<MeetingQueue>, mut writer: EventWriter<MeetingScheduledEvent>) {
    queue.pending += 1;
    writer.send(MeetingScheduledEvent);

}
