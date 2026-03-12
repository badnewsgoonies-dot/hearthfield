//! Quest board system for Hearthfield.
//!
//! Manages daily quest generation, progress tracking, completion rewards,
//! and expiration of timed quests. Quests are posted on a town bulletin board
//! and can involve delivering items, catching fish, harvesting crops,
//! mining ores, talking to NPCs, or slaying monsters.

use crate::shared::*;
use bevy::prelude::*;
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
    // Margaret – baked goods & ingredients
    ("egg", 3, 8, 150, RewardTier::Early, "Egg Delivery"),
    ("milk", 3, 6, 160, RewardTier::Early, "Fresh Milk Run"),
    ("bread", 2, 4, 180, RewardTier::Early, "Bread Order"),
    (
        "fried_egg",
        2,
        4,
        200,
        RewardTier::Early,
        "Breakfast Platter",
    ),
    ("pancakes", 2, 4, 250, RewardTier::Mid, "Pancake Platter"),
    ("cookie", 3, 6, 280, RewardTier::Mid, "Cookie Batch"),
    ("cake", 1, 3, 340, RewardTier::Mid, "Cake Request"),
    // Marco – cooking ingredients & prepared food
    ("pizza", 1, 3, 320, RewardTier::Mid, "Pizza Night"),
    ("spaghetti", 1, 3, 310, RewardTier::Mid, "Pasta Delivery"),
    // Elena – smelted bars
    (
        "copper_bar",
        2,
        5,
        300,
        RewardTier::Mid,
        "Copper Bars Needed",
    ),
    ("iron_bar", 2, 4, 380, RewardTier::Mid, "Iron Bars Wanted"),
    ("gold_bar", 1, 2, 580, RewardTier::Late, "Gold Bars Order"),
    // Mira – exotic & rare goods
    (
        "ancient_fruit",
        1,
        2,
        650,
        RewardTier::Late,
        "Ancient Relic Fruit",
    ),
    // Nora – animal products
    ("wool", 2, 5, 200, RewardTier::Early, "Wool Wanted"),
    // Sam – random fun items
    (
        "baked_potato",
        3,
        6,
        170,
        RewardTier::Early,
        "Baked Potato Stack",
    ),
];

/// Harvest quest templates: (crop_id, quantity_range, base_gold, tier, title_prefix)
const HARVEST_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    ("turnip", 3, 8, 130, RewardTier::Early, "Turnip Harvest"),
    ("potato", 3, 6, 160, RewardTier::Early, "Potato Bounty"),
    ("tomato", 3, 6, 170, RewardTier::Early, "Tomato Request"),
    ("corn", 3, 6, 180, RewardTier::Early, "Corn Collection"),
    (
        "strawberry",
        2,
        5,
        320,
        RewardTier::Mid,
        "Strawberry Picking",
    ),
    ("eggplant", 2, 5, 340, RewardTier::Mid, "Eggplant Request"),
    (
        "cauliflower",
        2,
        4,
        360,
        RewardTier::Mid,
        "Cauliflower Needed",
    ),
    ("melon", 1, 3, 520, RewardTier::Late, "Melon Delivery"),
    ("pumpkin", 1, 3, 560, RewardTier::Late, "Pumpkin Order"),
    (
        "cranberry",
        3,
        6,
        540,
        RewardTier::Late,
        "Cranberry Harvest",
    ),
    // Lily – flowers / seasonal crops
    (
        "blueberry",
        3,
        6,
        180,
        RewardTier::Early,
        "Blueberry Picking",
    ),
    // Nora – staple crops
    ("wheat", 4, 8, 150, RewardTier::Early, "Wheat Bundle"),
    ("yam", 2, 5, 200, RewardTier::Mid, "Yam Harvest"),
    // Marco / Mira – premium ingredients
    ("coffee", 2, 4, 320, RewardTier::Mid, "Coffee Beans"),
    // Mira – exotic produce
    (
        "ancient_fruit",
        1,
        2,
        580,
        RewardTier::Late,
        "Ancient Fruit Harvest",
    ),
    // Mayor Rex – town event staples
    (
        "pumpkin",
        2,
        4,
        500,
        RewardTier::Late,
        "Festival Pumpkin Order",
    ),
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
    // Old Tom – common catches he wants fresh
    ("sardine", 110, RewardTier::Early, "Sardine Request"),
    ("herring", 140, RewardTier::Early, "Smoked Herring Run"),
    // Marco – seafood for the kitchen
    ("trout", 160, RewardTier::Early, "Pan-Fried Trout"),
    ("salmon", 300, RewardTier::Mid, "Salmon Fillet"),
    // Doc – rare medicinal fish
    ("sturgeon", 500, RewardTier::Late, "Sturgeon for Research"),
];

