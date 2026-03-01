//! Quest board system for Hearthfield.
//!
//! Manages daily quest generation, progress tracking, completion rewards,
//! and expiration of timed quests. Quests are posted on a town bulletin board
//! and can involve delivering items, catching fish, harvesting crops,
//! mining ores, talking to NPCs, or slaying monsters.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

// ─────────────────────────────────────────────────────────────────────────────
// Quest template pools — used by post_daily_quests to generate variety
// ─────────────────────────────────────────────────────────────────────────────

/// Delivery quest templates: (item_id, quantity_range, base_gold, title_prefix)
const DELIVER_TEMPLATES: &[(&str, u8, u8, u32, &str)] = &[
    ("wood", 5, 15, 150, "Lumber Delivery"),
    ("stone", 5, 10, 120, "Stone Shipment"),
    ("iron_ore", 3, 8, 200, "Iron Needed"),
    ("copper_ore", 3, 8, 160, "Copper Request"),
    ("gold_ore", 2, 5, 300, "Gold Rush"),
    ("coal", 3, 8, 140, "Coal for Winter"),
    ("fiber", 5, 15, 100, "Fiber Collection"),
    ("hardwood", 3, 8, 250, "Hardwood Wanted"),
    ("clay", 3, 8, 130, "Clay Gathering"),
    ("sap", 5, 10, 110, "Sap Harvest"),
];

/// Harvest quest templates: (crop_id, quantity_range, base_gold, title_prefix)
const HARVEST_TEMPLATES: &[(&str, u8, u8, u32, &str)] = &[
    ("turnip", 3, 8, 180, "Turnip Harvest"),
    ("potato", 3, 6, 200, "Potato Bounty"),
    ("strawberry", 2, 5, 250, "Strawberry Picking"),
    ("tomato", 3, 6, 220, "Tomato Request"),
    ("melon", 1, 3, 300, "Melon Delivery"),
    ("pumpkin", 1, 3, 350, "Pumpkin Order"),
    ("corn", 3, 6, 200, "Corn Collection"),
    ("carrot", 3, 8, 170, "Carrot Haul"),
    ("cabbage", 2, 5, 190, "Cabbage Needed"),
    ("eggplant", 2, 5, 210, "Eggplant Request"),
];

/// Fish quest templates: (fish_id, base_gold, title_prefix)
const CATCH_TEMPLATES: &[(&str, u32, &str)] = &[
    ("bass", 200, "Bass Bounty"),
    ("trout", 220, "Trout Wanted"),
    ("salmon", 280, "Salmon Run"),
    ("catfish", 250, "Catfish Challenge"),
    ("carp", 180, "Carp Request"),
    ("perch", 190, "Perch Hunt"),
    ("pike", 300, "Pike Quest"),
    ("sturgeon", 400, "Sturgeon Search"),
    ("eel", 350, "Eel Expedition"),
    ("sunfish", 160, "Sunfish Catch"),
];

/// Mine quest templates: (item_id, quantity_range, base_gold, title_prefix)
const MINE_TEMPLATES: &[(&str, u8, u8, u32, &str)] = &[
    ("iron_ore", 3, 10, 200, "Mining: Iron"),
    ("copper_ore", 3, 10, 160, "Mining: Copper"),
    ("gold_ore", 2, 5, 350, "Mining: Gold"),
    ("amethyst", 1, 3, 400, "Gem Hunt: Amethyst"),
    ("topaz", 1, 3, 380, "Gem Hunt: Topaz"),
    ("emerald", 1, 2, 500, "Gem Hunt: Emerald"),
    ("ruby", 1, 2, 550, "Gem Hunt: Ruby"),
    ("diamond", 1, 1, 750, "Gem Hunt: Diamond"),
    ("quartz", 2, 5, 250, "Quartz Collection"),
    ("coal", 5, 12, 180, "Coal Expedition"),
];

