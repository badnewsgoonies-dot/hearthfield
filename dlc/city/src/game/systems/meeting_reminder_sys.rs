use bevy::prelude::*;
use crate::game::events::MeetingReminderEvent;

pub fn meeting_reminder_system(mut writer: EventWriter<MeetingReminderEvent>) {
    writer.send(MeetingReminderEvent);

}