/// Mine quest templates: (item_id, quantity_range, base_gold, tier, title_prefix)
const MINE_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    (
        "copper_ore",
        3,
        10,
        150,
        RewardTier::Early,
        "Mining: Copper",
    ),
    ("iron_ore", 3, 10, 180, RewardTier::Early, "Mining: Iron"),
    ("coal", 5, 12, 140, RewardTier::Early, "Coal Expedition"),
    ("quartz", 2, 5, 170, RewardTier::Early, "Quartz Collection"),
    ("gold_ore", 2, 5, 330, RewardTier::Mid, "Mining: Gold"),
    ("amethyst", 1, 3, 380, RewardTier::Mid, "Gem Hunt: Amethyst"),
    ("emerald", 1, 2, 460, RewardTier::Mid, "Gem Hunt: Emerald"),
    ("ruby", 1, 2, 500, RewardTier::Mid, "Gem Hunt: Ruby"),
    ("diamond", 1, 1, 700, RewardTier::Late, "Gem Hunt: Diamond"),
    ("gold_bar", 1, 2, 620, RewardTier::Late, "Smelter Contract"),
    // Elena – processed bars
    (
        "copper_bar",
        2,
        5,
        280,
        RewardTier::Mid,
        "Smelted Copper Bars",
    ),
    ("iron_bar", 2, 4, 350, RewardTier::Mid, "Forged Iron Bars"),
];

/// Monster quest templates: (monster_kind, quantity_range, base_gold, tier, title_prefix)
const SLAY_TEMPLATES: &[(&str, u8, u8, u32, RewardTier, &str)] = &[
    (
        "green_slime",
        5,
        12,
        160,
        RewardTier::Early,
        "Slime Extermination",
    ),
    ("bat", 3, 8, 180, RewardTier::Early, "Bat Clearing"),
    ("rock_crab", 2, 5, 340, RewardTier::Mid, "Rock Crab Hunt"),
];

/// Talk quest templates: (title_prefix, tier)
const TALK_TEMPLATES: &[(&str, RewardTier)] = &[
    ("Neighbor Check-in", RewardTier::Early),
    ("Personal Message", RewardTier::Mid),
    ("Delicate Mediation", RewardTier::Late),
    // NPC-flavored talk quests
    ("Baker's Errand", RewardTier::Early),
    ("Chef's Inquiry", RewardTier::Early),
    ("Mayor's Summons", RewardTier::Mid),
    ("Doctor's Request", RewardTier::Mid),
    ("Festival Coordination", RewardTier::Late),
];