/// Monster quest templates: (monster_kind, quantity_range, base_gold, title_prefix)
const SLAY_TEMPLATES: &[(&str, u8, u8, u32, &str)] = &[
    ("slime", 5, 12, 250, "Slime Extermination"),
    ("bat", 3, 8, 200, "Bat Clearing"),
    ("skeleton", 3, 6, 300, "Skeleton Hunt"),
    ("ghost", 2, 5, 350, "Ghost Busting"),
    ("golem", 1, 3, 400, "Golem Smashing"),
];

/// NPC IDs used for Talk quests.
const TALK_NPCS: &[&str] = &[
    "mayor_thomas", "elena", "marcus", "dr_iris", "old_pete",
    "chef_rosa", "miner_gil", "librarian_faye", "farmer_dale", "child_lily",
];

/// Generates a unique quest ID from day, season, year, and an index.
fn make_quest_id(day: u8, season: &Season, year: u32, index: u8) -> String {
    format!("quest_y{}s{}d{}_{}", year, season.index(), day, index)
}

/// Map a Season to its u8 index for the `accepted_day` tuple.
fn season_to_idx(season: &Season) -> u8 {
    season.index() as u8
}

// ─────────────────────────────────────────────────────────────────────────────
// System 1: post_daily_quests
// ─────────────────────────────────────────────────────────────────────────────

/// Logs each newly posted quest title for observability.
pub fn log_quest_posted(mut events: EventReader<QuestPostedEvent>) {
    for event in events.read() {
        info!("[Quests] New quest posted: {}", event.quest.title);
    }
}

