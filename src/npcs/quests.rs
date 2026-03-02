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

#[derive(Clone, Copy)]
enum RewardTier {
    Early,
    Mid,
    Late,
}

/// Delivery quest templates: (item_id, quantity_range, base_gold, tier, title_prefix)
const DELIVER_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    ("wood", 5, 15, 120, RewardTier::Early, "Lumber Delivery"),
    ("stone", 5, 10, 130, RewardTier::Early, "Stone Shipment"),
    ("fiber", 5, 15, 110, RewardTier::Early, "Fiber Collection"),
    ("sap", 5, 10, 140, RewardTier::Early, "Sap Harvest"),
    ("copper_ore", 3, 8, 260, RewardTier::Mid, "Copper Request"),
    ("coal", 3, 8, 280, RewardTier::Mid, "Coal for Winter"),
    ("iron_ore", 3, 8, 320, RewardTier::Mid, "Iron Needed"),
    ("hardwood", 3, 8, 360, RewardTier::Mid, "Hardwood Wanted"),
    ("gold_ore", 2, 5, 520, RewardTier::Late, "Gold Rush"),
    ("diamond", 1, 2, 600, RewardTier::Late, "Precious Delivery"),
];

/// Harvest quest templates: (crop_id, quantity_range, base_gold, tier, title_prefix)
const HARVEST_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    ("turnip", 3, 8, 130, RewardTier::Early, "Turnip Harvest"),
    ("potato", 3, 6, 160, RewardTier::Early, "Potato Bounty"),
    ("tomato", 3, 6, 170, RewardTier::Early, "Tomato Request"),
    ("corn", 3, 6, 180, RewardTier::Early, "Corn Collection"),
    ("strawberry", 2, 5, 320, RewardTier::Mid, "Strawberry Picking"),
    ("eggplant", 2, 5, 340, RewardTier::Mid, "Eggplant Request"),
    ("cauliflower", 2, 4, 360, RewardTier::Mid, "Cauliflower Needed"),
    ("melon", 1, 3, 520, RewardTier::Late, "Melon Delivery"),
    ("pumpkin", 1, 3, 560, RewardTier::Late, "Pumpkin Order"),
    ("cranberry", 3, 6, 540, RewardTier::Late, "Cranberry Harvest"),
];

/// Fish quest templates: (fish_id, base_gold, tier, title_prefix)
const CATCH_TEMPLATES: &[(&str, u32, RewardTier, &str)] = &[
    ("bass", 130, RewardTier::Early, "Bass Bounty"),
    ("trout", 150, RewardTier::Early, "Trout Wanted"),
    ("carp", 140, RewardTier::Early, "Carp Request"),
    ("perch", 160, RewardTier::Early, "Perch Hunt"),
    ("herring", 170, RewardTier::Early, "Herring Catch"),
    ("salmon", 320, RewardTier::Mid, "Salmon Run"),
    ("catfish", 340, RewardTier::Mid, "Catfish Challenge"),
    ("pike", 360, RewardTier::Mid, "Pike Quest"),
    ("eel", 380, RewardTier::Mid, "Eel Expedition"),
    ("tuna", 420, RewardTier::Mid, "Tuna Search"),
    ("sturgeon", 560, RewardTier::Late, "Sturgeon Search"),
    ("swordfish", 650, RewardTier::Late, "Swordfish Hunt"),
    ("anglerfish", 720, RewardTier::Late, "Abyssal Catch"),
];

/// Mine quest templates: (item_id, quantity_range, base_gold, tier, title_prefix)
const MINE_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    ("copper_ore", 3, 10, 150, RewardTier::Early, "Mining: Copper"),
    ("iron_ore", 3, 10, 180, RewardTier::Early, "Mining: Iron"),
    ("coal", 5, 12, 140, RewardTier::Early, "Coal Expedition"),
    ("quartz", 2, 5, 170, RewardTier::Early, "Quartz Collection"),
    ("gold_ore", 2, 5, 330, RewardTier::Mid, "Mining: Gold"),
    ("amethyst", 1, 3, 380, RewardTier::Mid, "Gem Hunt: Amethyst"),
    ("emerald", 1, 2, 460, RewardTier::Mid, "Gem Hunt: Emerald"),
    ("ruby", 1, 2, 500, RewardTier::Mid, "Gem Hunt: Ruby"),
    ("diamond", 1, 1, 700, RewardTier::Late, "Gem Hunt: Diamond"),
    ("gold_bar", 1, 2, 620, RewardTier::Late, "Smelter Contract"),
];

/// Monster quest templates: (monster_kind, quantity_range, base_gold, tier, title_prefix)
const SLAY_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    ("slime", 5, 12, 160, RewardTier::Early, "Slime Extermination"),
    ("bat", 3, 8, 180, RewardTier::Early, "Bat Clearing"),
    ("skeleton", 3, 6, 340, RewardTier::Mid, "Skeleton Hunt"),
    ("ghost", 2, 5, 380, RewardTier::Mid, "Ghost Busting"),
    ("golem", 1, 3, 560, RewardTier::Late, "Golem Smashing"),
];

