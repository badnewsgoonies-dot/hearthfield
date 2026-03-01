mod audio;
pub mod building_upgrade_menu;
mod chest_screen;
mod crafting_screen;
pub mod cutscene_runner;
mod debug_overlay;
pub mod dialogue_box;
mod hud;
mod input;
pub mod intro_sequence;
mod inventory_screen;
mod main_menu;
pub mod menu_input;
pub mod menu_kit;
mod pause_menu;
mod shop_screen;
mod toast;
pub mod transitions;
pub mod tutorial;

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// SHARED FONT HANDLE — used by all UI text across every screen
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource)]
pub struct UiFontHandle(pub Handle<Font>);

fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/sprout_lands.ttf");
    commands.insert_resource(UiFontHandle(font));
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // ─── FONT LOADING + MENU ASSETS — runs at Startup ───
        app.add_systems(Startup, (load_ui_font, menu_kit::load_menu_assets));

        // ─── AUDIO — music state resource + event handlers ───
        app.init_resource::<audio::MusicState>();
        app.add_systems(Update, (audio::handle_play_sfx, audio::handle_play_music));
        app.add_systems(OnEnter(GameState::Playing), audio::start_game_music);
        app.add_systems(OnEnter(GameState::MainMenu), audio::start_menu_music);
        app.add_systems(
            Update,
            (
                audio::switch_music_on_season_change,
                audio::switch_music_on_map_change,
            )
                .run_if(in_state(GameState::Playing)),
        );

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

        // ─── CUTSCENE RUNNER ───
        app.init_resource::<cutscene_runner::CutsceneFlags>();
        app.add_systems(
            OnEnter(GameState::Cutscene),
            cutscene_runner::on_enter_cutscene,
        );
        app.add_systems(
            OnExit(GameState::Cutscene),
            cutscene_runner::on_exit_cutscene,
        );
        app.add_systems(
            Update,
            cutscene_runner::run_cutscene_queue
                .run_if(in_state(GameState::Cutscene)),
        );
        // When entering Playing, check if a cutscene queue was pre-populated
        // (e.g. intro sequence from main menu) and redirect to Cutscene state.
        app.add_systems(
            OnEnter(GameState::Playing),
            cutscene_runner::start_pending_cutscene,
        );

        // After all Update systems have processed DayEndEvents, check if
        // trigger_sleep or tick_time queued a cutscene and activate it.
        app.add_systems(
            PostUpdate,
            cutscene_runner::activate_pending_cutscene
                .run_if(in_state(GameState::Playing)),
        );

        // ─── DIALOGUE LISTENER — runs in Playing AND Cutscene to catch events ───
        app.add_systems(
            Update,
            (
                dialogue_box::listen_for_dialogue_start,
                dialogue_box::handle_dialogue_end,
            )
                .run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::Cutscene)),
                ),
        );

        // ─── MAIN MENU ───
        app.add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu);
        app.add_systems(OnExit(GameState::MainMenu), main_menu::despawn_main_menu);
        app.add_systems(
            Update,
            (
                main_menu::update_main_menu_visuals,
                main_menu::main_menu_navigation,
                main_menu::handle_load_complete_in_main_menu,
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
                hud::update_map_name,
                hud::update_objective_display,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ─── TOAST NOTIFICATIONS ───
        app.add_systems(OnEnter(GameState::Playing), toast::spawn_toast_container);
        app.add_systems(OnExit(GameState::Playing), toast::despawn_toast_container);
        app.add_systems(
            Update,
            (
                toast::handle_toast_events,
                toast::update_toasts,
                toast::wire_gold_toasts,
                toast::wire_season_toasts,
                toast::wire_pickup_toasts,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ─── TUTORIAL & CONTEXTUAL HINTS ───
        app.add_systems(
            Update,
            (
                tutorial::check_tutorial_hints,
                tutorial::forward_hint_to_toast,
                tutorial::check_objectives,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        // ─── MENU ACTION RESET (PreUpdate, after input reader) ───
        app.add_systems(PreUpdate, menu_input::reset_menu_action);

        // ─── GLOBAL INPUT — unified via PlayerInput / MenuAction ───
        app.add_systems(
            Update,
            (
                menu_input::merge_keyboard_to_menu_action,
                menu_input::gameplay_state_transitions
                    .run_if(in_state(GameState::Playing)),
                menu_input::hotbar_input_handler
                    .run_if(in_state(GameState::Playing)),
                menu_input::menu_cancel_transitions.run_if(
                    in_state(GameState::Inventory)
                        .or(in_state(GameState::Shop))
                        .or(in_state(GameState::Crafting))
                        .or(in_state(GameState::Dialogue)),
                ),
            ),
        );

        // ─── INVENTORY SCREEN ───
        app.add_systems(
            OnEnter(GameState::Inventory),
            inventory_screen::spawn_inventory_screen,
        );
        app.add_systems(
            OnExit(GameState::Inventory),
            inventory_screen::despawn_inventory_screen,
        );
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
        app.add_systems(
            OnEnter(GameState::Dialogue),
            dialogue_box::spawn_dialogue_box,
        );
        app.add_systems(
            OnExit(GameState::Dialogue),
            dialogue_box::despawn_dialogue_box,
        );
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
        app.add_systems(
            OnEnter(GameState::Crafting),
            crafting_screen::spawn_crafting_screen,
        );
        app.add_systems(
            OnExit(GameState::Crafting),
            crafting_screen::despawn_crafting_screen,
        );
        app.add_systems(
            Update,
            (
                crafting_screen::update_crafting_display,
                crafting_screen::crafting_navigation,
                crafting_screen::crafting_status_timer,
            )
                .run_if(in_state(GameState::Crafting)),
        );

        // ─── BUILDING UPGRADE MENU ───
        app.add_systems(
            OnEnter(GameState::BuildingUpgrade),
            building_upgrade_menu::spawn_building_upgrade_menu,
        );
        app.add_systems(
            OnExit(GameState::BuildingUpgrade),
            building_upgrade_menu::despawn_building_upgrade_menu,
        );
        app.add_systems(
            Update,
            (
                building_upgrade_menu::update_building_upgrade_display,
                building_upgrade_menu::building_upgrade_navigation,
                building_upgrade_menu::building_upgrade_status_timer,
            )
                .run_if(in_state(GameState::BuildingUpgrade)),
        );

        // ─── PAUSE MENU ───
        app.add_systems(OnEnter(GameState::Paused), pause_menu::spawn_pause_menu);
        app.add_systems(OnExit(GameState::Paused), pause_menu::despawn_pause_menu);
        app.add_systems(
            Update,
            (
                pause_menu::update_pause_menu_visuals,
                pause_menu::pause_menu_navigation,
                pause_menu::handle_save_complete_in_pause_menu,
            )
                .run_if(in_state(GameState::Paused)),
        );

        // ─── DEBUG OVERLAY (always available, toggled by F3) ───
        app.init_resource::<DebugOverlayState>();
        app.add_systems(Startup, debug_overlay::spawn_debug_overlay.after(load_ui_font));
        app.add_systems(Update, (
            debug_overlay::toggle_debug_overlay,
            debug_overlay::update_debug_overlay,
        ));

        // ─── CHEST SCREEN (reactive overlay during Playing state) ───
        app.add_systems(
            Update,
            (
                chest_screen::update_chest_ui_lifecycle,
                chest_screen::update_chest_inv_display,
                chest_screen::update_chest_storage_display,
                chest_screen::update_chest_cursor,
                chest_screen::handle_chest_input,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
