//! Festival events for Hearthfield.
//!
//! Four seasonal festivals with unique gameplay:
//! - Egg Festival (Spring 13): Timed egg-collection minigame on the Farm.
//! - Luau (Summer 11): Contribute an item to the communal soup on the Beach.
//! - Harvest Festival (Fall 16): Submit a crop for judging in Town.
//! - Winter Star (Winter 25): Gift exchange with a randomly assigned NPC.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Identifies which festival is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FestivalKind {
    EggFestival,     // Spring 13
    Luau,            // Summer 11
    HarvestFestival, // Fall 16
    WinterStar,      // Winter 25
}

/// Tracks the currently active festival (if any) and its progress.
#[derive(Resource, Debug, Clone, Default)]
pub struct FestivalState {
    pub active: Option<FestivalKind>,
    pub started: bool,
    pub timer: Option<Timer>,
    pub score: u32,
    pub items_collected: u32,
    /// For Winter Star: the NPC assigned as the player's gift recipient.
    pub winter_star_recipient: Option<NpcId>,
    /// For Winter Star: the NPC who will give the player a gift.
    pub winter_star_giver: Option<NpcId>,
    /// Tracks whether the festival announcement toast was already sent
    /// for the current day so we don't spam.
    pub announced_day: Option<(Season, u8, u32)>,
}

/// Marker component for egg entities spawned during the Egg Festival.
#[derive(Component, Debug, Clone)]
pub struct FestivalEgg;

// ═══════════════════════════════════════════════════════════════════════
// HELPER — map (Season, day) to FestivalKind
// ═══════════════════════════════════════════════════════════════════════

fn festival_for_date(season: Season, day: u8) -> Option<FestivalKind> {
    match (season, day) {
        (Season::Spring, 13) => Some(FestivalKind::EggFestival),
        (Season::Summer, 11) => Some(FestivalKind::Luau),
        (Season::Fall, 16) => Some(FestivalKind::HarvestFestival),
        (Season::Winter, 25) => Some(FestivalKind::WinterStar),
        _ => None,
    }
}

