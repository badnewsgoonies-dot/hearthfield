use crate::game::components::{Enemy, EnemyKey, WaveOrigin};
use crate::game::events::{
    EnemyDefeatedEvent, ExperienceGainedEvent, ItemPickedUpEvent, ScoreChangedEvent,
};
use crate::game::systems::tombstone_sys::{EnemySnapshot, HistoryEventRef, KillLog};
use bevy::prelude::*;
use hearthfield::shared::{CropHarvestedEvent, ItemQuality};

pub fn resolve_combat_system(
    mut commands: Commands,
    mut reader: EventReader<EnemyDefeatedEvent>,
    mut xp_writer: EventWriter<ExperienceGainedEvent>,
    mut score_writer: EventWriter<ScoreChangedEvent>,
    mut loot_writer: EventWriter<ItemPickedUpEvent>,
    mut crop_writer: EventWriter<CropHarvestedEvent>,
    enemies: Query<(&EnemyKey, &Transform, &Sprite, Option<&WaveOrigin>), With<Enemy>>,
    mut kill_log: ResMut<KillLog>,
) {
    for ev in reader.read() {
        let Ok((enemy_key, transform, sprite, wave_origin)) = enemies.get(ev.enemy) else {
            continue;
        };
        if kill_log.has_removal_for(*enemy_key) {
            continue;
        }
        xp_writer.send(ExperienceGainedEvent { amount: 25 });
        score_writer.send(ScoreChangedEvent {
            old_score: 0,
            new_score: 10,
        });
        loot_writer.send(ItemPickedUpEvent { item_id: 1 });
        // I5: also emit the host's CropHarvestedEvent. Theming:
        // defeating a critter saves the crop it would have eaten,
        // counted as a successful harvest in shared Hearthfield state.
        let pos = transform.translation;
        crop_writer.send(CropHarvestedEvent {
            crop_id: "greenfield_turnip".to_string(),
            harvest_id: "turnip".to_string(),
            quantity: 1,
            x: (pos.x / 16.0) as i32,
            y: (pos.y / 16.0) as i32,
            quality: Some(ItemQuality::Normal),
        });
        kill_log.append_enemy_removed(
            *enemy_key,
            EnemySnapshot::capture(transform, sprite),
            wave_origin.map(|origin| HistoryEventRef(origin.0)),
        );
        commands.entity(ev.enemy).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::events::{
        AttackStartedEvent, CombatInitiatedEvent, CombatResolvedEvent, DamageAppliedEvent,
        EnemyDefeatedEvent, ExperienceGainedEvent, ItemPickedUpEvent, ScoreChangedEvent,
    };
    use crate::game::systems::combat_attack_sys::combat_attack_system;
    use crate::game::systems::combat_damage_sys::combat_damage_system;
    use crate::game::systems::combat_initiate_sys::combat_initiate_system;
    use crate::game::systems::combat_resolve_sys::combat_resolve_system;
    use crate::game::systems::tombstone_sys::{EnemyHistoryEvent, KillLog};
    use hearthfield::shared::CropHarvestedEvent;

    #[test]
    fn defeated_event_removes_its_named_enemy_only_once() {
        let mut app = App::new();
        app.add_event::<EnemyDefeatedEvent>()
            .add_event::<ExperienceGainedEvent>()
            .add_event::<ScoreChangedEvent>()
            .add_event::<ItemPickedUpEvent>()
            .add_event::<CropHarvestedEvent>()
            .init_resource::<KillLog>()
            .add_systems(Update, resolve_combat_system);

        let first = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyKey(11),
                Transform::from_xyz(10.0, 20.0, 1.0),
                Sprite::default(),
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyKey(22),
                Transform::from_xyz(30.0, 40.0, 1.0),
                Sprite {
                    color: Color::srgb(0.2, 0.3, 0.4),
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
            ))
            .id();
        app.world_mut()
            .send_event(EnemyDefeatedEvent { enemy: target });
        app.world_mut()
            .send_event(EnemyDefeatedEvent { enemy: target });
        app.update();

        assert!(app.world().get_entity(first).is_ok());
        assert!(app.world().get_entity(target).is_err());
        let log = app.world().resource::<KillLog>();
        assert_eq!(log.records().len(), 1);
        assert!(matches!(
            log.records()[0].event,
            EnemyHistoryEvent::EnemyRemoved {
                entity_key: EnemyKey(22),
                ..
            }
        ));
    }

    #[test]
    fn combat_event_chain_preserves_the_selected_enemy_identity() {
        let mut app = App::new();
        app.add_event::<CombatInitiatedEvent>()
            .add_event::<AttackStartedEvent>()
            .add_event::<DamageAppliedEvent>()
            .add_event::<CombatResolvedEvent>()
            .add_event::<EnemyDefeatedEvent>()
            .add_event::<ExperienceGainedEvent>()
            .add_event::<ScoreChangedEvent>()
            .add_event::<ItemPickedUpEvent>()
            .add_event::<CropHarvestedEvent>()
            .init_resource::<KillLog>()
            .add_systems(
                Update,
                (
                    combat_initiate_system,
                    combat_attack_system,
                    combat_damage_system,
                    combat_resolve_system,
                    resolve_combat_system,
                )
                    .chain(),
            );

        let untouched = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyKey(31),
                Transform::from_xyz(5.0, 6.0, 1.0),
                Sprite::default(),
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyKey(32),
                Transform::from_xyz(7.0, 8.0, 1.0),
                Sprite::default(),
            ))
            .id();
        for _ in 0..3 {
            app.world_mut()
                .send_event(CombatInitiatedEvent { enemy: target });
        }
        app.update();

        assert!(app.world().get_entity(untouched).is_ok());
        assert!(app.world().get_entity(target).is_err());
        let log = app.world().resource::<KillLog>();
        assert!(matches!(
            log.records()[0].event,
            EnemyHistoryEvent::EnemyRemoved {
                entity_key: EnemyKey(32),
                ..
            }
        ));
    }
}
