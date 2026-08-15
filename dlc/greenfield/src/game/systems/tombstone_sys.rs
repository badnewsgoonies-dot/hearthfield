//! Append-only enemy removal/restoration history and its bounded replayer.
//!
//! The historical `KillLog(Vec<KillRecord>)` erased its own evidence with
//! `pop()` during undo.  This generation never deletes a record.  Undo appends
//! `EnemyRestored`, and the replacement receives a new stable `EnemyKey` while
//! retaining the removal snapshot and causal wave parent.
//!
//! Replay certifies one declared quotient only: live enemies introduced by a
//! recorded addressed wave, plus restorations of witnessed removals; their
//! stable key, wave parent, transform, sprite colour, custom size, and flip
//! flags. Timing, player input, Bevy `Entity` handles, and external enemies
//! that were never removed are deliberately outside that quotient.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::components::{Enemy, EnemyKey, WaveOrigin};
use crate::game::systems::wave_address_sys::addressed_wave;

pub const ADDRESSED_WAVE_GENERATION: u64 = 1;
pub const REPLAY_STATE_QUOTIENT: &str = "addressed-wave/restored live enemies: \
EnemyKey + wave parent + Transform + Sprite(color/custom_size/flip); excludes \
timing, inputs, Bevy Entity handles, and unremoved external spawns";

