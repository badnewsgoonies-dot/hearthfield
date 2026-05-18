//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Map NPC id → unique spritesheet filename in assets/sprites/npcs/.
pub fn npc_sprite_file_helper(npc_id: &str) -> &'static str {
    match npc_id {
        "margaret" => "sprites/npcs/npc_mage.png", // baker — magical with food
        "marco" => "sprites/npcs/npc_traveler.png", // chef — worldly
        "lily" => "sprites/npcs/npc_child.png",     // florist — youthful
        "old_tom" => "sprites/npcs/npc_pirate.png", // fisherman — seafaring
        "elena" => "sprites/npcs/npc_blacksmith.png", // blacksmith
        "mira" => "sprites/npcs/npc_merchant.png",  // traveling merchant
        "doc" => "sprites/npcs/npc_healer.png",     // doctor
        "mayor_rex" => "sprites/npcs/npc_noble.png", // mayor — regal
        "sam" => "sprites/npcs/npc_scholar.png",    // musician — scholarly
        "nora" => "sprites/npcs/npc_farmer.png",    // farmer
        "bjorn" => "sprites/npcs/npc_miner.png",    // carpenter — sturdy build
        _ => "sprites/npcs/npc_guard.png",          // fallback
    }
}

