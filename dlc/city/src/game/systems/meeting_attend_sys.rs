use bevy::prelude::*;
use crate::game::events::{MeetingStartingEvent, MeetingAttendedEvent};

pub fn meeting_attend_system(mut reader: EventReader<MeetingStartingEvent>, mut writer: EventWriter<MeetingAttendedEvent>) {
    for _ev in reader.read() {
        writer.send(MeetingAttendedEvent);
    }

}
