mod hud;
mod inventory_screen;
mod dialogue_box;
mod shop_screen;
mod crafting_screen;
mod pause_menu;
mod main_menu;
mod input;
mod transitions;

use bevy::prelude::*;
use crate::shared::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // ─── FADE OVERLAY — always present ───
        app.add_systems(Startup, transitions::spawn_fade_overlay);
        app.add_systems(
            Update,
            (
                transitions::trigger_fade_on_transition,
                transitions::update_fade,
            )
                .chain(),
        );

        // ─── DIALOGUE LISTENER — runs in Playing to catch events ───
        app.add_systems(
            Update,
            dialogue_box::listen_for_dialogue_start.run_if(in_state(GameState::Playing)),
        );

        // ─── MAIN MENU ───
        app.add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu);
        app.add_systems(OnExit(GameState::MainMenu), main_menu::despawn_main_menu);
        app.add_systems(
            Update,
            (
                main_menu::update_main_menu_visuals,
                main_menu::main_menu_navigation,
            )
                .run_if(in_state(GameState::MainMenu)),
        );

        // ─── HUD — visible during Playing state ───
        app.add_systems(OnEnter(GameState::Playing), hud::spawn_hud);
        app.add_systems(OnExit(GameState::Playing), hud::despawn_hud);
        app.add_systems(
            Update,
            (
                hud::update_time_display,
                hud::update_weather_display,
                hud::update_gold_display,
                hud::update_stamina_bar,
                hud::update_tool_display,
                hud::update_hotbar,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ─── GLOBAL INPUT — runs during Playing ───
        app.add_systems(
            Update,
            (
                input::global_input_handler,
                input::hotbar_input_handler,
            )
                .run_if(in_state(GameState::Playing)),
        );
        // Also run global_input_handler in overlay states for Escape to close
        app.add_systems(
            Update,
            input::global_input_handler
                .run_if(
                    in_state(GameState::Inventory)
                        .or(in_state(GameState::Shop))
                        .or(in_state(GameState::Crafting))
                        .or(in_state(GameState::Dialogue))
                ),
        );

        // ─── INVENTORY SCREEN ───
        app.add_systems(OnEnter(GameState::Inventory), inventory_screen::spawn_inventory_screen);
        app.add_systems(OnExit(GameState::Inventory), inventory_screen::despawn_inventory_screen);
        app.add_systems(
            Update,
            (
                inventory_screen::update_inventory_slots,
                inventory_screen::update_inventory_cursor,
                inventory_screen::inventory_navigation,
            )
                .run_if(in_state(GameState::Inventory)),
        );

        // ─── DIALOGUE BOX ───
        app.add_systems(OnEnter(GameState::Dialogue), dialogue_box::spawn_dialogue_box);
        app.add_systems(OnExit(GameState::Dialogue), dialogue_box::despawn_dialogue_box);
        app.add_systems(
            Update,
            dialogue_box::advance_dialogue.run_if(in_state(GameState::Dialogue)),
        );

        // ─── SHOP SCREEN ───
        app.add_systems(OnEnter(GameState::Shop), shop_screen::spawn_shop_screen);
        app.add_systems(OnExit(GameState::Shop), shop_screen::despawn_shop_screen);
        app.add_systems(
            Update,
            (
                shop_screen::update_shop_display,
                shop_screen::shop_navigation,
            )
                .run_if(in_state(GameState::Shop)),
        );

        // ─── CRAFTING SCREEN ───
        app.add_systems(OnEnter(GameState::Crafting), crafting_screen::spawn_crafting_screen);
        app.add_systems(OnExit(GameState::Crafting), crafting_screen::despawn_crafting_screen);
        app.add_systems(
            Update,
            (
                crafting_screen::update_crafting_display,
                crafting_screen::crafting_navigation,
                crafting_screen::crafting_status_timer,
            )
                .run_if(in_state(GameState::Crafting)),
        );

        // ─── PAUSE MENU ───
        app.add_systems(OnEnter(GameState::Paused), pause_menu::spawn_pause_menu);
        app.add_systems(OnExit(GameState::Paused), pause_menu::despawn_pause_menu);
        app.add_systems(
            Update,
            (
                pause_menu::update_pause_menu_visuals,
                pause_menu::pause_menu_navigation,
            )
                .run_if(in_state(GameState::Paused)),
        );
    }
}
