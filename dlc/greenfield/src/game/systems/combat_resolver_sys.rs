use bevy::prelude::*;
use crate::game::events::{EnemyDefeatedEvent, ExperienceGainedEvent, ScoreChangedEvent, ItemPickedUpEvent};
use crate::game::components::Enemy;

pub fn resolve_combat_system(mut commands: Commands, mut reader: EventReader<EnemyDefeatedEvent>, mut xp_writer: EventWriter<ExperienceGainedEvent>, mut score_writer: EventWriter<ScoreChangedEvent>, mut loot_writer: EventWriter<ItemPickedUpEvent>, enemies: Query<Entity, With<Enemy>>) {
    for _ev in reader.read() {
        xp_writer.send(ExperienceGainedEvent { amount: 25 });
        score_writer.send(ScoreChangedEvent { old_score: 0, new_score: 10 });
        loot_writer.send(ItemPickedUpEvent { item_id: 1 });
        if let Some(enemy) = enemies.iter().next() {
            commands.entity(enemy).despawn();
        }
    }
}
