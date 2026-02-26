//! Map transition and day-end event handlers for the NPC domain.

use bevy::prelude::*;
use crate::shared::*;
use std::collections::HashMap;
use super::spawning::{SpawnedNpcs, NpcMapTag, NpcSpriteData, spawn_npcs_for_map};

/// NPC-domain resource tracking how many consecutive days each NPC has gone without a gift.
/// When this counter exceeds 7, friendship starts decaying.
#[derive(Resource, Debug, Default)]
pub struct GiftDecayTracker {
    /// NpcId → number of consecutive days without a gift
    pub days_since_gift: HashMap<NpcId, u32>,
}

/// System: handle MapTransitionEvent — despawn old map NPCs, spawn new map NPCs.
pub fn handle_map_transition(
    mut commands: Commands,
    mut transition_reader: EventReader<MapTransitionEvent>,
    mut spawned: ResMut<SpawnedNpcs>,
    npc_map_tags: Query<(Entity, &NpcMapTag)>,
    calendar: Res<Calendar>,
    npc_registry: Res<NpcRegistry>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut npc_sprites: ResMut<NpcSpriteData>,
) {
    for event in transition_reader.read() {
        // Despawn all currently loaded NPC entities
        let mut despawned_entities = Vec::new();
        for (entity, _tag) in npc_map_tags.iter() {
            commands.entity(entity).despawn_recursive();
            despawned_entities.push(entity);
        }
        // Clear the tracking map
        spawned.entities.retain(|_, e| !despawned_entities.contains(e));

        // Spawn NPCs that belong to the new map
        spawn_npcs_for_map(
            &mut commands,
            &calendar,
            event.to_map,
            &npc_registry,
            &mut spawned,
            &asset_server,
            &mut layouts,
            &mut npc_sprites,
        );
    }
}

/// System: handle DayEndEvent — reset gifted_today for all NPCs and apply friendship decay.
///
/// Decay logic:
/// - If an NPC was gifted today: reset their days_since_gift counter to 0.
/// - If not gifted: increment their counter.
/// - If counter exceeds 7 days: deduct 2 friendship points (minimum 0).
pub fn handle_day_end(
    mut day_end_reader: EventReader<DayEndEvent>,
    mut relationships: ResMut<Relationships>,
    mut decay_tracker: ResMut<GiftDecayTracker>,
    npc_registry: Res<NpcRegistry>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for _event in day_end_reader.read() {
        // Process each known NPC for decay
        let npc_ids: Vec<NpcId> = npc_registry.npcs.keys().cloned().collect();

        for npc_id in &npc_ids {
            let gifted_today = relationships.gifted_today
                .get(npc_id.as_str())
                .copied()
                .unwrap_or(false);

            if gifted_today {
                // Player gifted this NPC today — reset the drought counter
                decay_tracker.days_since_gift.insert(npc_id.clone(), 0);
            } else {
                // No gift today — increment counter
                let days = decay_tracker.days_since_gift
                    .entry(npc_id.clone())
                    .or_insert(0);
                *days += 1;

                // Apply decay if counter has exceeded the grace period of 7 days
                if *days > 7 {
                    let before = relationships.friendship
                        .get(npc_id.as_str())
                        .copied()
                        .unwrap_or(0);

                    // Only decay if there is friendship to lose
                    if before > 0 {
                        relationships.add_friendship(npc_id, -2);

                        // Notify the player if a significant friendship threshold was crossed
                        let after = relationships.friendship
                            .get(npc_id.as_str())
                            .copied()
                            .unwrap_or(0);

                        // Warn when a heart is lost (crossed a 100-point boundary)
                        let hearts_before = (before / 100).min(10);
                        let hearts_after = (after / 100).min(10);
                        if hearts_after < hearts_before {
                            let npc_name = npc_registry.npcs.get(npc_id.as_str())
                                .map(|d| d.name.as_str())
                                .unwrap_or(npc_id.as_str());
                            toast_writer.send(ToastEvent {
                                message: format!(
                                    "Your friendship with {} has faded a little...",
                                    npc_name
                                ),
                                duration_secs: 3.0,
                            });
                        }
                    }
                }
            }
        }

        // Clear gifted_today after processing decay (so the state is fresh for the next day)
        relationships.gifted_today.clear();
    }
}