/// Talk quest templates: (title_prefix, tier)
const TALK_TEMPLATES: &[(&str, RewardTier)] = &[
    ("Neighbor Check-in", RewardTier::Early),
    ("Personal Message", RewardTier::Mid),
    ("Delicate Mediation", RewardTier::Late),
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

fn reward_bounds(tier: RewardTier) -> (u32, u32) {
    match tier {
        RewardTier::Early => (100, 300),
        RewardTier::Mid => (300, 600),
        RewardTier::Late => (500, 1000),
    }
}

fn scaled_reward(
    rng: &mut impl Rng,
    tier: RewardTier,
    base_gold: u32,
    quantity: u8,
    per_unit: u32,
) -> u32 {
    let variance = rng.gen_range(0..=80);
    let raw = base_gold + (quantity as u32) * per_unit + variance;
    let (min_gold, max_gold) = reward_bounds(tier);
    raw.clamp(min_gold, max_gold)
}

fn deliver_description(rng: &mut impl Rng, giver: &str, quantity: u8, item_id: &str) -> String {
    match rng.gen_range(0..3) {
        0 => format!(
            "{} needs {} {}. Bring them to the quest board.",
            giver, quantity, item_id
        ),
        1 => format!(
            "Supply run: deliver {} {} to {} before dusk.",
            quantity, item_id, giver
        ),
        _ => format!(
            "{} posted an order for {} {}. Drop them off at town hall.",
            giver, quantity, item_id
        ),
    }
}

fn harvest_description(rng: &mut impl Rng, giver: &str, quantity: u8, crop_id: &str) -> String {
    match rng.gen_range(0..3) {
        0 => format!(
            "{} is looking for {} freshly harvested {}.",
            giver, quantity, crop_id
        ),
        1 => format!(
            "{} asked for a farmer's bundle: {} {} picked today.",
            giver, quantity, crop_id
        ),
        _ => format!(
            "Festival prep request from {}: bring {} ripe {}.",
            giver, quantity, crop_id
        ),
    }
}

fn catch_description(rng: &mut impl Rng, giver: &str, fish_id: &str) -> String {
    match rng.gen_range(0..3) {
        0 => format!("{} wants a fresh {}. Cast your line!", giver, fish_id),
        1 => format!(
            "{} needs one {} for tonight's meal. Deliver it while it's fresh.",
            giver, fish_id
        ),
        _ => format!(
            "Fishing order from {}: catch and deliver a {}.",
            giver, fish_id
        ),
    }
}

fn mine_description(rng: &mut impl Rng, giver: &str, quantity: u8, item_id: &str) -> String {
    match rng.gen_range(0..3) {
        0 => format!("{} needs {} {} from the mines.", giver, quantity, item_id),
        1 => format!(
            "Mine contract for {}: extract {} {} and report back.",
            giver, quantity, item_id
        ),
        _ => format!(
            "{} placed a materials order for {} {} from deep levels.",
            giver, quantity, item_id
        ),
    }
}

fn talk_description(rng: &mut impl Rng, giver: &str, target_npc: &str) -> String {
    match rng.gen_range(0..3) {
        0 => format!(
            "{} wants you to check on {}. Give {} a gift to complete.",
            giver, target_npc, target_npc
        ),
        1 => format!(
            "{} needs a messenger. Visit {} and offer a gift.",
            giver, target_npc
        ),
        _ => format!(
            "Please speak with {} on behalf of {} and bring a small gift.",
            target_npc, giver
        ),
    }
}

fn slay_description(rng: &mut impl Rng, giver: &str, quantity: u8, monster_kind: &str) -> String {
    match rng.gen_range(0..3) {
        0 => format!(
            "{} wants you to defeat {} {}s in the mines.",
            giver, quantity, monster_kind
        ),
        1 => format!(
            "Safety request from {}: clear out {} {}s underground.",
            giver, quantity, monster_kind
        ),
        _ => format!(
            "{} posted a bounty on {} {}s. Return when the mine is safer.",
            giver, quantity, monster_kind
        ),
    }
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
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 12);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver),
                        description: deliver_description(&mut rng, &giver, qty, tmpl.0),
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
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 18);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver),
                        description: harvest_description(&mut rng, &giver, qty, tmpl.0),
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
                    let gold = scaled_reward(&mut rng, tmpl.2, tmpl.1, 1, 20);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.3, giver),
                        description: catch_description(&mut rng, &giver, tmpl.0),
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
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 22);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver),
                        description: mine_description(&mut rng, &giver, qty, tmpl.0),
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
                    let talk_tmpl = &TALK_TEMPLATES[rng.gen_range(0..TALK_TEMPLATES.len())];
                    let base_gold = match talk_tmpl.1 {
                        RewardTier::Early => 120,
                        RewardTier::Mid => 340,
                        RewardTier::Late => 580,
                    };
                    let gold = scaled_reward(&mut rng, talk_tmpl.1, base_gold, 1, 5);
                    Quest {
                        id: quest_id,
                        title: format!("{}: Visit {} for {}", talk_tmpl.0, target_npc, giver),
                        description: talk_description(&mut rng, &giver, &target_npc),
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
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 25);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver),
                        description: slay_description(&mut rng, &giver, qty, tmpl.0),
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
