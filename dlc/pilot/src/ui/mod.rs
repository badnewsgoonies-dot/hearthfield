//! UI domain — HUD, menus, dialogue box, flight instruments, toast notifications.

use bevy::prelude::*;
use crate::shared::*;

pub mod hud;
pub mod main_menu;
pub mod pause_menu;
pub mod dialogue_box;
pub mod mission_screen;
pub mod inventory_screen;
pub mod shop_screen;
pub mod flight_hud;
pub mod toast;
pub mod transitions;
pub mod audio;
pub mod debug_overlay;
pub mod settings;
pub mod logbook_screen;
pub mod map_screen;
pub mod crew_screen;
pub mod achievement_screen;
pub mod cutscene_runner;
pub mod intro_sequence;
pub mod notification_center;
pub mod profile_screen;
pub mod radio_screen;
pub mod tutorial;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_font)
            .add_systems(
                Update,
                (
                    hud::update_hud.run_if(in_state(GameState::Playing)),
                    toast::update_toasts,
                    transitions::update_screen_fade,
                    debug_overlay::toggle_debug_overlay,
                    audio::handle_play_sfx,
                    audio::handle_play_music,
                    flight_hud::update_flight_hud.run_if(in_state(GameState::Flying)),
                ),
            )
            .add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), main_menu::despawn_main_menu)
            .add_systems(OnEnter(GameState::Paused), pause_menu::spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), pause_menu::despawn_pause_menu)
            .add_systems(
                OnEnter(GameState::Dialogue),
                dialogue_box::spawn_dialogue_box,
            )
            .add_systems(
                OnExit(GameState::Dialogue),
                dialogue_box::despawn_dialogue_box,
            )
            .add_systems(
                OnEnter(GameState::MissionBoard),
                mission_screen::spawn_mission_screen,
            )
            .add_systems(
                OnExit(GameState::MissionBoard),
                mission_screen::despawn_mission_screen,
            )
            .add_systems(
                OnEnter(GameState::Inventory),
                inventory_screen::spawn_inventory_screen,
            )
            .add_systems(
                OnExit(GameState::Inventory),
                inventory_screen::despawn_inventory_screen,
            )
            .add_systems(OnEnter(GameState::Shop), shop_screen::spawn_shop_screen)
            .add_systems(OnExit(GameState::Shop), shop_screen::despawn_shop_screen)
            .add_systems(OnEnter(GameState::RadioComm), radio_screen::spawn_radio_screen)
            .add_systems(OnExit(GameState::RadioComm), radio_screen::despawn_radio_screen)
            .add_systems(OnEnter(GameState::CrewLounge), crew_screen::spawn_crew_screen)
            .add_systems(OnExit(GameState::CrewLounge), crew_screen::despawn_crew_screen)
            .add_systems(OnEnter(GameState::Cutscene), cutscene_runner::spawn_cutscene_overlay)
            .add_systems(OnExit(GameState::Cutscene), cutscene_runner::despawn_cutscene_overlay)
            .add_systems(OnEnter(GameState::Playing), hud::spawn_hud)
            .add_systems(OnExit(GameState::Playing), hud::despawn_hud)
            .add_systems(OnEnter(GameState::Flying), flight_hud::spawn_flight_hud)
            .add_systems(OnExit(GameState::Flying), flight_hud::despawn_flight_hud)
            .add_systems(
                Update,
                main_menu::handle_main_menu_input.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                Update,
                pause_menu::handle_pause_input.run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                Update,
                mission_screen::handle_mission_screen_input
                    .run_if(in_state(GameState::MissionBoard)),
            )
            .add_systems(
                Update,
                radio_screen::handle_radio_input.run_if(in_state(GameState::RadioComm)),
            )
            .add_systems(
                Update,
                crew_screen::handle_crew_screen_input.run_if(in_state(GameState::CrewLounge)),
            )
            .add_systems(
                Update,
                (
                    cutscene_runner::run_cutscene,
                    cutscene_runner::skip_cutscene,
                )
                    .run_if(in_state(GameState::Cutscene)),
            )
            .add_systems(
                Update,
                (
                    shop_screen::handle_shop_buy,
                    shop_screen::handle_shop_keyboard,
                )
                    .run_if(in_state(GameState::Shop)),
            );
    }
}

fn load_font(mut ui_font: ResMut<UiFontHandle>, asset_server: Res<AssetServer>) {
    ui_font.0 = asset_server.load("fonts/pixel_font.ttf");
}
