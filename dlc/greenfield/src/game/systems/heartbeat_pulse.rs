use bevy::prelude::*;

pub fn heartbeat_pulse_system(mut writer: EventWriter<crate::game::events::HeartbeatPulseEvent>) {
    writer.send(crate::game::events::HeartbeatPulseEvent);
}
