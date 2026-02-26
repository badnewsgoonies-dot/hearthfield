//! Dialogue system: handle player interaction with NPCs, build dialogue lines
//! based on friendship level, and emit DialogueStartEvent.

use bevy::prelude::*;
use crate::shared::*;

/// Component marking an NPC that the player is adjacent to and can interact with.
#[derive(Component, Debug)]
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
    let lines = build_dialogue_lines(npc_id, hearts, &npc_registry, &relationships);

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
pub fn build_dialogue_lines(
    npc_id: &str,
    hearts: u8,
    npc_registry: &NpcRegistry,
    relationships: &Relationships,
) -> Vec<String> {
    let Some(npc_def) = npc_registry.npcs.get(npc_id) else {
        return vec!["...".to_string()];
    };

    let mut lines = Vec::new();

    // Check if today is NPC's birthday (use a simple marker in the line)
    // The calendar check would need injecting — we annotate the line instead
    // In a full system, CalendarRes would be passed in
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
