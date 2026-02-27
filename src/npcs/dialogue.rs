//! Dialogue system: handle player interaction with NPCs, build dialogue lines
//! based on friendship level, and emit DialogueStartEvent.

use bevy::prelude::*;
use crate::shared::*;

/// Component marking an NPC that the player is adjacent to and can interact with.
#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct NpcInteractable;

/// Resource tracking the last-interacted NPC (for gift-giving context).
#[derive(Resource, Debug, Default)]
pub struct ActiveNpcInteraction {
    pub npc_id: Option<String>,
}

/// System: detect player pressing Space near an NPC and start dialogue.
pub fn handle_npc_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(&Npc, &Transform)>,
    relationships: Res<Relationships>,
    npc_registry: Res<NpcRegistry>,
    calendar: Res<Calendar>,
    mut dialogue_writer: EventWriter<DialogueStartEvent>,
    mut active_interaction: ResMut<ActiveNpcInteraction>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Only check interaction during Playing state
    if *current_state.get() != GameState::Playing {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let interaction_range = TILE_SIZE * PIXEL_SCALE * 1.5; // ~72px in world space

    // Find the closest adjacent NPC within range
    let mut closest: Option<(&Npc, f32)> = None;
    for (npc, npc_transform) in npc_query.iter() {
        let npc_pos = npc_transform.translation.truncate();
        let dist = player_pos.distance(npc_pos);
        if dist <= interaction_range {
            match closest {
                None => closest = Some((npc, dist)),
                Some((_, best_dist)) if dist < best_dist => closest = Some((npc, dist)),
                _ => {}
            }
        }
    }

    let Some((npc, _)) = closest else {
        return;
    };

    let npc_id = &npc.id;
    let hearts = relationships.hearts(npc_id);
    let lines = build_dialogue_lines(npc_id, hearts, &npc_registry, &relationships, &calendar);

    let portrait_index = npc_registry.npcs.get(npc_id)
        .map(|def| def.portrait_index);

    active_interaction.npc_id = Some(npc_id.clone());

    dialogue_writer.send(DialogueStartEvent {
        npc_id: npc_id.clone(),
        lines,
        portrait_index,
    });

    next_state.set(GameState::Dialogue);
}

/// Build dialogue lines for an NPC at the given friendship tier.
/// Falls back through lower tiers to always provide content.
/// Also prepends contextual lines based on weather and season.
pub fn build_dialogue_lines(
    npc_id: &str,
    hearts: u8,
    npc_registry: &NpcRegistry,
    relationships: &Relationships,
    calendar: &Calendar,
) -> Vec<String> {
    let Some(npc_def) = npc_registry.npcs.get(npc_id) else {
        return vec!["...".to_string()];
    };

    let mut lines = Vec::new();

    // --- Contextual: "already gifted today" notice ---
    if relationships.gifted_today.get(npc_id).copied().unwrap_or(false) {
        lines.push(
            "I really appreciate everything you've given me today. Come back tomorrow!".to_string(),
        );
        return lines;
    }

    // --- Contextual: birthday greeting (checked here with real calendar) ---
    let is_birthday =
        calendar.season == npc_def.birthday_season && calendar.day == npc_def.birthday_day;
    if is_birthday {
        lines.push(format!(
            "Oh! Today is my birthday! I can't believe you remembered, {}!",
            npc_def.name
        ));
    }

    // --- Contextual: weather comment (prepend one line) ---
    let weather_line = npc_weather_comment(npc_id, calendar.weather);
    if let Some(wl) = weather_line {
        lines.push(wl);
    }

    // --- Contextual: seasonal comment ---
    let season_line = npc_season_comment(npc_id, calendar.season);
    if let Some(sl) = season_line {
        lines.push(sl);
    }

    let tier = friendship_tier(hearts);

    // Get tier-specific dialogue if available
    if let Some(tier_lines) = npc_def.heart_dialogue.get(&tier) {
        if !tier_lines.is_empty() {
            // Pick pseudo-randomly based on total friendship to vary lines day-to-day
            let total_pts = relationships.friendship.get(npc_id).copied().unwrap_or(0);
            let idx = (total_pts as usize / 5) % tier_lines.len();
            lines.push(tier_lines[idx].clone());
            // Add a second line if available
            if tier_lines.len() > 1 {
                let idx2 = (idx + 1) % tier_lines.len();
                lines.push(tier_lines[idx2].clone());
            }
            return lines;
        }
    }

    // Fall back through lower tiers
    let lower_tiers = if tier > 0 {
        (0..tier).rev().collect::<Vec<_>>()
    } else {
        vec![]
    };

    for lower_tier in lower_tiers {
        if let Some(tier_lines) = npc_def.heart_dialogue.get(&lower_tier) {
            if !tier_lines.is_empty() {
                lines.push(tier_lines[0].clone());
                return lines;
            }
        }
    }

    // Ultimate fallback: default dialogue
    if !npc_def.default_dialogue.is_empty() {
        let total_pts = relationships.friendship.get(npc_id).copied().unwrap_or(0);
        let idx = (total_pts as usize) % npc_def.default_dialogue.len();
        lines.push(npc_def.default_dialogue[idx].clone());
    } else {
        lines.push(format!("Hello there! I'm {}.", npc_def.name));
    }

    lines
}