/// Listens to `DayEndEvent` and generates 2-3 random quests for the next day.
/// Each quest is posted via `QuestPostedEvent` and added to `QuestLog.active`.
pub fn post_daily_quests(
    mut day_end_events: EventReader<DayEndEvent>,
    mut quest_posted: EventWriter<QuestPostedEvent>,
    mut accepted_events: EventWriter<QuestAcceptedEvent>,
    mut quest_log: ResMut<QuestLog>,
    calendar: Res<Calendar>,
    npc_registry: Res<NpcRegistry>,
) {
    for _event in day_end_events.read() {
        let mut rng = rand::thread_rng();

        // Generate 2-3 quests
        let quest_count = rng.gen_range(2u8..=3);

        // Build a pool of available NPC names from registry (fallback to constant list)
        let npc_names: Vec<String> = if npc_registry.npcs.is_empty() {
            TALK_NPCS.iter().map(|s| s.to_string()).collect()
        } else {
            npc_registry.npcs.keys().cloned().collect()
        };

        // Track which quest types we've already picked to add variety
        let mut used_types: Vec<u8> = Vec::new();

        for i in 0..quest_count {
            // Pick a quest type (0-5), trying not to repeat
            let quest_type = loop {
                let t = rng.gen_range(0u8..6);
                if !used_types.contains(&t) || used_types.len() >= 5 {
                    break t;
                }
            };
            used_types.push(quest_type);

            let next_day = if calendar.day >= 28 { 1 } else { calendar.day + 1 };
            let next_season = if calendar.day >= 28 {
                calendar.season.next()
            } else {
                calendar.season
            };
            let next_year = if calendar.day >= 28 && matches!(calendar.season, Season::Winter) {
                calendar.year + 1
            } else {
                calendar.year
            };

            let quest_id = make_quest_id(next_day, &next_season, next_year, i);
            let quest_id_clone = quest_id.clone();
            let deadline = rng.gen_range(3u8..=7);

            // Pick a random NPC as the quest giver
            let giver = if npc_names.is_empty() {
                "Villager".to_string()
            } else {
                npc_names[rng.gen_range(0..npc_names.len())].clone()
            };

            let quest = match quest_type {
                0 => {
                    // Deliver quest
                    let tmpl = &DELIVER_TEMPLATES[rng.gen_range(0..DELIVER_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = tmpl.3 + (qty as u32) * 10;
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.4, giver),
                        description: format!(
                            "{} needs {} {}. Bring them to the quest board.",
                            giver, qty, tmpl.0
                        ),
                        giver: giver.clone(),
                        objective: QuestObjective::Deliver {
                            item_id: tmpl.0.to_string(),
                            quantity: qty,
                            delivered: 0,
                        },
                        reward_gold: gold,
                        reward_items: Vec::new(),
                        reward_friendship: rng.gen_range(20..=50) as i16,
                        days_remaining: Some(deadline),
                        accepted_day: (
                            next_day,
                            season_to_idx(&next_season),
                            next_year as u16,
                        ),
                    }
                }
                1 => {
                    // Harvest quest
                    let tmpl = &HARVEST_TEMPLATES[rng.gen_range(0..HARVEST_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = tmpl.3 + (qty as u32) * 15;
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.4, giver),
                        description: format!(
                            "{} is looking for {} freshly harvested {}.",
                            giver, qty, tmpl.0
                        ),
                        giver: giver.clone(),
                        objective: QuestObjective::Harvest {
                            crop_id: tmpl.0.to_string(),
                            quantity: qty,
                            harvested: 0,
                        },
                        reward_gold: gold,
                        reward_items: Vec::new(),
                        reward_friendship: rng.gen_range(25..=60) as i16,
                        days_remaining: Some(deadline),
                        accepted_day: (
                            next_day,
                            season_to_idx(&next_season),
                            next_year as u16,
                        ),
                    }
                }
                2 => {
                    // Catch fish quest
                    let tmpl = &CATCH_TEMPLATES[rng.gen_range(0..CATCH_TEMPLATES.len())];
                    let gold = tmpl.1 + rng.gen_range(0..100);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.2, giver),
                        description: format!(
                            "{} wants a fresh {}. Cast your line!",
                            giver, tmpl.0
                        ),
                        giver: giver.clone(),
                        objective: QuestObjective::Catch {
                            fish_id: tmpl.0.to_string(),
                            delivered: false,
                        },
                        reward_gold: gold,
                        reward_items: Vec::new(),
                        reward_friendship: rng.gen_range(30..=60) as i16,
                        days_remaining: Some(deadline),
                        accepted_day: (
                            next_day,
                            season_to_idx(&next_season),
                            next_year as u16,
                        ),
                    }
                }
                3 => {
                    // Mine quest
                    let tmpl = &MINE_TEMPLATES[rng.gen_range(0..MINE_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = tmpl.3 + (qty as u32) * 20;
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.4, giver),
                        description: format!(
                            "{} needs {} {} from the mines.",
                            giver, qty, tmpl.0
                        ),
                        giver: giver.clone(),
                        objective: QuestObjective::Mine {
                            item_id: tmpl.0.to_string(),
                            quantity: qty,
                            collected: 0,
                        },
                        reward_gold: gold,
                        reward_items: Vec::new(),
                        reward_friendship: rng.gen_range(25..=55) as i16,
                        days_remaining: Some(deadline),
                        accepted_day: (
                            next_day,
                            season_to_idx(&next_season),
                            next_year as u16,
                        ),
                    }
                }
                4 => {
                    // Talk quest (visit an NPC)
                    let target_npc = if npc_names.len() > 1 {
                        // Pick a different NPC than the giver
                        let mut target = giver.clone();
                        for _ in 0..10 {
                            target = npc_names[rng.gen_range(0..npc_names.len())].clone();
                            if target != giver {
                                break;
                            }
                        }
                        target
                    } else {
                        npc_names.first().cloned().unwrap_or_else(|| "child_lily".to_string())
                    };
                    let gold = rng.gen_range(80..=200);
                    Quest {
                        id: quest_id,
                        title: format!("Visit {} for {}", target_npc, giver),
                        description: format!(
                            "{} wants you to check on {}. Give {} a gift to complete.",
                            giver, target_npc, target_npc
                        ),
                        giver: giver.clone(),
                        objective: QuestObjective::Talk {
                            npc_name: target_npc,
                            talked: false,
                        },
                        reward_gold: gold,
                        reward_items: Vec::new(),
                        reward_friendship: rng.gen_range(30..=70) as i16,
                        days_remaining: Some(deadline),
                        accepted_day: (
                            next_day,
                            season_to_idx(&next_season),
                            next_year as u16,
                        ),
                    }
                }
                _ => {
                    // Slay monsters quest
                    let tmpl = &SLAY_TEMPLATES[rng.gen_range(0..SLAY_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = tmpl.3 + (qty as u32) * 25;
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.4, giver),
                        description: format!(
                            "{} wants you to defeat {} {}s in the mines.",
                            giver, qty, tmpl.0
                        ),
                        giver: giver.clone(),
                        objective: QuestObjective::Slay {
                            monster_kind: tmpl.0.to_string(),
                            quantity: qty,
                            slain: 0,
                        },
                        reward_gold: gold,
                        reward_items: Vec::new(),
                        reward_friendship: rng.gen_range(30..=60) as i16,
                        days_remaining: Some(deadline),
                        accepted_day: (
                            next_day,
                            season_to_idx(&next_season),
                            next_year as u16,
                        ),
                    }
                }
            };

            // Fire event and add to quest log
            quest_posted.send(QuestPostedEvent {
                quest: quest.clone(),
            });
            quest_log.active.push(quest);
            accepted_events.send(QuestAcceptedEvent { quest_id: quest_id_clone });
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 2: handle_quest_accepted
// ─────────────────────────────────────────────────────────────────────────────

/// Reads `QuestAcceptedEvent` and marks a quest as accepted.
/// In our model, quests posted via `post_daily_quests` are already in
/// `QuestLog.active`, so this is a confirmation/no-op if already active.
/// If a UI later separates "posted" from "accepted", this system would
/// move the quest between lists.
pub fn handle_quest_accepted(
    mut accepted_events: EventReader<QuestAcceptedEvent>,
    quest_log: Res<QuestLog>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for event in accepted_events.read() {
        // Check if quest is already in active list
        if let Some(quest) = quest_log.active.iter().find(|q| q.id == event.quest_id) {
            toast_writer.send(ToastEvent {
                message: format!("Quest accepted: {}", quest.title),
                duration_secs: 3.0,
            });
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 3: track_quest_progress
// ─────────────────────────────────────────────────────────────────────────────

/// Listens to game events and updates quest objective progress.
/// Automatically completes quests when their objective is fully met.
///
/// Tracked events:
/// - `CropHarvestedEvent` -> `QuestObjective::Harvest`
/// - `ItemPickupEvent` -> `QuestObjective::Deliver`, `QuestObjective::Mine`, `QuestObjective::Catch`
/// - `GiftGivenEvent` -> `QuestObjective::Talk` (giving a gift = visiting the NPC)
pub fn track_quest_progress(
    mut crop_events: EventReader<CropHarvestedEvent>,
    mut item_events: EventReader<ItemPickupEvent>,
    mut gift_events: EventReader<GiftGivenEvent>,
    mut quest_log: ResMut<QuestLog>,
    mut completed_writer: EventWriter<QuestCompletedEvent>,
) {
    // Collect completed quest IDs to avoid borrow issues
    let mut newly_completed: Vec<(String, u32)> = Vec::new();

    // --- CropHarvestedEvent -> Harvest objectives ---
    for event in crop_events.read() {
        for quest in quest_log.active.iter_mut() {
            if let QuestObjective::Harvest {
                ref crop_id,
                quantity,
                ref mut harvested,
            } = quest.objective
            {
                // Match on crop_id (the event's crop_id is the planted crop identifier)
                if *crop_id == event.crop_id || *crop_id == event.harvest_id {
                    *harvested = (*harvested + event.quantity).min(quantity);
                    if *harvested >= quantity {
                        newly_completed.push((quest.id.clone(), quest.reward_gold));
                    }
                }
            }
        }
    }

    // --- ItemPickupEvent -> Deliver, Mine, and Catch objectives ---
    for event in item_events.read() {
        for quest in quest_log.active.iter_mut() {
            match &mut quest.objective {
                QuestObjective::Deliver {
                    ref item_id,
                    quantity,
                    ref mut delivered,
                } => {
                    if *item_id == event.item_id {
                        *delivered = (*delivered + event.quantity).min(*quantity);
                        if *delivered >= *quantity {
                            newly_completed.push((quest.id.clone(), quest.reward_gold));
                        }
                    }
                }
                QuestObjective::Mine {
                    ref item_id,
                    quantity,
                    ref mut collected,
                } => {
                    if *item_id == event.item_id {
                        *collected = (*collected + event.quantity).min(*quantity);
                        if *collected >= *quantity {
                            newly_completed.push((quest.id.clone(), quest.reward_gold));
                        }
                    }
                }
                QuestObjective::Catch {
                    ref fish_id,
                    ref mut delivered,
                } => {
                    if !*delivered && *fish_id == event.item_id {
                        *delivered = true;
                        newly_completed.push((quest.id.clone(), quest.reward_gold));
                    }
                }
                _ => {}
            }
        }
    }

    // --- GiftGivenEvent -> Talk objectives ---
    for event in gift_events.read() {
        for quest in quest_log.active.iter_mut() {
            if let QuestObjective::Talk {
                ref npc_name,
                ref mut talked,
            } = quest.objective
            {
                if *npc_name == event.npc_id && !*talked {
                    *talked = true;
                    newly_completed.push((quest.id.clone(), quest.reward_gold));
                }
            }
        }
    }

    // --- End of quest-progress updates for this frame.
    // Catch objectives are intentionally handled with ItemPickupEvent since
    // fish collection and catch completion share that same event path.
    // Deduplicate completed quest IDs
    newly_completed.sort_by(|a, b| a.0.cmp(&b.0));
    newly_completed.dedup_by(|a, b| a.0 == b.0);

    // Fire completion events
    for (quest_id, reward_gold) in newly_completed {
        completed_writer.send(QuestCompletedEvent {
            quest_id,
            reward_gold,
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 4: handle_quest_completed
// ─────────────────────────────────────────────────────────────────────────────

/// Processes `QuestCompletedEvent`: awards gold, adds reward items to inventory,
/// boosts friendship with the quest giver, moves quest to completed list,
/// and sends a toast notification.
pub fn handle_quest_completed(
    mut completed_events: EventReader<QuestCompletedEvent>,
    mut quest_log: ResMut<QuestLog>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut inventory: ResMut<Inventory>,
    mut relationships: ResMut<Relationships>,
) {
    for event in completed_events.read() {
        // Find the quest in the active list
        let quest_index = quest_log
            .active
            .iter()
            .position(|q| q.id == event.quest_id);

        if let Some(idx) = quest_index {
            let quest = quest_log.active.remove(idx);

            // Award gold
            if quest.reward_gold > 0 {
                gold_writer.send(GoldChangeEvent {
                    amount: quest.reward_gold as i32,
                    reason: format!("Quest completed: {}", quest.title),
                });
            }

            // Award items
            for (item_id, qty) in &quest.reward_items {
                inventory.try_add(item_id, *qty, 99);
            }

            // Award friendship with quest giver
            if quest.reward_friendship != 0 {
                relationships.add_friendship(
                    &quest.giver,
                    quest.reward_friendship as i32,
                );
            }

            // Toast notification
            toast_writer.send(ToastEvent {
                message: format!(
                    "Quest complete: {}! +{}g",
                    quest.title, quest.reward_gold
                ),
                duration_secs: 4.0,
            });

            // Move to completed list
            quest_log.completed.push(quest.id);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 5: expire_quests
// ─────────────────────────────────────────────────────────────────────────────

/// On `DayEndEvent`, decrements `days_remaining` on all active quests.
/// Removes any quests that have expired (days_remaining reaches 0)
/// and sends a toast notification for each expired quest.
pub fn expire_quests(
    mut day_end_events: EventReader<DayEndEvent>,
    mut quest_log: ResMut<QuestLog>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for _event in day_end_events.read() {
        let mut expired_titles: Vec<String> = Vec::new();

        // Decrement days and collect expired
        quest_log.active.retain_mut(|quest| {
            if let Some(ref mut days) = quest.days_remaining {
                if *days <= 1 {
                    // Quest has expired
                    expired_titles.push(quest.title.clone());
                    return false; // Remove from active
                }
                *days -= 1;
            }
            true // Keep quest
        });

        // Send toast for each expired quest
        for title in expired_titles {
            toast_writer.send(ToastEvent {
                message: format!("Quest expired: {}", title),
                duration_secs: 3.5,
            });
        }
    }
}
