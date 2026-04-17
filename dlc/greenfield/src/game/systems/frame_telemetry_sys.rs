use bevy::prelude::*;
use crate::game::events;

pub fn frame_telemetry_system(time: Res<Time>, mut writer: EventWriter<events::FrameSampledEvent>) {
    info!("frame at {:?}", time.elapsed());
    writer.send(events::FrameSampledEvent);

}