/// Return a weather-aware comment for the given NPC, or None if no comment is warranted.
fn npc_weather_comment(npc_id: &str, weather: Weather) -> Option<String> {
    match weather {
        Weather::Sunny => None, // sunny is default — no special comment
        Weather::Rainy => {
            let line = match npc_id {
                "mayor_thomas"   => "Wet day today! Good for the crops, I suppose.",
                "elena"          => "The rain has kept the shop quiet. I like the sound it makes on the roof, honestly.",
                "marcus"         => "Rainy days mean fewer customers. I use the time to sharpen the tools.",
                "dr_iris"        => "Stay dry out there! Wet feet lead to all sorts of trouble.",
                "old_pete"       => "Rain's good fishing weather, if you know where to look.",
                "chef_rosa"      => "Rain days are soup days. Come by the inn later — I'm making chowder.",
                "miner_gil"      => "Rain keeps me out of the mountains. Might as well dig inside.",
                "librarian_faye" => "A perfect reading day. I almost don't mind the clouds.",
                "farmer_dale"    => "Good steady rain. The fields will thank us for it.",
                "child_lily"     => "It's RAINING! I jumped in three puddles already!",
                _                => "Wet day, isn't it? Hope you brought an umbrella.",
            };
            Some(line.to_string())
        }
        Weather::Stormy => {
            let line = match npc_id {
                "mayor_thomas"   => "Nasty storm rolling in. Please be careful out there.",
                "elena"          => "This storm has me worried. Please don't stay out too long.",
                "marcus"         => "A storm like this could bring down a tree. Watch your head.",
                "dr_iris"        => "In this weather? You're lucky I'm open! Please be safe.",
                "old_pete"       => "I'd stay off the water today. That storm's no joke.",
                "chef_rosa"      => "I closed the shutters and lit the fire. Come in if you need to warm up!",
                "miner_gil"      => "Even I stay out of the mine when lightning's this close.",
                "librarian_faye" => "The storm knocked my shutters open! Three books got damp. I'm devastated.",
                "farmer_dale"    => "Storm like this can flatten crops. Hope you've got some shelter built.",
                "child_lily"     => "Mama said I can't go outside but the thunder is SO COOL.",
                _                => "Quite a storm today, eh? You should head inside soon.",
            };
            Some(line.to_string())
        }
        Weather::Snowy => {
            let line = match npc_id {
                "mayor_thomas"   => "First snow of winter! Hearthfield looks like a painting today.",
                "elena"          => "Snow! The shop window looks magical with the flakes drifting by.",
                "marcus"         => "Snow days are slow days. I use the time to plan spring inventory.",
                "dr_iris"        => "Bundle up! Frostbite is no laughing matter.",
                "old_pete"       => "Ice fishing season. The pond freezes just right by the old bridge.",
                "chef_rosa"      => "Snow means hot cocoa weather. I'll put a pot on.",
                "miner_gil"      => "Doesn't matter what the sky does — it's always the same temperature down in the mine.",
                "librarian_faye" => "Snow makes everything so quiet. I could read all day.",
                "farmer_dale"    => "Snow cover actually protects the winter crops. Nature's blanket.",
                "child_lily"     => "SNOW!! Can you build a snowman with me later?!",
                _                => "Snow's here! Stay warm out there.",
            };
            Some(line.to_string())
        }
    }
}

/// Return a season-aware comment for the given NPC, or None if no comment is warranted.
/// To keep dialogue from feeling repetitive, only return something for specific season/NPC combos.
fn npc_season_comment(npc_id: &str, season: Season) -> Option<String> {
    let line = match (npc_id, season) {
        // Spring
        ("elena", Season::Spring) =>
            Some("Spring is my favourite time of year. Everything is blooming!"),
        ("farmer_dale", Season::Spring) =>
            Some("Time to get those seeds in the ground. Spring waits for no one."),
        ("child_lily", Season::Spring) =>
            Some("Spring means the Egg Festival is coming! I can't wait!!"),

        // Summer
        ("old_pete", Season::Summer) =>
            Some("Summer's when the big fish come in. Best season for the rod, no question."),
        ("chef_rosa", Season::Summer) =>
            Some("Summer produce is just bursting with flavour. Best season to cook."),
        ("mayor_thomas", Season::Summer) =>
            Some("The town square festival is my proudest achievement as mayor. Still months of planning every year."),

        // Fall
        ("farmer_dale", Season::Fall) =>
            Some("Harvest time. This is what all that spring planting was for."),
        ("librarian_faye", Season::Fall) =>
            Some("Fall colours through the library window. It's almost too pretty to concentrate."),
        ("miner_gil", Season::Fall) =>
            Some("The mine runs deep this time of year. Some say the rocks change with the seasons. I believe it."),

        // Winter
        ("old_pete", Season::Winter) =>
            Some("Not much fishing to be had above ground. Good time to mend the nets."),
        ("dr_iris", Season::Winter) =>
            Some("Winter's my busiest season — everyone catches cold. Please dress warmly!"),
        ("marcus", Season::Winter) =>
            Some("Winter gives me time to practice new techniques at the forge. Quiet work is good work."),

        _ => None,
    };
    line.map(|s| s.to_string())
}

