//! Romance & marriage system for Hearthfield.
//!
//! Handles the full romance progression: friendship stages, bouquet (dating),
//! proposal (engagement), wedding, and married life (spouse daily actions,
//! happiness tracking).

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// WEDDING TIMER — local resource for scheduling the wedding ceremony
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Debug, Clone, Default)]
pub struct WeddingTimer {
    pub days_remaining: Option<u8>,
    pub npc_name: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 1: update_relationship_stages
// ═══════════════════════════════════════════════════════════════════════

/// Each frame during Playing, synchronize RelationshipStages with the current
/// heart levels in the Relationships resource.
///
/// Thresholds:
///   0-1 hearts  → Stranger
///   2-3 hearts  → Acquaintance
///   4-5 hearts  → Friend
///   6-7 hearts  → CloseFriend
///   8+  hearts  → remains CloseFriend unless already Dating+
///   10  hearts  → remains Dating unless already Engaged/Married
///
/// Higher stages (Dating, Engaged, Married) are only set by the bouquet,
/// proposal, and wedding systems — never demoted by this system.
pub fn update_relationship_stages(
    relationships: Res<Relationships>,
    npc_registry: Res<NpcRegistry>,
    mut stages: ResMut<RelationshipStages>,
) {
    for (npc_id, _npc_def) in npc_registry.npcs.iter() {
        let hearts = relationships.hearts(npc_id);
        let current = stages
            .stages
            .get(npc_id)
            .copied()
            .unwrap_or(RelationshipStage::Stranger);

        // Never demote from Dating / Engaged / Married via automatic thresholds
        if current >= RelationshipStage::Dating {
            continue;
        }

        let new_stage = match hearts {
            0..=1 => RelationshipStage::Stranger,
            2..=3 => RelationshipStage::Acquaintance,
            4..=5 => RelationshipStage::Friend,
            // 6+ but below Dating — cap at CloseFriend
            _ => RelationshipStage::CloseFriend,
        };

        // Only promote, never demote (prevents flickering if hearts temporarily drop)
        if new_stage > current {
            stages.stages.insert(npc_id.clone(), new_stage);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 2: handle_bouquet
// ═══════════════════════════════════════════════════════════════════════

/// Process BouquetGivenEvent. Validates prerequisites and transitions the
/// NPC to the Dating stage.
pub fn handle_bouquet(
    mut bouquet_reader: EventReader<BouquetGivenEvent>,
    npc_registry: Res<NpcRegistry>,
    relationships: Res<Relationships>,
    mut stages: ResMut<RelationshipStages>,
    mut inventory: ResMut<Inventory>,
    mut toast_writer: EventWriter<ToastEvent>,
    marriage_state: Res<MarriageState>,
) {
    for event in bouquet_reader.read() {
        let npc_name = &event.npc_name;

        // Look up the NPC by name (events use display name, registry uses id)
        let Some((npc_id, npc_def)) = find_npc_by_name(&npc_registry, npc_name) else {
            toast_writer.send(ToastEvent {
                message: format!("There's no one named {} in town.", npc_name),
                duration_secs: 3.0,
            });
            continue;
        };

        // Check: NPC must be a romance candidate
        if !npc_def.is_marriageable {
            toast_writer.send(ToastEvent {
                message: format!("{} is not interested in romance.", npc_name),
                duration_secs: 3.0,
            });
            continue;
        }

        // Check: already married to someone else
        if marriage_state.spouse.is_some() {
            toast_writer.send(ToastEvent {
                message: "You are already married!".to_string(),
                duration_secs: 3.0,
            });
            continue;
        }

        // Check: already dating/engaged/married this NPC
        let current_stage = stages
            .stages
            .get(&npc_id)
            .copied()
            .unwrap_or(RelationshipStage::Stranger);
        if current_stage >= RelationshipStage::Dating {
            let stage_name = match current_stage {
                RelationshipStage::Dating => "dating",
                RelationshipStage::Engaged => "engaged to",
                RelationshipStage::Married => "married to",
                _ => "involved with",
            };
            toast_writer.send(ToastEvent {
                message: format!("You are already {} {}!", stage_name, npc_name),
                duration_secs: 3.0,
            });
            continue;
        }

        // Check: 8+ hearts required
        let hearts = relationships.hearts(&npc_id);
        if hearts < 8 {
            toast_writer.send(ToastEvent {
                message: format!(
                    "You need at least 8 hearts with {} to give a bouquet. (Currently: {})",
                    npc_name, hearts
                ),
                duration_secs: 4.0,
            });
            continue;
        }

        // All checks passed — begin dating
        stages.stages.insert(npc_id.clone(), RelationshipStage::Dating);

        // Consume bouquet from inventory
        inventory.try_remove("bouquet", 1);

        toast_writer.send(ToastEvent {
            message: format!("You are now dating {}!", npc_name),
            duration_secs: 4.0,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 3: handle_proposal
// ═══════════════════════════════════════════════════════════════════════

/// Process ProposalEvent. Validates prerequisites and sets the NPC to Engaged,
/// scheduling the wedding in 3 days.
pub fn handle_proposal(
    mut proposal_reader: EventReader<ProposalEvent>,
    npc_registry: Res<NpcRegistry>,
    relationships: Res<Relationships>,
    house_state: Res<HouseState>,
    mut wedding_timer: ResMut<WeddingTimer>,
    mut relationship_stages: ResMut<RelationshipStages>,
    mut inventory: ResMut<Inventory>,
    mut toast_writer: EventWriter<ToastEvent>,
) {

    for event in proposal_reader.read() {
        let npc_name = &event.npc_name;

        let Some((npc_id, _npc_def)) = find_npc_by_name(&npc_registry, npc_name) else {
            toast_writer.send(ToastEvent {
                message: format!("There's no one named {} in town.", npc_name),
                duration_secs: 3.0,
            });
            continue;
        };

        // Check: must be Dating
        let current_stage = relationship_stages
            .stages
            .get(&npc_id)
            .copied()
            .unwrap_or(RelationshipStage::Stranger);

        if current_stage < RelationshipStage::Dating {
            toast_writer.send(ToastEvent {
                message: format!("You need to be dating {} first!", npc_name),
                duration_secs: 3.0,
            });
            continue;
        }

        if current_stage >= RelationshipStage::Engaged {
            toast_writer.send(ToastEvent {
                message: format!("You are already engaged to or married to {}!", npc_name),
                duration_secs: 3.0,
            });
            continue;
        }

        // Check: 10 hearts
        let hearts = relationships.hearts(&npc_id);
        if hearts < 10 {
            toast_writer.send(ToastEvent {
                message: format!(
                    "Your hearts aren't high enough. You need 10 hearts with {}. (Currently: {})",
                    npc_name, hearts
                ),
                duration_secs: 4.0,
            });
            continue;
        }

        // Check: house tier >= Big
        if house_state.tier < HouseTier::Big {
            toast_writer.send(ToastEvent {
                message: "Your house is too small. Upgrade to at least a Big house before proposing.".to_string(),
                duration_secs: 4.0,
            });
            continue;
        }

        // All checks passed — become engaged
        relationship_stages
            .stages
            .insert(npc_id.clone(), RelationshipStage::Engaged);

        // Consume mermaid pendant from inventory
        inventory.try_remove("mermaid_pendant", 1);

        // Schedule wedding in 3 days
        wedding_timer.days_remaining = Some(3);
        wedding_timer.npc_name = Some(npc_name.clone());

        toast_writer.send(ToastEvent {
            message: format!(
                "{} accepted your proposal! The wedding will be in 3 days!",
                npc_name
            ),
            duration_secs: 5.0,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 4: tick_wedding_timer
// ═══════════════════════════════════════════════════════════════════════

/// DayEndEvent listener. Decrements the wedding timer each day. When it
/// reaches 0, fires a WeddingEvent.
pub fn tick_wedding_timer(
    mut day_end_reader: EventReader<DayEndEvent>,
    mut wedding_timer: ResMut<WeddingTimer>,
    mut wedding_writer: EventWriter<WeddingEvent>,
) {
    for _event in day_end_reader.read() {
        if let Some(ref mut days) = wedding_timer.days_remaining {
            if *days > 0 {
                *days -= 1;
            }
            if *days == 0 {
                if let Some(ref npc_name) = wedding_timer.npc_name {
                    wedding_writer.send(WeddingEvent {
                        npc_name: npc_name.clone(),
                    });
                }
                // Clear the timer so it doesn't fire again
                wedding_timer.days_remaining = None;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 5: handle_wedding
// ═══════════════════════════════════════════════════════════════════════

/// Process WeddingEvent. Finalizes the marriage: sets relationship to Married,
/// updates MarriageState, and notifies the player.
pub fn handle_wedding(
    mut wedding_reader: EventReader<WeddingEvent>,
    npc_registry: Res<NpcRegistry>,
    mut relationship_stages: ResMut<RelationshipStages>,
    mut marriage_state: ResMut<MarriageState>,
    mut relationships: ResMut<Relationships>,
    calendar: Res<Calendar>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for event in wedding_reader.read() {
        let npc_name = &event.npc_name;

        let Some((npc_id, _npc_def)) = find_npc_by_name(&npc_registry, npc_name) else {
            continue;
        };

        // Set relationship stage to Married
        relationship_stages
            .stages
            .insert(npc_id.clone(), RelationshipStage::Married);

        // Update MarriageState
        marriage_state.spouse = Some(npc_name.clone());
        marriage_state.wedding_date = Some((
            calendar.day,
            calendar.season.index() as u8,
            calendar.year as u16,
        ));
        marriage_state.days_married = 0;
        marriage_state.spouse_happiness = 50; // start at neutral-positive

        // Mark spouse in Relationships
        relationships.spouse = Some(npc_id.clone());

        toast_writer.send(ToastEvent {
            message: format!("You married {}! Congratulations!", npc_name),
            duration_secs: 5.0,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 6: spouse_daily_action
// ═══════════════════════════════════════════════════════════════════════

/// Runs every frame during Playing. At 8:00 AM game time, if married, the
/// spouse performs a random helpful action on the farm.
pub fn spouse_daily_action(
    calendar: Res<Calendar>,
    marriage_state: Res<MarriageState>,
    mut spouse_action_writer: EventWriter<SpouseActionEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut last_action_day: Local<u32>,
) {
    // Only trigger at exactly 8:00 AM
    if calendar.hour != 8 || calendar.minute != 0 {
        return;
    }

    // Must be married
    let Some(ref spouse_name) = marriage_state.spouse else {
        return;
    };

    // Prevent firing multiple times on the same day (the system runs every frame)
    let current_total_day = calendar.total_days_elapsed();
    if *last_action_day == current_total_day {
        return;
    }
    *last_action_day = current_total_day;

    // Roll a random action
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();

    let action = if roll < 0.40 {
        // 40% WaterCrops: water 6 random unwatered crop tiles
        SpouseAction::WaterCrops(6)
    } else if roll < 0.65 {
        // 25% FeedAnimals
        SpouseAction::FeedAnimals
    } else if roll < 0.80 {
        // 15% GiveBreakfast: give a random breakfast item
        let breakfasts = [
            "fried_egg",
            "pancakes",
            "toast",
            "porridge",
            "fruit_salad",
        ];
        let idx = rng.gen_range(0..breakfasts.len());
        SpouseAction::GiveBreakfast(breakfasts[idx].to_string())
    } else if roll < 0.90 {
        // 10% RepairFence
        SpouseAction::RepairFence
    } else {
        // 10% StandOnPorch
        SpouseAction::StandOnPorch
    };

    // Toast describing what the spouse did
    let action_description = match &action {
        SpouseAction::WaterCrops(n) => {
            format!("{} watered {} of your crops this morning.", spouse_name, n)
        }
        SpouseAction::FeedAnimals => {
            format!("{} fed all the animals this morning.", spouse_name)
        }
        SpouseAction::GiveBreakfast(item) => {
            let display_name = item.replace('_', " ");
            format!(
                "{} made you {} for breakfast!",
                spouse_name, display_name
            )
        }
        SpouseAction::RepairFence => {
            format!("{} repaired a fence on the farm.", spouse_name)
        }
        SpouseAction::StandOnPorch => {
            format!(
                "{} is standing on the porch, enjoying the morning air.",
                spouse_name
            )
        }
    };

    toast_writer.send(ToastEvent {
        message: action_description,
        duration_secs: 4.0,
    });

    spouse_action_writer.send(SpouseActionEvent { action });
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 7: handle_spouse_action
// ═══════════════════════════════════════════════════════════════════════

/// Reads SpouseActionEvent and applies the concrete game effects.
pub fn handle_spouse_action(
    mut action_reader: EventReader<SpouseActionEvent>,
    mut farm_state: ResMut<FarmState>,
    mut animal_state: ResMut<AnimalState>,
    mut inventory: ResMut<Inventory>,
) {
    for event in action_reader.read() {
        match &event.action {
            SpouseAction::WaterCrops(count) => {
                water_random_crops(&mut farm_state, *count as usize);
            }
            SpouseAction::FeedAnimals => {
                for animal in animal_state.animals.iter_mut() {
                    animal.fed_today = true;
                }
            }
            SpouseAction::GiveBreakfast(item_id) => {
                // Add the breakfast item to the player's inventory
                inventory.try_add(item_id, 1, 99);
            }
            SpouseAction::RepairFence => {
                // Repair fences don't have a strong mechanical effect currently,
                // but we mark it as handled. In the future this could prevent
                // fence degradation or restore broken fences.
            }
            SpouseAction::StandOnPorch => {
                // Purely cosmetic — no game state change.
            }
        }
    }
}

/// Water up to `count` random unwatered crop tiles in the farm.
fn water_random_crops(farm_state: &mut FarmState, count: usize) {
    // Collect all tiles that have crops and are tilled but not yet watered
    let unwatered: Vec<(i32, i32)> = farm_state
        .crops
        .iter()
        .filter(|(pos, crop)| {
            !crop.watered_today
                && !crop.dead
                && farm_state
                    .soil
                    .get(pos)
                    .map_or(false, |s| *s == SoilState::Tilled)
        })
        .map(|(pos, _)| *pos)
        .collect();

    if unwatered.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let to_water = count.min(unwatered.len());

    // Fisher-Yates partial shuffle to pick `to_water` unique random tiles
    let mut indices: Vec<usize> = (0..unwatered.len()).collect();
    for i in 0..to_water {
        let j = rng.gen_range(i..indices.len());
        indices.swap(i, j);
    }

    for i in 0..to_water {
        let pos = unwatered[indices[i]];
        // Water the soil tile
        if let Some(soil) = farm_state.soil.get_mut(&pos) {
            *soil = SoilState::Watered;
        }
        // Mark the crop as watered today
        if let Some(crop) = farm_state.crops.get_mut(&pos) {
            crop.watered_today = true;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM 8: update_spouse_happiness
// ═══════════════════════════════════════════════════════════════════════

/// DayEndEvent listener. Adjusts spouse happiness based on whether the player
/// interacted with the spouse today (using gifted_today as a proxy for talking).
pub fn update_spouse_happiness(
    mut day_end_reader: EventReader<DayEndEvent>,
    relationships: Res<Relationships>,
    mut marriage_state: ResMut<MarriageState>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for _event in day_end_reader.read() {
        // Only applies when married
        if marriage_state.spouse.is_none() {
            continue;
        }

        // Check if the spouse was interacted with today
        let spouse_id = relationships.spouse.as_deref();
        let talked_today = spouse_id
            .and_then(|id| relationships.gifted_today.get(id))
            .copied()
            .unwrap_or(false);

        let old_happiness = marriage_state.spouse_happiness;

        if talked_today {
            marriage_state.spouse_happiness = (marriage_state.spouse_happiness + 2).min(100);
        } else {
            marriage_state.spouse_happiness = (marriage_state.spouse_happiness - 3).max(-100);
        }

        // Increment days married
        marriage_state.days_married += 1;

        // Warn if happiness drops below certain thresholds
        let new_happiness = marriage_state.spouse_happiness;
        if old_happiness >= 0 && new_happiness < 0 {
            if let Some(ref name) = marriage_state.spouse {
                toast_writer.send(ToastEvent {
                    message: format!("{} seems unhappy. Try spending more time together.", name),
                    duration_secs: 4.0,
                });
            }
        } else if old_happiness >= -50 && new_happiness < -50 {
            if let Some(ref name) = marriage_state.spouse {
                toast_writer.send(ToastEvent {
                    message: format!("{} is very unhappy...", name),
                    duration_secs: 4.0,
                });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Look up an NPC definition by display name (case-insensitive match).
/// Returns (npc_id, &NpcDef) if found.
fn find_npc_by_name<'a>(
    npc_registry: &'a NpcRegistry,
    name: &str,
) -> Option<(NpcId, &'a NpcDef)> {
    let name_lower = name.to_lowercase();
    for (id, def) in npc_registry.npcs.iter() {
        if def.name.to_lowercase() == name_lower {
            return Some((id.clone(), def));
        }
    }
    None
}