const WAVE_KEY_TAG: u64 = 1 << 63;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HistoryEventRef(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemySnapshot {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub color: [f32; 4],
    pub custom_size: Option<[f32; 2]>,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl EnemySnapshot {
    pub fn capture(transform: &Transform, sprite: &Sprite) -> Self {
        let color = sprite.color.to_srgba();
        Self {
            translation: transform.translation.to_array(),
            rotation: transform.rotation.to_array(),
            scale: transform.scale.to_array(),
            color: [color.red, color.green, color.blue, color.alpha],
            custom_size: sprite.custom_size.map(|size| size.to_array()),
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
        }
    }

    pub fn sprite(&self) -> Sprite {
        Sprite {
            color: Color::srgba(self.color[0], self.color[1], self.color[2], self.color[3]),
            custom_size: self.custom_size.map(Vec2::from_array),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            ..default()
        }
    }

    pub fn transform(&self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_array(self.rotation),
            scale: Vec3::from_array(self.scale),
        }
    }
}

pub fn addressed_enemy_snapshot(x: f32, y: f32, kind: u8) -> EnemySnapshot {
    let color = match kind {
        0 => Color::srgb(0.85, 0.20, 0.20),
        1 => Color::srgb(0.90, 0.55, 0.20),
        2 => Color::srgb(0.80, 0.25, 0.60),
        _ => Color::srgb(0.55, 0.20, 0.20),
    };
    EnemySnapshot::capture(
        &Transform::from_xyz(x, y, 1.0),
        &Sprite {
            color,
            custom_size: Some(Vec2::splat(20.0)),
            ..default()
        },
    )
}

pub fn wave_enemy_key(parent: HistoryEventRef, member_index: usize) -> Option<EnemyKey> {
    if parent.0 >= (1 << 47) || member_index > u16::MAX as usize {
        return None;
    }
    Some(EnemyKey(
        WAVE_KEY_TAG | (parent.0 << 16) | member_index as u64,
    ))
}

#[derive(Resource, Debug)]
pub struct EnemyKeyAllocator {
    next: u64,
}

impl Default for EnemyKeyAllocator {
    fn default() -> Self {
        Self { next: 1 }
    }
}

impl EnemyKeyAllocator {
    pub fn allocate(&mut self) -> Option<EnemyKey> {
        if self.next == 0 || self.next & WAVE_KEY_TAG != 0 {
            return None;
        }
        let key = EnemyKey(self.next);
        self.next = self.next.checked_add(1)?;
        Some(key)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnemyHistoryEvent {
    WaveSpawned {
        seed: u64,
        generation: u64,
    },
    EnemyRemoved {
        entity_key: EnemyKey,
        full_snapshot: EnemySnapshot,
        causal_parent: Option<HistoryEventRef>,
    },
    EnemyRestored {
        removal_receipt_ref: HistoryEventRef,
        new_entity_key: EnemyKey,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemyHistoryRecord {
    pub receipt_ref: HistoryEventRef,
    pub event: EnemyHistoryEvent,
}

/// Historical name retained for API continuity; the value is now append-only.
#[derive(Resource, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KillLog {
    records: Vec<EnemyHistoryRecord>,
}

impl KillLog {
    fn append(&mut self, event: EnemyHistoryEvent) -> HistoryEventRef {
        let receipt_ref = HistoryEventRef(self.records.len() as u64);
        self.records.push(EnemyHistoryRecord { receipt_ref, event });
        receipt_ref
    }

    pub fn records(&self) -> &[EnemyHistoryRecord] {
        &self.records
    }

    pub fn append_wave_spawned(&mut self, seed: u64, generation: u64) -> HistoryEventRef {
        self.append(EnemyHistoryEvent::WaveSpawned { seed, generation })
    }

    pub fn append_enemy_removed(
        &mut self,
        entity_key: EnemyKey,
        full_snapshot: EnemySnapshot,
        causal_parent: Option<HistoryEventRef>,
    ) -> HistoryEventRef {
        self.append(EnemyHistoryEvent::EnemyRemoved {
            entity_key,
            full_snapshot,
            causal_parent,
        })
    }

    pub fn append_enemy_restored(
        &mut self,
        removal_receipt_ref: HistoryEventRef,
        new_entity_key: EnemyKey,
    ) -> HistoryEventRef {
        self.append(EnemyHistoryEvent::EnemyRestored {
            removal_receipt_ref,
            new_entity_key,
        })
    }

    pub fn has_removal_for(&self, entity_key: EnemyKey) -> bool {
        self.records().iter().any(|record| {
            matches!(
                record.event,
                EnemyHistoryEvent::EnemyRemoved { entity_key: key, .. }
                    if key == entity_key
            )
        })
    }

    fn latest_unrestored_removal(
        &self,
    ) -> Option<(HistoryEventRef, EnemySnapshot, Option<HistoryEventRef>)> {
        let restored: BTreeSet<_> = self
            .records()
            .iter()
            .filter_map(|record| match record.event {
                EnemyHistoryEvent::EnemyRestored {
                    removal_receipt_ref,
                    ..
                } => Some(removal_receipt_ref),
                _ => None,
            })
            .collect();
        self.records()
            .iter()
            .rev()
            .find_map(|record| match &record.event {
                EnemyHistoryEvent::EnemyRemoved {
                    full_snapshot,
                    causal_parent,
                    ..
                } if !restored.contains(&record.receipt_ref) => {
                    Some((record.receipt_ref, full_snapshot.clone(), *causal_parent))
                }
                _ => None,
            })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayedEnemy {
    pub entity_key: EnemyKey,
    pub full_snapshot: EnemySnapshot,
    pub causal_parent: Option<HistoryEventRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayCertificate {
    pub schema: String,
    pub state_quotient: String,
    pub history_event_count: usize,
    pub live_enemies: Vec<ReplayedEnemy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRefusal {
    pub reason_code: &'static str,
    pub receipt_ref: Option<HistoryEventRef>,
}

fn replay_refusal(
    reason_code: &'static str,
    receipt_ref: Option<HistoryEventRef>,
) -> ReplayRefusal {
    ReplayRefusal {
        reason_code,
        receipt_ref,
    }
}

pub fn replay_history(log: &KillLog) -> Result<ReplayCertificate, ReplayRefusal> {
    let mut live = BTreeMap::<EnemyKey, ReplayedEnemy>::new();
    let mut removals = BTreeMap::<HistoryEventRef, ReplayedEnemy>::new();
    let mut restored = BTreeSet::<HistoryEventRef>::new();
    let mut seen_keys = BTreeSet::<EnemyKey>::new();

    for (index, record) in log.records().iter().enumerate() {
        let expected_ref = HistoryEventRef(index as u64);
        if record.receipt_ref != expected_ref {
            return Err(replay_refusal(
                "history_receipt_sequence_torn",
                Some(record.receipt_ref),
            ));
        }
        match &record.event {
            EnemyHistoryEvent::WaveSpawned { seed, generation } => {
                if *generation != ADDRESSED_WAVE_GENERATION {
                    return Err(replay_refusal(
                        "wave_generation_unreplayable",
                        Some(record.receipt_ref),
                    ));
                }
                for (member_index, (x, y, _hp, kind)) in
                    addressed_wave(*seed).into_iter().enumerate()
                {
                    let key =
                        wave_enemy_key(record.receipt_ref, member_index).ok_or_else(|| {
                            replay_refusal("wave_enemy_key_outside_field", Some(record.receipt_ref))
                        })?;
                    let enemy = ReplayedEnemy {
                        entity_key: key,
                        full_snapshot: addressed_enemy_snapshot(x, y, kind),
                        causal_parent: Some(record.receipt_ref),
                    };
                    if !seen_keys.insert(key) || live.insert(key, enemy).is_some() {
                        return Err(replay_refusal(
                            "replay_enemy_key_ambiguous",
                            Some(record.receipt_ref),
                        ));
                    }
                }
            }
            EnemyHistoryEvent::EnemyRemoved {
                entity_key,
                full_snapshot,
                causal_parent,
            } => {
                if let Some(observed) = live.remove(entity_key) {
                    if observed.full_snapshot != *full_snapshot
                        || observed.causal_parent != *causal_parent
                    {
                        return Err(replay_refusal(
                            "removal_snapshot_torn",
                            Some(record.receipt_ref),
                        ));
                    }
                } else if causal_parent.is_some() {
                    return Err(replay_refusal(
                        "removal_subject_absent",
                        Some(record.receipt_ref),
                    ));
                } else if !seen_keys.insert(*entity_key) {
                    return Err(replay_refusal(
                        "removal_subject_already_witnessed",
                        Some(record.receipt_ref),
                    ));
                }
                removals.insert(
                    record.receipt_ref,
                    ReplayedEnemy {
                        entity_key: *entity_key,
                        full_snapshot: full_snapshot.clone(),
                        causal_parent: *causal_parent,
                    },
                );
            }
            EnemyHistoryEvent::EnemyRestored {
                removal_receipt_ref,
                new_entity_key,
            } => {
                let removed = removals.get(removal_receipt_ref).ok_or_else(|| {
                    replay_refusal("restoration_removal_unknown", Some(record.receipt_ref))
                })?;
                if !restored.insert(*removal_receipt_ref) {
                    return Err(replay_refusal(
                        "restoration_duplicate",
                        Some(record.receipt_ref),
                    ));
                }
                if !seen_keys.insert(*new_entity_key) {
                    return Err(replay_refusal(
                        "restoration_entity_key_reused",
                        Some(record.receipt_ref),
                    ));
                }
                let replacement = ReplayedEnemy {
                    entity_key: *new_entity_key,
                    full_snapshot: removed.full_snapshot.clone(),
                    causal_parent: removed.causal_parent,
                };
                if live.insert(*new_entity_key, replacement).is_some() {
                    return Err(replay_refusal(
                        "restoration_entity_key_ambiguous",
                        Some(record.receipt_ref),
                    ));
                }
            }
        }
    }

    Ok(ReplayCertificate {
        schema: "enemy_history_replay_certificate.v0".to_string(),
        state_quotient: REPLAY_STATE_QUOTIENT.to_string(),
        history_event_count: log.records().len(),
        live_enemies: live.into_values().collect(),
    })
}

/// Cold import boundary. The game loop audits the resident resource directly;
/// persisted JSON enters through this function after a process restart.
#[allow(dead_code)]
pub fn replay_history_json(encoded: &str) -> Result<ReplayCertificate, ReplayRefusal> {
    let log: KillLog = serde_json::from_str(encoded)
        .map_err(|_| replay_refusal("history_encoding_invalid", None))?;
    replay_history(&log)
}

pub fn audit_enemy_history_system(log: Res<KillLog>) {
    if !log.is_changed() {
        return;
    }
    if let Err(refusal) = replay_history(&log) {
        error!(
            "enemy history replay refused: {} at {:?}",
            refusal.reason_code, refusal.receipt_ref
        );
    }
}

/// Press **U**: append a restoration; never erase the removal receipt.
pub fn revert_last_kill_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut log: ResMut<KillLog>,
    mut allocator: ResMut<EnemyKeyAllocator>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::KeyU) {
        return;
    }
    let Some((removal_ref, snapshot, causal_parent)) = log.latest_unrestored_removal() else {
        return;
    };
    let Some(new_key) = allocator.allocate() else {
        error!("enemy key field exhausted; restoration refused");
        return;
    };
    let mut spawned = commands.spawn((snapshot.sprite(), snapshot.transform(), Enemy, new_key));
    if let Some(parent) = causal_parent {
        spawned.insert(WaveOrigin(parent.0));
    }
    log.append_enemy_restored(removal_ref, new_key);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    const COLD_REPLAY_INPUT: &str = "HEARTHFIELD_COLD_REPLAY_INPUT";
    const COLD_REPLAY_PREFIX: &str = "HEARTHFIELD_COLD_REPLAY_CERTIFICATE=";

    #[test]
    fn cold_replay_subprocess_entrypoint() {
        let Ok(encoded) = std::env::var(COLD_REPLAY_INPUT) else {
            return;
        };
        let certificate = replay_history_json(&encoded).expect("cold child must replay history");
        println!(
            "{COLD_REPLAY_PREFIX}{}",
            serde_json::to_string(&certificate).unwrap()
        );
    }

    #[test]
    fn separate_process_cold_replay_reconstructs_the_declared_quotient_exactly() {
        let seed = 42;
        let mut log = KillLog::default();
        let wave_ref = log.append_wave_spawned(seed, ADDRESSED_WAVE_GENERATION);
        let wave = addressed_wave(seed);
        let (x, y, _hp, kind) = wave[3];
        let removed_key = wave_enemy_key(wave_ref, 3).unwrap();
        let removal_ref = log.append_enemy_removed(
            removed_key,
            addressed_enemy_snapshot(x, y, kind),
            Some(wave_ref),
        );
        let replacement_key = EnemyKey(77);
        log.append_enemy_restored(removal_ref, replacement_key);

        let warm = replay_history(&log).unwrap();
        let encoded = serde_json::to_string(&log).unwrap();
        drop(log);

        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "game::systems::tombstone_sys::tests::cold_replay_subprocess_entrypoint",
                "--nocapture",
            ])
            .env(COLD_REPLAY_INPUT, encoded)
            .output()
            .expect("cold replay child must start");
        assert!(
            output.status.success(),
            "cold replay child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let cold_json = stdout
            .lines()
            .find_map(|line| {
                line.find(COLD_REPLAY_PREFIX)
                    .map(|offset| &line[offset + COLD_REPLAY_PREFIX.len()..])
            })
            .expect("cold replay child must emit its certificate");
        let cold: ReplayCertificate = serde_json::from_str(cold_json).unwrap();

        assert_eq!(warm, cold);
        assert_eq!(
            serde_json::to_vec(&warm).unwrap(),
            serde_json::to_vec(&cold).unwrap()
        );
        assert_eq!(cold.history_event_count, 3);
        assert_eq!(cold.live_enemies.len(), wave.len());
        assert!(!cold
            .live_enemies
            .iter()
            .any(|enemy| enemy.entity_key == removed_key));
        assert!(cold
            .live_enemies
            .iter()
            .any(|enemy| enemy.entity_key == replacement_key));
    }

    #[test]
    fn undo_appends_restoration_and_keeps_the_removal_receipt() {
        let snapshot = addressed_enemy_snapshot(10.0, 20.0, 1);
        let mut log = KillLog::default();
        let removal_ref = log.append_enemy_removed(EnemyKey(5), snapshot.clone(), None);
        let before = log.records.clone();

        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<EnemyKeyAllocator>()
            .insert_resource(log)
            .add_systems(Update, revert_last_kill_system);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyU);
        app.update();

        let log = app.world().resource::<KillLog>();
        assert_eq!(&log.records[..before.len()], before.as_slice());
        assert_eq!(log.records.len(), before.len() + 1);
        assert!(matches!(
            log.records.last().unwrap().event,
            EnemyHistoryEvent::EnemyRestored {
                removal_receipt_ref,
                ..
            } if removal_receipt_ref == removal_ref
        ));
        let mut query = app.world_mut().query::<(&EnemyKey, &Transform, &Sprite)>();
        let enemies: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(enemies.len(), 1);
        assert_eq!(EnemySnapshot::capture(enemies[0].1, enemies[0].2), snapshot);
    }

    #[test]
    fn duplicate_restoration_refuses_during_replay() {
        let mut log = KillLog::default();
        let removal_ref =
            log.append_enemy_removed(EnemyKey(5), addressed_enemy_snapshot(10.0, 20.0, 0), None);
        log.append_enemy_restored(removal_ref, EnemyKey(6));
        log.append_enemy_restored(removal_ref, EnemyKey(7));
        assert_eq!(
            replay_history(&log).unwrap_err().reason_code,
            "restoration_duplicate"
        );
    }

    #[test]
    fn torn_history_sequence_refuses_by_name() {
        let mut log = KillLog::default();
        log.append_wave_spawned(42, ADDRESSED_WAVE_GENERATION);
        log.records[0].receipt_ref = HistoryEventRef(9);

        let refusal = replay_history(&log).unwrap_err();
        assert_eq!(refusal.reason_code, "history_receipt_sequence_torn");
        assert_eq!(refusal.receipt_ref, Some(HistoryEventRef(9)));
    }

    #[test]
    fn removal_snapshot_must_match_the_causal_wave() {
        let seed = 42;
        let mut log = KillLog::default();
        let wave_ref = log.append_wave_spawned(seed, ADDRESSED_WAVE_GENERATION);
        let (x, y, _hp, kind) = addressed_wave(seed)[0];
        let mut torn = addressed_enemy_snapshot(x, y, kind);
        torn.translation[0] += 1.0;
        log.append_enemy_removed(wave_enemy_key(wave_ref, 0).unwrap(), torn, Some(wave_ref));

        assert_eq!(
            replay_history(&log).unwrap_err().reason_code,
            "removal_snapshot_torn"
        );
    }

    #[test]
    fn replay_refuses_reused_stable_identity() {
        let mut duplicate_removal = KillLog::default();
        duplicate_removal.append_enemy_removed(
            EnemyKey(5),
            addressed_enemy_snapshot(10.0, 20.0, 0),
            None,
        );
        duplicate_removal.append_enemy_removed(
            EnemyKey(5),
            addressed_enemy_snapshot(10.0, 20.0, 0),
            None,
        );
        assert_eq!(
            replay_history(&duplicate_removal).unwrap_err().reason_code,
            "removal_subject_already_witnessed"
        );

        let mut reused_restoration = KillLog::default();
        let removal_ref = reused_restoration.append_enemy_removed(
            EnemyKey(8),
            addressed_enemy_snapshot(30.0, 40.0, 1),
            None,
        );
        reused_restoration.append_enemy_restored(removal_ref, EnemyKey(8));
        assert_eq!(
            replay_history(&reused_restoration).unwrap_err().reason_code,
            "restoration_entity_key_reused"
        );
    }
}