/// Get the dialogue tier key for a given heart count.
/// Tiers: 0 (0-2 hearts), 3 (3-5 hearts), 6 (6-8 hearts), 9 (9-10 hearts)
fn friendship_tier(hearts: u8) -> u8 {
    match hearts {
        0..=2 => 0,
        3..=5 => 3,
        6..=8 => 6,
        _ => 9,
    }
}

/// Build gift-response dialogue lines based on the NPC and gift preference.
pub fn build_gift_response_lines(
    npc_id: &str,
    _npc_name: &str,
    preference: GiftPreference,
    item_name: &str,
    is_birthday: bool,
) -> Vec<String> {
    let birthday_prefix = if is_birthday {
        format!("Oh! Is it really for me? And on my birthday! ")
    } else {
        String::new()
    };

    match preference {
        GiftPreference::Loved => {
            let response = loved_response(npc_id, item_name);
            vec![format!("{}{}", birthday_prefix, response)]
        }
        GiftPreference::Liked => {
            vec![
                format!("{}{} — oh, I really like this! Thank you so much.", birthday_prefix, item_name),
            ]
        }
        GiftPreference::Neutral => {
            vec![
                format!("{}Oh, a gift! Thank you for thinking of me. That's very kind.", birthday_prefix),
            ]
        }
        GiftPreference::Disliked => {
            vec![
                format!("{}I appreciate the thought, but... a {}? Hmm. Thank you anyway.", birthday_prefix, item_name),
            ]
        }
        GiftPreference::Hated => {
            let response = hated_response(npc_id, item_name);
            vec![response]
        }
    }
}

fn loved_response(npc_id: &str, item_name: &str) -> String {
    match npc_id {
        "mayor_thomas" => format!("A {}! I've been looking for one like this for ages. You have exceptional taste, my friend.", item_name),
        "elena" => format!("A {}! Oh, it's absolutely beautiful! I'll press this one into my favorite book.", item_name),
        "marcus" => format!("Now THIS is a proper gift. A {}. I can feel the quality just holding it. Thank you.", item_name),
        "dr_iris" => format!("Wonderful! Do you know how rare {} is around here? This will be very useful. Thank you.", item_name),
        "old_pete" => format!("Ha! A {}! You really know the way to an old fisherman's heart. Much appreciated.", item_name),
        "chef_rosa" => format!("A {}! Oh, the things I could make with this... you'll have to come try the dish when it's ready!", item_name),
        "miner_gil" => format!("Now we're talking! A quality {}. You've got a good eye. I appreciate this more than you know.", item_name),
        "librarian_faye" => format!("A {}! Do you realize how significant this is? I've only read about these! Thank you, truly.", item_name),
        "farmer_dale" => format!("A {}! I haven't seen one this fine in years. You're growing into quite the farmer — and gift-giver!", item_name),
        "child_lily" => format!("A {}!! THIS IS THE BEST THING EVER!! Can I keep it? Can I? THANK YOU!!", item_name),
        _ => format!("Oh, a {}! This is exactly what I wanted! Thank you so much!", item_name),
    }
}

fn hated_response(npc_id: &str, item_name: &str) -> String {
    match npc_id {
        "mayor_thomas" => format!("A {}...? I... well. I'm sure you meant well.", item_name),
        "elena" => format!("Oh. A {}. I... don't really know what to do with this. But thank you for the thought.", item_name),
        "marcus" => format!("A {}? What am I supposed to do with that? ...I'll figure something out.", item_name),
        "dr_iris" => format!("A {}. Hmm. I can't say this is medically advisable, but... thank you?", item_name),
        "old_pete" => format!("A {}? Boy, I've pulled stranger things out of the water. ...I'll pass.", item_name),
        "chef_rosa" => format!("A {}... in MY kitchen? I... no. No, I don't think so. But thank you.", item_name),
        "miner_gil" => format!("A {}? I've seen things like this at the bottom of the mine. Left 'em there too.", item_name),
        "librarian_faye" => format!("A {}? I... must admit I'm not sure this belongs in a library. Or anywhere near me.", item_name),
        "farmer_dale" => format!("A {}? Hmph. Been farming sixty years and never needed one of those.", item_name),
        "child_lily" => format!("Ewwww! A {}! I don't want THAT! ...but okay fine I'll keep it.", item_name),
        _ => format!("Oh. A {}. I... see. Thank you for thinking of me.", item_name),
    }
}
