use bevy::prelude::*;
use crate::game::events::{EnemyDefeatedEvent, ExperienceGainedEvent, ScoreChangedEvent, ItemPickedUpEvent};
use crate::game::components::Enemy;
use hearthfield::shared::{CropHarvestedEvent, ItemQuality};

pub fn resolve_combat_system(
    mut commands: Commands,
    mut reader: EventReader<EnemyDefeatedEvent>,
    mut xp_writer: EventWriter<ExperienceGainedEvent>,
    mut score_writer: EventWriter<ScoreChangedEvent>,
    mut loot_writer: EventWriter<ItemPickedUpEvent>,
    mut crop_writer: EventWriter<CropHarvestedEvent>,
    enemies: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    mut kill_log: ResMut<crate::game::systems::tombstone_sys::KillLog>,
) {
    for _ev in reader.read() {
        xp_writer.send(ExperienceGainedEvent { amount: 25 });
        score_writer.send(ScoreChangedEvent { old_score: 0, new_score: 10 });
        loot_writer.send(ItemPickedUpEvent { item_id: 1 });
        // I5: also emit the host's CropHarvestedEvent. Theming:
        // defeating a critter saves the crop it would have eaten,
        // counted as a successful harvest in shared Hearthfield state.
        if let Some((enemy, transform, sprite)) = enemies.iter().next() {
            let pos = transform.translation;
            crop_writer.send(CropHarvestedEvent {
                crop_id: "greenfield_turnip".to_string(),
                harvest_id: "turnip".to_string(),
                quantity: 1,
                x: (pos.x / 16.0) as i32,
                y: (pos.y / 16.0) as i32,
                quality: Some(ItemQuality::Normal),
            });
            // tombstone: append the witness BEFORE removal so death is exactly reversible (press U).
            let s = sprite.color.to_srgba();
            kill_log.0.push(crate::game::systems::tombstone_sys::KillRecord {
                x: pos.x, y: pos.y, r: s.red, g: s.green, b: s.blue,
            });
            commands.entity(enemy).despawn();
        }
    }
}
