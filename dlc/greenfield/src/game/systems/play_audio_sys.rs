use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Listen for PlaySfxEvent and spawn one-shot audio sources that auto-despawn.
pub fn play_audio_system(
    mut events: EventReader<PlaySfxEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        if let Some(path) = sfx_path(&event.sfx_id) {
            commands.spawn((
                AudioPlayer::<AudioSource>::new(asset_server.load(path)),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}