/// NPC IDs used for Talk quests.
const TALK_NPCS: &[&str] = &[
    "margaret",
    "marco",
    "lily",
    "old_tom",
    "elena",
    "mira",
    "doc",
    "mayor_rex",
    "sam",
    "nora",
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
            "{} wants you to check on {}. Have a chat with them.",
            giver, target_npc
        ),
        1 => format!(
            "{} needs a messenger. Visit {} and have a conversation.",
            giver, target_npc
        ),
        _ => format!("Please speak with {} on behalf of {}.", target_npc, giver),
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

            let next_day = if calendar.day >= DAYS_PER_SEASON {
                1
            } else {
                calendar.day + 1
            };
            let next_season = if calendar.day >= DAYS_PER_SEASON {
                calendar.season.next()
            } else {
                calendar.season
            };
            let next_year =
                if calendar.day >= DAYS_PER_SEASON && matches!(calendar.season, Season::Winter) {
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
            // Resolve display name for use in titles
            let giver_display = npc_registry
                .npcs
                .get(&giver)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| giver.clone());

            let quest = match quest_type {
                0 => {
                    // Deliver quest
                    let tmpl = &DELIVER_TEMPLATES[rng.gen_range(0..DELIVER_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 12);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver_display),
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
                        accepted_day: (next_day, season_to_idx(&next_season), next_year as u16),
                    }
                }
                1 => {
                    // Harvest quest
                    let tmpl = &HARVEST_TEMPLATES[rng.gen_range(0..HARVEST_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 18);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver_display),
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
                        accepted_day: (next_day, season_to_idx(&next_season), next_year as u16),
                    }
                }
                2 => {
                    // Catch fish quest
                    let tmpl = &CATCH_TEMPLATES[rng.gen_range(0..CATCH_TEMPLATES.len())];
                    let gold = scaled_reward(&mut rng, tmpl.2, tmpl.1, 1, 20);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.3, giver_display),
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
                        accepted_day: (next_day, season_to_idx(&next_season), next_year as u16),
                    }
                }
                3 => {
                    // Mine quest
                    let tmpl = &MINE_TEMPLATES[rng.gen_range(0..MINE_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 22);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver_display),
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
                        accepted_day: (next_day, season_to_idx(&next_season), next_year as u16),
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
                        npc_names
                            .first()
                            .cloned()
                            .unwrap_or_else(|| TALK_NPCS[0].to_string())
                    };
                    let talk_tmpl = &TALK_TEMPLATES[rng.gen_range(0..TALK_TEMPLATES.len())];
                    let base_gold = match talk_tmpl.1 {
                        RewardTier::Early => 120,
                        RewardTier::Mid => 340,
                        RewardTier::Late => 580,
                    };
                    let gold = scaled_reward(&mut rng, talk_tmpl.1, base_gold, 1, 5);
                    let target_npc_display = npc_registry
                        .npcs
                        .get(&target_npc)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| target_npc.clone());
                    Quest {
                        id: quest_id,
                        title: format!(
                            "{}: Visit {} for {}",
                            talk_tmpl.0, target_npc_display, giver_display
                        ),
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
                        accepted_day: (next_day, season_to_idx(&next_season), next_year as u16),
                    }
                }
                _ => {
                    // Slay monsters quest
                    let tmpl = &SLAY_TEMPLATES[rng.gen_range(0..SLAY_TEMPLATES.len())];
                    let qty = rng.gen_range(tmpl.1..=tmpl.2);
                    let gold = scaled_reward(&mut rng, tmpl.4, tmpl.3, qty, 25);
                    Quest {
                        id: quest_id,
                        title: format!("{} for {}", tmpl.5, giver_display),
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
                        accepted_day: (next_day, season_to_idx(&next_season), next_year as u16),
                    }
                }
            };

            // Fire event and add to quest log
            quest_posted.send(QuestPostedEvent {
                quest: quest.clone(),
            });
            quest_log.active.push(quest);
            accepted_events.send(QuestAcceptedEvent {
                quest_id: quest_id_clone,
            });
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
/// - `DialogueStartEvent` -> `QuestObjective::Talk` (talking to the NPC)
pub fn track_quest_progress(
    mut crop_events: EventReader<CropHarvestedEvent>,
    mut item_events: EventReader<ItemPickupEvent>,
    mut dialogue_events: EventReader<DialogueStartEvent>,
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

    // --- DialogueStartEvent -> Talk objectives ---
    for event in dialogue_events.read() {
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
        let quest_index = quest_log.active.iter().position(|q| q.id == event.quest_id);

        if let Some(idx) = quest_index {
            let quest = quest_log.active.remove(idx);

            // Award gold (use the amount carried by the event, set at completion time)
            if event.reward_gold > 0 {
                gold_writer.send(GoldChangeEvent {
                    amount: event.reward_gold as i32,
                    reason: format!("Quest completed: {}", quest.title),
                });
            }

            // Award items
            for (item_id, qty) in &quest.reward_items {
                inventory.try_add(item_id, *qty, 99);
            }

            // Award friendship with quest giver
            if quest.reward_friendship != 0 {
                relationships.add_friendship(&quest.giver, quest.reward_friendship as i32);
            }

            // Toast notification
            toast_writer.send(ToastEvent {
                message: format!("Quest complete: {}! +{}g", quest.title, event.reward_gold),
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
                *days -= 1;
                if *days == 0 {
                    expired_titles.push(quest.title.clone());
                    return false; // Remove from active
                }
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

pub fn track_monster_slain(
    mut events: EventReader<MonsterSlainEvent>,
    mut quest_log: ResMut<QuestLog>,
    mut completed_writer: EventWriter<QuestCompletedEvent>,
) {
    let mut newly_completed: Vec<(String, u32)> = Vec::new();

    for event in events.read() {
        for quest in quest_log.active.iter_mut() {
            if let QuestObjective::Slay {
                ref monster_kind,
                quantity,
                ref mut slain,
            } = quest.objective
            {
                if *monster_kind == event.monster_kind && *slain < quantity {
                    *slain += 1;
                    if *slain >= quantity {
                        newly_completed.push((quest.id.clone(), quest.reward_gold));
                    }
                }
            }
        }
    }

    newly_completed.sort_by(|a, b| a.0.cmp(&b.0));
    newly_completed.dedup_by(|a, b| a.0 == b.0);

    for (quest_id, reward_gold) in newly_completed {
        completed_writer.send(QuestCompletedEvent {
            quest_id,
            reward_gold,
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Seasonal quest definitions — hand-crafted, posted on day 1 of each season
// ─────────────────────────────────────────────────────────────────────────────

/// Static objective description for a seasonal quest template.
#[derive(Clone, Copy)]
enum SeasonalObj {
    Deliver { item_id: &'static str, quantity: u8 },
    Harvest { crop_id: &'static str, quantity: u8 },
    Catch { fish_id: &'static str },
    Mine { item_id: &'static str, quantity: u8 },
}

/// A hand-crafted seasonal quest template, posted on day 1 of its season.
struct SeasonalQuestTemplate {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    giver: &'static str,
    objective: SeasonalObj,
    reward_gold: u32,
    reward_items: &'static [(&'static str, u8)],
    reward_friendship: i16,
    /// 0 = Spring, 1 = Summer, 2 = Fall, 3 = Winter
    season_idx: u8,
}

/// 12 hand-crafted seasonal story quests: 3 per season.
/// These are posted automatically on day 1 of their respective season
/// and expire on day 28 (last day of the season).
const SEASONAL_QUESTS: &[SeasonalQuestTemplate] = &[
    // ── SPRING ──────────────────────────────────────────────────────────────
    SeasonalQuestTemplate {
        id: "seasonal_spring_cleanup",
        title: "Spring Cleanup",
        description: "Mayor Rex is organising the annual spring cleanup. He needs 20 wood to repair fences and town structures damaged over winter. Help out and earn a reward!",
        giver: "mayor_rex",
        objective: SeasonalObj::Deliver { item_id: "wood", quantity: 20 },
        reward_gold: 500,
        reward_items: &[],
        reward_friendship: 50,
        season_idx: 0,
    },
    SeasonalQuestTemplate {
        id: "seasonal_first_harvest",
        title: "First Harvest",
        description: "Nora wants to see your first crop of the season. Harvest 5 turnips from your farm and show her the fruits of your labour.",
        giver: "nora",
        objective: SeasonalObj::Harvest { crop_id: "turnip", quantity: 5 },
        reward_gold: 300,
        reward_items: &[],
        reward_friendship: 40,
        season_idx: 0,
    },
    SeasonalQuestTemplate {
        id: "seasonal_fishing_apprentice",
        title: "Fishing Apprentice",
        description: "Sam has taken up fishing and wants a fresh bass to cook for dinner. Catch one from the river and deliver it to him.",
        giver: "sam",
        objective: SeasonalObj::Catch { fish_id: "bass" },
        reward_gold: 400,
        reward_items: &[("fiber", 10)],
        reward_friendship: 45,
        season_idx: 0,
    },
    // ── SUMMER ──────────────────────────────────────────────────────────────
    SeasonalQuestTemplate {
        id: "seasonal_summer_bounty",
        title: "Summer Bounty",
        description: "The summer harvest is in full swing! Lily wants to celebrate by collecting 10 melons for the town market stall. Ship them out for a generous reward.",
        giver: "lily",
        objective: SeasonalObj::Harvest { crop_id: "melon", quantity: 10 },
        reward_gold: 800,
        reward_items: &[],
        reward_friendship: 50,
        season_idx: 1,
    },
    SeasonalQuestTemplate {
        id: "seasonal_beach_cookout",
        title: "Beach Cookout",
        description: "Marco is hosting a summer beach cookout and needs 3 pizzas for the event. Bring them to him before the celebration starts!",
        giver: "marco",
        objective: SeasonalObj::Deliver { item_id: "pizza", quantity: 3 },
        reward_gold: 600,
        reward_items: &[],
        reward_friendship: 80,
        season_idx: 1,
    },
    SeasonalQuestTemplate {
        id: "seasonal_mining_expedition",
        title: "Mining Expedition",
        description: "Elena needs gold ore from the deeper mine levels for a new batch of tools. Dig down and bring back 5 gold ore to prove you can handle the deep mines.",
        giver: "elena",
        objective: SeasonalObj::Mine { item_id: "gold_ore", quantity: 5 },
        reward_gold: 1000,
        reward_items: &[("gold_ore", 5)],
        reward_friendship: 60,
        season_idx: 1,
    },
    // ── FALL ────────────────────────────────────────────────────────────────
    SeasonalQuestTemplate {
        id: "seasonal_harvest_festival_prep",
        title: "Harvest Festival Prep",
        description: "Margaret is baking for the autumn harvest festival and needs 5 pumpkins for her famous pumpkin pies. Deliver them before the festival begins!",
        giver: "margaret",
        objective: SeasonalObj::Deliver { item_id: "pumpkin", quantity: 5 },
        reward_gold: 700,
        reward_items: &[("cake", 1)],
        reward_friendship: 60,
        season_idx: 2,
    },
    SeasonalQuestTemplate {
        id: "seasonal_mushroom_hunt",
        title: "Mushroom Hunt",
        description: "Mira is looking for autumn produce from the forest and farm. Harvest 8 yams — they are perfect for her seasonal trading stock.",
        giver: "mira",
        objective: SeasonalObj::Harvest { crop_id: "yam", quantity: 8 },
        reward_gold: 500,
        reward_items: &[],
        reward_friendship: 50,
        season_idx: 2,
    },
    SeasonalQuestTemplate {
        id: "seasonal_animal_husbandry",
        title: "Animal Husbandry",
        description: "Nora is impressed by well-kept animals. Collect 3 wool from your livestock and deliver it — it shows your animals are healthy and thriving.",
        giver: "nora",
        objective: SeasonalObj::Deliver { item_id: "wool", quantity: 3 },
        reward_gold: 600,
        reward_items: &[],
        reward_friendship: 60,
        season_idx: 2,
    },
    // ── WINTER ──────────────────────────────────────────────────────────────
    SeasonalQuestTemplate {
        id: "seasonal_winter_stockpile",
        title: "Winter Stockpile",
        description: "The cold months are here. Mayor Rex wants to make sure the town is prepared — deliver 20 coal to keep the community warm through winter.",
        giver: "mayor_rex",
        objective: SeasonalObj::Deliver { item_id: "coal", quantity: 20 },
        reward_gold: 1200,
        reward_items: &[],
        reward_friendship: 70,
        season_idx: 3,
    },
    SeasonalQuestTemplate {
        id: "seasonal_community_bundle",
        title: "Community Bundle",
        description: "Elena is assembling a community workshop bundle for next year. Deliver 1 gold bar as the centrepiece of the community's collective effort.",
        giver: "elena",
        objective: SeasonalObj::Deliver { item_id: "gold_bar", quantity: 1 },
        reward_gold: 1500,
        reward_items: &[],
        reward_friendship: 80,
        season_idx: 3,
    },
    SeasonalQuestTemplate {
        id: "seasonal_frozen_lake_fishing",
        title: "Frozen Lake Fishing",
        description: "Old Tom swears the winter fish taste the best. Catch a sturgeon from the frozen lake and bring it to him — he says it is the hardest catch of the year.",
        giver: "old_tom",
        objective: SeasonalObj::Catch { fish_id: "sturgeon" },
        reward_gold: 800,
        reward_items: &[("gold_ore", 2)],
        reward_friendship: 60,
        season_idx: 3,
    },
];

/// Converts a `SeasonalObj` to a concrete `QuestObjective`.
fn seasonal_obj_to_objective(obj: SeasonalObj) -> QuestObjective {
    match obj {
        SeasonalObj::Deliver { item_id, quantity } => QuestObjective::Deliver {
            item_id: item_id.to_string(),
            quantity,
            delivered: 0,
        },
        SeasonalObj::Harvest { crop_id, quantity } => QuestObjective::Harvest {
            crop_id: crop_id.to_string(),
            quantity,
            harvested: 0,
        },
        SeasonalObj::Catch { fish_id } => QuestObjective::Catch {
            fish_id: fish_id.to_string(),
            delivered: false,
        },
        SeasonalObj::Mine { item_id, quantity } => QuestObjective::Mine {
            item_id: item_id.to_string(),
            quantity,
            collected: 0,
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 6b: post_seasonal_quests — posts SEASONAL_QUESTS on day 1 of each season
// ─────────────────────────────────────────────────────────────────────────────

/// On day 1 of each season, posts the 3 hand-crafted seasonal quests for that
/// season. Quests expire on day 28. Duplicates are suppressed via a
/// `Local<Vec<String>>` tracker that stores the quest `id` of every quest
/// already posted (persists for the lifetime of the session, keyed by season
/// and year to allow re-posting in subsequent years).
pub fn post_seasonal_quests(
    calendar: Res<Calendar>,
    mut quest_log: ResMut<QuestLog>,
    mut posted_events: EventWriter<QuestPostedEvent>,
    mut accepted_events: EventWriter<QuestAcceptedEvent>,
    mut tracker: Local<Vec<String>>,
) {
    if calendar.day != 1 {
        return;
    }

    let current_season_idx = season_to_idx(&calendar.season);

    for tmpl in SEASONAL_QUESTS {
        if tmpl.season_idx != current_season_idx {
            continue;
        }

        // Build a per-year tracker key so quests repeat each year
        let tracker_key = format!("{}_y{}", tmpl.id, calendar.year);
        if tracker.contains(&tracker_key) {
            continue;
        }

        // Also skip if the quest id is already active in the log
        if quest_log.active.iter().any(|q| q.id == tmpl.id) {
            continue;
        }

        let quest = Quest {
            id: tmpl.id.to_string(),
            title: tmpl.title.to_string(),
            description: tmpl.description.to_string(),
            giver: tmpl.giver.to_string(),
            objective: seasonal_obj_to_objective(tmpl.objective),
            reward_gold: tmpl.reward_gold,
            reward_items: tmpl
                .reward_items
                .iter()
                .map(|(id, qty)| (id.to_string(), *qty))
                .collect(),
            reward_friendship: tmpl.reward_friendship,
            days_remaining: Some(28),
            accepted_day: (1, current_season_idx, calendar.year as u16),
        };

        tracker.push(tracker_key);
        let quest_id = quest.id.clone();
        posted_events.send(QuestPostedEvent {
            quest: quest.clone(),
        });
        quest_log.active.push(quest);
        accepted_events.send(QuestAcceptedEvent { quest_id });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 7: check_story_quests — hand-crafted Year 1 narrative quests
// ─────────────────────────────────────────────────────────────────────────────

/// Posts hand-crafted story quests on specific days during Year 1.
/// These provide guided progression and introduce the player to game systems
/// and NPCs. 12 total quests: 3 per season.
/// Each quest fires exactly once, tracked by a `Local<Vec<String>>`.
pub fn check_story_quests(
    calendar: Res<Calendar>,
    mut quest_log: ResMut<QuestLog>,
    mut posted_events: EventWriter<QuestPostedEvent>,
    mut accepted_events: EventWriter<QuestAcceptedEvent>,
    mut tracker: Local<Vec<String>>,
) {
    // Only fire during Year 1
    if calendar.year != 1 {
        return;
    }

    let story_id: &str;
    let quest: Quest;

    match (calendar.season, calendar.day) {
        // ═══════════════════════════════════════════════════════════════
        // SPRING — 3 quests
        // ═══════════════════════════════════════════════════════════════

        // Spring Quest 1: "Mayor's Welcome" — Day 2
        (Season::Spring, 2) if !tracker.contains(&"mayors_welcome".to_string()) => {
            story_id = "mayors_welcome";
            quest = Quest {
                id: "story_mayors_welcome".to_string(),
                title: "A Warm Welcome".to_string(),
                description: "Mayor Rex wants to see how you're settling in. Bring him 5 turnips from your farm.".to_string(),
                giver: "mayor_rex".to_string(),
                objective: QuestObjective::Deliver {
                    item_id: "turnip".to_string(),
                    quantity: 5,
                    delivered: 0,
                },
                reward_gold: 300,
                reward_items: Vec::new(),
                reward_friendship: 50,
                days_remaining: Some(10),
                accepted_day: (2, season_to_idx(&Season::Spring), 1),
            };
        }
        // Spring Quest 2: "Elena's Ore Request" — Day 8
        (Season::Spring, 8) if !tracker.contains(&"ore_request".to_string()) => {
            story_id = "ore_request";
            quest = Quest {
                id: "story_ore_request".to_string(),
                title: "Forge Materials".to_string(),
                description: "Elena the blacksmith needs copper ore for a special project. Can you bring her 10 from the mines?".to_string(),
                giver: "elena".to_string(),
                objective: QuestObjective::Deliver {
                    item_id: "copper_ore".to_string(),
                    quantity: 10,
                    delivered: 0,
                },
                reward_gold: 500,
                reward_items: vec![("copper_bar".to_string(), 1)],
                reward_friendship: 50,
                days_remaining: Some(14),
                accepted_day: (8, season_to_idx(&Season::Spring), 1),
            };
        }
        // Spring Quest 3: "Old Tom's Fishing Challenge" — Day 18
        (Season::Spring, 18) if !tracker.contains(&"fishing_challenge".to_string()) => {
            story_id = "fishing_challenge";
            quest = Quest {
                id: "story_fishing_challenge".to_string(),
                title: "Catch of the Day".to_string(),
                description:
                    "Old Tom wants to see if you've got what it takes. Catch a bass and show him!"
                        .to_string(),
                giver: "old_tom".to_string(),
                objective: QuestObjective::Catch {
                    fish_id: "bass".to_string(),
                    delivered: false,
                },
                reward_gold: 400,
                reward_items: Vec::new(),
                reward_friendship: 50,
                days_remaining: Some(7),
                accepted_day: (18, season_to_idx(&Season::Spring), 1),
            };
        }

        // ═══════════════════════════════════════════════════════════════
        // SUMMER — 3 quests
        // ═══════════════════════════════════════════════════════════════

        // Summer Quest 1: "Marco's Summer Recipe" — Day 3
        (Season::Summer, 3) if !tracker.contains(&"summer_recipe".to_string()) => {
            story_id = "summer_recipe";
            quest = Quest {
                id: "story_summer_recipe".to_string(),
                title: "Summer Recipe Challenge".to_string(),
                description: "Marco is perfecting his summer menu. He needs 5 tomatoes and wants you to deliver them fresh from the field.".to_string(),
                giver: "marco".to_string(),
                objective: QuestObjective::Harvest {
                    crop_id: "tomato".to_string(),
                    quantity: 5,
                    harvested: 0,
                },
                reward_gold: 400,
                reward_items: vec![("spaghetti".to_string(), 2)],
                reward_friendship: 60,
                days_remaining: Some(10),
                accepted_day: (3, season_to_idx(&Season::Summer), 1),
            };
        }
        // Summer Quest 2: "Lily's Bouquet Materials" — Day 12
        (Season::Summer, 12) if !tracker.contains(&"bouquet_materials".to_string()) => {
            story_id = "bouquet_materials";
            quest = Quest {
                id: "story_bouquet_materials".to_string(),
                title: "Flowers for the Festival".to_string(),
                description: "Lily needs help gathering materials for the summer festival decorations. Talk to Nora about borrowing some garden shears.".to_string(),
                giver: "lily".to_string(),
                objective: QuestObjective::Talk {
                    npc_name: "nora".to_string(),
                    talked: false,
                },
                reward_gold: 250,
                reward_items: Vec::new(),
                reward_friendship: 40,
                days_remaining: Some(5),
                accepted_day: (12, season_to_idx(&Season::Summer), 1),
            };
        }
        // Summer Quest 3: "Mira's Rare Trade" — Day 22
        (Season::Summer, 22) if !tracker.contains(&"rare_trade".to_string()) => {
            story_id = "rare_trade";
            quest = Quest {
                id: "story_rare_trade".to_string(),
                title: "Exotic Goods Exchange".to_string(),
                description: "Mira the traveling merchant has a special request. She needs 3 gold ore from the deep mines for a lucrative trade deal.".to_string(),
                giver: "mira".to_string(),
                objective: QuestObjective::Mine {
                    item_id: "gold_ore".to_string(),
                    quantity: 3,
                    collected: 0,
                },
                reward_gold: 800,
                reward_items: Vec::new(),
                reward_friendship: 70,
                days_remaining: Some(7),
                accepted_day: (22, season_to_idx(&Season::Summer), 1),
            };
        }

        // ═══════════════════════════════════════════════════════════════
        // FALL — 3 quests
        // ═══════════════════════════════════════════════════════════════

        // Fall Quest 1: "Margaret's Harvest Feast" — Day 4
        (Season::Fall, 4) if !tracker.contains(&"harvest_feast".to_string()) => {
            story_id = "harvest_feast";
            quest = Quest {
                id: "story_harvest_feast".to_string(),
                title: "Harvest Feast Preparation".to_string(),
                description: "Margaret is baking for the fall harvest feast. She needs 8 pumpkins for her famous pumpkin pies.".to_string(),
                giver: "margaret".to_string(),
                objective: QuestObjective::Deliver {
                    item_id: "pumpkin".to_string(),
                    quantity: 8,
                    delivered: 0,
                },
                reward_gold: 600,
                reward_items: vec![("cake".to_string(), 1)],
                reward_friendship: 60,
                days_remaining: Some(12),
                accepted_day: (4, season_to_idx(&Season::Fall), 1),
            };
        }
        // Fall Quest 2: "Doc's Research" — Day 14
        (Season::Fall, 14) if !tracker.contains(&"docs_research".to_string()) => {
            story_id = "docs_research";
            quest = Quest {
                id: "story_docs_research".to_string(),
                title: "Medical Research".to_string(),
                description: "Doc is researching winter remedies and needs a sturgeon for its medicinal properties. Catch one from the mountain lake.".to_string(),
                giver: "doc".to_string(),
                objective: QuestObjective::Catch {
                    fish_id: "sturgeon".to_string(),
                    delivered: false,
                },
                reward_gold: 700,
                reward_items: Vec::new(),
                reward_friendship: 80,
                days_remaining: Some(10),
                accepted_day: (14, season_to_idx(&Season::Fall), 1),
            };
        }
        // Fall Quest 3: "Sam's Autumn Concert" — Day 24
        (Season::Fall, 24) if !tracker.contains(&"autumn_concert".to_string()) => {
            story_id = "autumn_concert";
            quest = Quest {
                id: "story_autumn_concert".to_string(),
                title: "Concert Preparations".to_string(),
                description: "Sam is organizing a fall concert in the town square. He needs you to talk to Mayor Rex about getting a permit.".to_string(),
                giver: "sam".to_string(),
                objective: QuestObjective::Talk {
                    npc_name: "mayor_rex".to_string(),
                    talked: false,
                },
                reward_gold: 350,
                reward_items: Vec::new(),
                reward_friendship: 50,
                days_remaining: Some(4),
                accepted_day: (24, season_to_idx(&Season::Fall), 1),
            };
        }

        // ═══════════════════════════════════════════════════════════════
        // WINTER — 3 quests
        // ═══════════════════════════════════════════════════════════════

        // Winter Quest 1: "Nora's Winter Prep" — Day 3
        (Season::Winter, 3) if !tracker.contains(&"winter_prep".to_string()) => {
            story_id = "winter_prep";
            quest = Quest {
                id: "story_winter_prep".to_string(),
                title: "Winter Preparations".to_string(),
                description: "Nora wants to make sure the farm animals are safe for winter. Deliver 10 wood for reinforcing the barn walls.".to_string(),
                giver: "nora".to_string(),
                objective: QuestObjective::Deliver {
                    item_id: "wood".to_string(),
                    quantity: 10,
                    delivered: 0,
                },
                reward_gold: 350,
                reward_items: Vec::new(),
                reward_friendship: 50,
                days_remaining: Some(7),
                accepted_day: (3, season_to_idx(&Season::Winter), 1),
            };
        }
        // Winter Quest 2: "Elena's Masterwork" — Day 12
        (Season::Winter, 12) if !tracker.contains(&"masterwork".to_string()) => {
            story_id = "masterwork";
            quest = Quest {
                id: "story_masterwork".to_string(),
                title: "The Masterwork".to_string(),
                description: "Elena is crafting a masterwork blade and needs 5 iron bars and 2 gold bars. Help her gather the materials from the mine.".to_string(),
                giver: "elena".to_string(),
                objective: QuestObjective::Mine {
                    item_id: "iron_bar".to_string(),
                    quantity: 5,
                    collected: 0,
                },
                reward_gold: 900,
                reward_items: Vec::new(),
                reward_friendship: 80,
                days_remaining: Some(14),
                accepted_day: (12, season_to_idx(&Season::Winter), 1),
            };
        }
        // Winter Quest 3: "Mayor's Year-End Review" — Day 25
        (Season::Winter, 25) if !tracker.contains(&"year_end_review".to_string()) => {
            story_id = "year_end_review";
            quest = Quest {
                id: "story_year_end_review".to_string(),
                title: "Year-End Celebration".to_string(),
                description: "Mayor Rex wants to celebrate the town's first year with you. Deliver 3 cakes for the winter festival gathering.".to_string(),
                giver: "mayor_rex".to_string(),
                objective: QuestObjective::Deliver {
                    item_id: "cake".to_string(),
                    quantity: 3,
                    delivered: 0,
                },
                reward_gold: 1000,
                reward_items: Vec::new(),
                reward_friendship: 100,
                days_remaining: Some(3),
                accepted_day: (25, season_to_idx(&Season::Winter), 1),
            };
        }

        _ => return,
    }

    // Track, post, and auto-accept the quest
    tracker.push(story_id.to_string());
    let quest_id = quest.id.clone();
    posted_events.send(QuestPostedEvent {
        quest: quest.clone(),
    });
    quest_log.active.push(quest);
    accepted_events.send(QuestAcceptedEvent { quest_id });
}