fn festival_display_name(kind: FestivalKind) -> &'static str {
    match kind {
        FestivalKind::EggFestival => "Egg Festival",
        FestivalKind::Luau => "Luau",
        FestivalKind::HarvestFestival => "Harvest Festival",
        FestivalKind::WinterStar => "Winter Star Festival",
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: check_festival_day
// ═══════════════════════════════════════════════════════════════════════

/// Runs every frame in Playing.  If the current calendar date matches a
/// festival and we haven't announced it yet, set `FestivalState.active`
/// and send a toast.  If the date no longer matches (day advanced),
/// clear the festival state.
pub fn check_festival_day(
    calendar: Res<Calendar>,
    mut festival: ResMut<FestivalState>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    let today = (calendar.season, calendar.day, calendar.year);

    if let Some(kind) = festival_for_date(calendar.season, calendar.day) {
        // Set the active festival if not already set.
        if festival.active.is_none() {
            festival.active = Some(kind);
            festival.started = false;
            festival.score = 0;
            festival.items_collected = 0;
            festival.timer = None;
        }

        // Send announcement toast once per day.
        if festival.announced_day != Some(today) {
            festival.announced_day = Some(today);
            let name = festival_display_name(kind);
            toast_writer.send(ToastEvent {
                message: format!("Today is the {}! Press E to participate.", name),
                duration_secs: 5.0,
            });
            info!("[Festivals] Announced {} on Day {} {:?}", name, calendar.day, calendar.season);
        }
    } else {
        // Not a festival day — reset state if it was active.
        if festival.active.is_some() {
            festival.active = None;
            festival.started = false;
            festival.score = 0;
            festival.items_collected = 0;
            festival.timer = None;
            festival.winter_star_recipient = None;
            festival.winter_star_giver = None;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: start_egg_hunt  (Spring 13, Farm map, press E)
// ═══════════════════════════════════════════════════════════════════════

/// When the player presses E on the Farm during the Egg Festival and
/// the hunt hasn't started yet, spawn 20 collectible eggs and start a
/// 30-second timer.
pub fn start_egg_hunt(
    player_input: Res<PlayerInput>,
    mut festival: ResMut<FestivalState>,
    player_state: Res<PlayerState>,
    mut commands: Commands,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    // Guard: must be Egg Festival, not yet started, player on Farm, pressing E.
    if festival.active != Some(FestivalKind::EggFestival) {
        return;
    }
    if festival.started {
        return;
    }
    if player_state.current_map != MapId::Farm {
        return;
    }
    if !player_input.interact {
        return;
    }

    festival.started = true;
    festival.score = 0;
    festival.items_collected = 0;
    festival.timer = Some(Timer::from_seconds(30.0, TimerMode::Once));

    // Spawn 20 egg entities at random positions within the farm area.
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-8..8) as f32 * TILE_SIZE;
        let y = rng.gen_range(-8..8) as f32 * TILE_SIZE;

        commands.spawn((
            FestivalEgg,
            Sprite {
                color: Color::srgb(1.0, 0.95, 0.2), // bright yellow
                custom_size: Some(Vec2::new(6.0, 6.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 5.0)),
        ));
    }

    toast_writer.send(ToastEvent {
        message: "Egg Hunt! Collect eggs in 30 seconds!".into(),
        duration_secs: 4.0,
    });

    info!("[Festivals] Egg Hunt started — 20 eggs spawned.");
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: collect_eggs  (runs while egg hunt is active)
// ═══════════════════════════════════════════════════════════════════════

/// Checks player proximity to each egg entity; despawns collected eggs
/// and awards the player when the timer expires.
pub fn collect_eggs(
    time: Res<Time>,
    mut festival: ResMut<FestivalState>,
    player_query: Query<&Transform, With<Player>>,
    egg_query: Query<(Entity, &Transform), With<FestivalEgg>>,
    mut commands: Commands,
    mut toast_writer: EventWriter<ToastEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
) {
    // Only run if egg hunt is active.
    if festival.active != Some(FestivalKind::EggFestival) || !festival.started {
        return;
    }
    if festival.timer.is_none() {
        return;
    }

    // Tick the timer, then immediately read whether it finished, before
    // doing anything else with `festival`.  This avoids overlapping
    // mutable borrows.
    festival.timer.as_mut().unwrap().tick(time.delta());
    let timer_finished = festival.timer.as_ref().unwrap().just_finished();

    // Check player proximity to eggs.
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation.truncate();

        for (entity, egg_transform) in egg_query.iter() {
            let egg_pos = egg_transform.translation.truncate();
            let distance = player_pos.distance(egg_pos);

            if distance < TILE_SIZE {
                commands.entity(entity).despawn();
                festival.items_collected += 1;

                sfx_writer.send(PlaySfxEvent {
                    sfx_id: "item_pickup".into(),
                });
            }
        }
    }

    // Check if timer expired.
    if timer_finished {
        let collected = festival.items_collected;

        toast_writer.send(ToastEvent {
            message: format!("You found {} eggs!", collected),
            duration_secs: 4.0,
        });

        // Prize: finding 15 or more eggs awards a rare seed.
        if collected >= 15 {
            pickup_writer.send(ItemPickupEvent {
                item_id: "rare_seed".into(),
                quantity: 1,
            });
            toast_writer.send(ToastEvent {
                message: "Amazing! You won a Rare Seed as a prize!".into(),
                duration_secs: 4.0,
            });
        }

        // Despawn any remaining eggs.
        for (entity, _) in egg_query.iter() {
            commands.entity(entity).despawn();
        }

        // End festival.
        festival.started = false;
        festival.timer = None;

        info!(
            "[Festivals] Egg Hunt ended — {} eggs collected.",
            collected
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: start_luau  (Summer 11, Beach map, press E)
// ═══════════════════════════════════════════════════════════════════════

/// The player contributes their currently selected hotbar item to the
/// communal luau soup.  Quality determines NPC friendship gain.
pub fn start_luau(
    player_input: Res<PlayerInput>,
    mut festival: ResMut<FestivalState>,
    player_state: Res<PlayerState>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut relationships: ResMut<Relationships>,
    npc_registry: Res<NpcRegistry>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    if festival.active != Some(FestivalKind::Luau) {
        return;
    }
    if festival.started {
        return;
    }
    if player_state.current_map != MapId::Beach {
        return;
    }
    if !player_input.interact {
        return;
    }

    // Check selected hotbar item.
    let slot_index = inventory.selected_slot;
    let item_info = inventory.slots[slot_index].as_ref().map(|slot| {
        (slot.item_id.clone(), slot.quantity)
    });

    let (item_id, _qty) = match item_info {
        Some(info) => info,
        None => {
            toast_writer.send(ToastEvent {
                message: "You need to hold an item to contribute to the soup!".into(),
                duration_secs: 3.0,
            });
            return;
        }
    };

    // Remove one of the item from inventory.
    inventory.try_remove(&item_id, 1);

    // Evaluate quality based on item sell price.
    let sell_price = item_registry
        .get(&item_id)
        .map(|def| def.sell_price)
        .unwrap_or(0);

    // Determine quality tier from sell price.
    let (message, friendship_gain) = if sell_price >= 300 {
        (
            "The governor loved it! Everyone is impressed!",
            200i32, // +2 hearts
        )
    } else if sell_price >= 100 {
        (
            "Very tasty! The town enjoyed the soup.",
            100i32, // +1 heart
        )
    } else {
        (
            "Not bad. The soup was decent.",
            0i32,
        )
    };

    // Apply friendship to all registered NPCs.
    if friendship_gain > 0 {
        for npc_id in npc_registry.npcs.keys() {
            relationships.add_friendship(npc_id, friendship_gain);
        }
    }

    toast_writer.send(ToastEvent {
        message: message.into(),
        duration_secs: 5.0,
    });

    sfx_writer.send(PlaySfxEvent {
        sfx_id: "festival_complete".into(),
    });

    festival.started = true; // Mark as participated so player can't re-enter.
    info!(
        "[Festivals] Luau completed — contributed '{}', friendship gain: {}",
        item_id, friendship_gain
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: start_harvest_festival  (Fall 16, Town map, press E)
// ═══════════════════════════════════════════════════════════════════════

/// The player submits a crop item for judging.  Score is calculated from
/// the crop's sell price.  If the score beats a threshold the player
/// wins a gold prize.
pub fn start_harvest_festival(
    player_input: Res<PlayerInput>,
    mut festival: ResMut<FestivalState>,
    player_state: Res<PlayerState>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    if festival.active != Some(FestivalKind::HarvestFestival) {
        return;
    }
    if festival.started {
        return;
    }
    if player_state.current_map != MapId::Town {
        return;
    }
    if !player_input.interact {
        return;
    }

    // Check selected hotbar item.
    let slot_index = inventory.selected_slot;
    let item_info = inventory.slots[slot_index].as_ref().map(|slot| {
        (slot.item_id.clone(), slot.quantity)
    });

    let (item_id, _qty) = match item_info {
        Some(info) => info,
        None => {
            toast_writer.send(ToastEvent {
                message: "Hold a crop to enter the Harvest Festival judging!".into(),
                duration_secs: 3.0,
            });
            return;
        }
    };

    // Only crops should be submittable.
    let item_def = item_registry.get(&item_id);
    let (sell_price, is_crop) = match item_def {
        Some(def) => (def.sell_price, def.category == ItemCategory::Crop),
        None => (0, false),
    };

    if !is_crop {
        toast_writer.send(ToastEvent {
            message: "You need to submit a crop for judging!".into(),
            duration_secs: 3.0,
        });
        return;
    }

    // Remove one from inventory.
    inventory.try_remove(&item_id, 1);

    // Score based on sell price.  A quality multiplier would enhance this
    // if the QualityStack system is in use; for now we use base sell price.
    let score = sell_price;
    festival.score = score;

    // Threshold to win: sell price >= 150.
    let prize_threshold = 150;
    let won = score >= prize_threshold;

    if won {
        toast_writer.send(ToastEvent {
            message: format!(
                "Your {} scored {} points! You win first place and 500g!",
                item_def.map(|d| d.name.as_str()).unwrap_or("crop"),
                score
            ),
            duration_secs: 5.0,
        });
        gold_writer.send(GoldChangeEvent {
            amount: 500,
            reason: "Harvest Festival prize".into(),
        });
    } else {
        toast_writer.send(ToastEvent {
            message: format!(
                "Your {} scored {} points. Not quite enough to win this year.",
                item_def.map(|d| d.name.as_str()).unwrap_or("crop"),
                score
            ),
            duration_secs: 5.0,
        });
    }

    sfx_writer.send(PlaySfxEvent {
        sfx_id: "festival_complete".into(),
    });

    festival.started = true;
    info!(
        "[Festivals] Harvest Festival — submitted '{}', score: {}, won: {}",
        item_id, score, won
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: setup_winter_star  (Winter 25, assign gift recipient/giver)
// ═══════════════════════════════════════════════════════════════════════

/// On Winter 25, if no recipient has been assigned yet, pick a random
/// NPC as the player's gift recipient and another as the giver.
pub fn setup_winter_star(
    mut festival: ResMut<FestivalState>,
    npc_registry: Res<NpcRegistry>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    if festival.active != Some(FestivalKind::WinterStar) {
        return;
    }
    // Only assign once.
    if festival.winter_star_recipient.is_some() {
        return;
    }

    let npc_ids: Vec<String> = npc_registry.npcs.keys().cloned().collect();
    if npc_ids.is_empty() {
        // No NPCs registered yet — nothing to do.
        return;
    }

    let mut rng = rand::thread_rng();
    let recipient_index = rng.gen_range(0..npc_ids.len());
    let recipient = npc_ids[recipient_index].clone();

    // Pick a different NPC as the giver (or the same if only one NPC).
    let giver = if npc_ids.len() > 1 {
        let mut giver_index = rng.gen_range(0..npc_ids.len());
        while giver_index == recipient_index {
            giver_index = rng.gen_range(0..npc_ids.len());
        }
        npc_ids[giver_index].clone()
    } else {
        npc_ids[0].clone()
    };

    let recipient_name = npc_registry
        .npcs
        .get(&recipient)
        .map(|def| def.name.as_str())
        .unwrap_or("someone");

    festival.winter_star_recipient = Some(recipient.clone());
    festival.winter_star_giver = Some(giver.clone());

    toast_writer.send(ToastEvent {
        message: format!(
            "Winter Star: You've been assigned to give a gift to {}!",
            recipient_name
        ),
        duration_secs: 5.0,
    });

    info!(
        "[Festivals] Winter Star assigned — recipient: {}, giver: {}",
        recipient, giver
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: winter_star_give_gift  (Winter 25, Town map, press E)
// ═══════════════════════════════════════════════════════════════════════

/// When the player presses E in Town on Winter 25, the held item is
/// given to the assigned NPC.  The player then receives a random gift
/// from their assigned giver NPC.
pub fn winter_star_give_gift(
    player_input: Res<PlayerInput>,
    mut festival: ResMut<FestivalState>,
    player_state: Res<PlayerState>,
    mut inventory: ResMut<Inventory>,
    npc_registry: Res<NpcRegistry>,
    mut relationships: ResMut<Relationships>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    if festival.active != Some(FestivalKind::WinterStar) {
        return;
    }
    if festival.started {
        return; // Already participated.
    }
    if player_state.current_map != MapId::Town {
        return;
    }
    if !player_input.interact {
        return;
    }

    let recipient_id = match &festival.winter_star_recipient {
        Some(id) => id.clone(),
        None => return, // Not set up yet.
    };
    let giver_id = match &festival.winter_star_giver {
        Some(id) => id.clone(),
        None => return,
    };

    // Check that the player is holding an item.
    let slot_index = inventory.selected_slot;
    let item_info = inventory.slots[slot_index].as_ref().map(|slot| {
        slot.item_id.clone()
    });

    let item_id = match item_info {
        Some(id) => id,
        None => {
            toast_writer.send(ToastEvent {
                message: "Hold an item to give as your Winter Star gift!".into(),
                duration_secs: 3.0,
            });
            return;
        }
    };

    // Remove one of the gift item from inventory.
    inventory.try_remove(&item_id, 1);

    let recipient_name = npc_registry
        .npcs
        .get(&recipient_id)
        .map(|def| def.name.as_str())
        .unwrap_or("your friend");

    let giver_name = npc_registry
        .npcs
        .get(&giver_id)
        .map(|def| def.name.as_str())
        .unwrap_or("A friend");

    // Giving a gift always adds +2 hearts to the recipient.
    relationships.add_friendship(&recipient_id, 200);

    toast_writer.send(ToastEvent {
        message: format!("You gave your gift to {}. They look happy!", recipient_name),
        duration_secs: 4.0,
    });

    // The player receives a random gift from their secret giver.
    // Pick from a list of nice winter gifts.
    let winter_gifts = [
        ("gold_bar", "a Gold Bar"),
        ("pumpkin_soup", "Pumpkin Soup"),
        ("ruby", "a Ruby"),
        ("wool", "Wool"),
        ("honey", "Honey"),
    ];
    let mut rng = rand::thread_rng();
    let (gift_item, gift_name) = winter_gifts[rng.gen_range(0..winter_gifts.len())];

    pickup_writer.send(ItemPickupEvent {
        item_id: gift_item.into(),
        quantity: 1,
    });

    toast_writer.send(ToastEvent {
        message: format!("{} gave you {}! Happy Winter Star!", giver_name, gift_name),
        duration_secs: 5.0,
    });

    sfx_writer.send(PlaySfxEvent {
        sfx_id: "festival_complete".into(),
    });

    festival.started = true;
    info!(
        "[Festivals] Winter Star exchange — gave '{}' to {}, received '{}' from {}",
        item_id, recipient_id, gift_item, giver_id
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM: cleanup_festival_on_day_end
// ═══════════════════════════════════════════════════════════════════════

/// When a DayEndEvent fires, reset the festival state so the next day
/// starts fresh.  Also despawns any leftover egg entities.
pub fn cleanup_festival_on_day_end(
    mut day_end_reader: EventReader<DayEndEvent>,
    mut festival: ResMut<FestivalState>,
    egg_query: Query<Entity, With<FestivalEgg>>,
    mut commands: Commands,
) {
    for _event in day_end_reader.read() {
        if festival.active.is_some() {
            info!("[Festivals] Day ended — cleaning up festival state.");
            festival.active = None;
            festival.started = false;
            festival.timer = None;
            festival.score = 0;
            festival.items_collected = 0;
            festival.winter_star_recipient = None;
            festival.winter_star_giver = None;

            // Despawn any leftover eggs.
            for entity in egg_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}
