//! Mining domain plugin for Hearthfield.
//!
//! Provides:
//! - Procedural mine floor generation (20 floors, rocks, enemies, ladder)
//! - Rock breaking with pickaxe (scaled by tool tier)
//! - Combat with mine monsters (slimes, bats, rock crabs)
//! - Enemy AI movement and attack
//! - Ladder discovery and floor descent
//! - Elevator system (every 5 floors)
//! - Mine HUD (floor indicator, elevator selection)
//! - Mine entry/exit via MapTransitionEvent
//! - Player knockout on death (gold penalty, return to surface)
//! - Day-end handling (pass out penalty)

mod combat;
mod components;
mod floor_gen;
mod hud;
mod ladder;
mod movement;
mod rock_breaking;
mod spawning;
mod transitions;

use bevy::prelude::*;
use crate::shared::*;
use components::*;
use movement::MineMoveCooldown;

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        // Mining-local resources
        app.init_resource::<ActiveFloor>();
        app.init_resource::<FloorSpawnRequest>();
        app.init_resource::<InMine>();
        app.init_resource::<PlayerIFrames>();
        app.init_resource::<MineMoveCooldown>();
        app.init_resource::<ElevatorUiOpen>();
        app.init_resource::<spawning::MiningAtlas>();

        // === Systems that run during Playing state ===
        app.add_systems(
            OnEnter(GameState::Playing),
            spawning::load_mining_atlas,
        );
        app.add_systems(
            Update,
            (
                // Map transition handling (entry/exit detection)
                transitions::handle_mine_entry,
                transitions::handle_day_end_in_mine,
                transitions::cleanup_mine_on_exit,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // === Mine gameplay systems — run when Playing AND in the mine ===
        app.add_systems(
            Update,
            (
                // Floor spawning
                spawning::spawn_mine_floor,
                // Player movement and actions in the mine
                movement::mine_player_movement,
                movement::mine_player_action,
                // Rock breaking
                rock_breaking::handle_rock_breaking,
                // Combat
                combat::handle_player_attack,
                combat::enemy_ai_movement,
                combat::enemy_attack_player,
                combat::check_player_knockout,
                // Ladder and elevator
                ladder::handle_ladder_interaction,
                ladder::handle_mine_exit,
                ladder::handle_elevator_selection,
                // HUD
                hud::spawn_mine_hud,
                hud::update_floor_label,
                hud::show_elevator_prompt,
                hud::despawn_mine_hud,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
