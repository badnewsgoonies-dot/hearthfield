//! Skywarden — Pilot Life Simulator
//!
//! Main entry point. Registers all resources, events, states, and plugins.

#![allow(dead_code, unused_imports, clippy::upper_case_acronyms)]

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use std::path::PathBuf;

mod aircraft;
mod airports;
mod crew;
mod data;
mod economy;
mod flight;
mod input;
mod missions;
mod player;
mod save;
mod shared;
mod ui;
mod weather;
mod world;

use shared::*;

fn main() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("pilot crate should live under dlc/pilot/")
        .to_path_buf()
        .to_string_lossy()
        .into_owned();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Skywarden — Pilot Life Sim".into(),
                        resolution: bevy::window::WindowResolution::new(1280.0, 720.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_root,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // ── State ────────────────────────────────────────────────────────
        .init_state::<GameState>()
        // ── Core Resources ───────────────────────────────────────────────
        .init_resource::<Calendar>()
        .init_resource::<WeatherState>()
        .init_resource::<PlayerLocation>()
        .init_resource::<PilotState>()
        .init_resource::<PlayerMovement>()
        .init_resource::<GridPosition>()
        .init_resource::<PlayerInput>()
        .init_resource::<KeyBindings>()
        .init_resource::<InputState>()
        .init_resource::<Inventory>()
        .init_resource::<ItemRegistry>()
        .init_resource::<Gold>()
        .init_resource::<EconomyStats>()
        .init_resource::<ActiveShop>()
        .init_resource::<Fleet>()
        .init_resource::<AircraftRegistry>()
        .init_resource::<FlightState>()
        .init_resource::<MissionBoard>()
        .init_resource::<MissionLog>()
        .init_resource::<CrewRegistry>()
        .init_resource::<Relationships>()
        .init_resource::<DialogueState>()
        .init_resource::<Achievements>()
        .init_resource::<PlayStats>()
        .init_resource::<WorldMap>()
        .init_resource::<CollisionMap>()
        .init_resource::<InteractionClaimed>()
        .init_resource::<UiFontHandle>()
        .init_resource::<MenuTheme>()
        .init_resource::<DebugOverlay>()
        .init_resource::<MusicState>()
        .init_resource::<SaveSlots>()
        .init_resource::<SessionTimer>()
        .init_resource::<TutorialState>()
        .init_resource::<CutsceneQueue>()
        .init_resource::<ActiveCutscene>()
        .init_resource::<CityRegistry>()
        .init_resource::<flight::navigation::NavigationState>()
        .init_resource::<player::camera::CameraState>()
        .init_resource::<economy::gold::TransactionLog>()
        .init_resource::<economy::gold::GoldMilestones>()
        .init_resource::<aircraft::fuel::FuelWarnings>()
        .init_resource::<aircraft::maintenance::MaintenanceTracker>()
        .init_resource::<aircraft::fleet::HangarAssignments>()
        .init_resource::<economy::shop::ShopRestockTimer>()
        .init_resource::<economy::progression::ActivityTracker>()
        .init_resource::<player::movement::PlayerAnimState>()
        // ── Events ───────────────────────────────────────────────────────
        .add_event::<DayEndEvent>()
        .add_event::<SeasonChangeEvent>()
        .add_event::<ZoneTransitionEvent>()
        .add_event::<AirportArrivalEvent>()
        .add_event::<FlightStartEvent>()
        .add_event::<FlightCompleteEvent>()
        .add_event::<FlightPhaseChangeEvent>()
        .add_event::<EmergencyEvent>()
        .add_event::<GoldChangeEvent>()
        .add_event::<PurchaseEvent>()
        .add_event::<DialogueStartEvent>()
        .add_event::<GiftGivenEvent>()
        .add_event::<FriendshipChangeEvent>()
        .add_event::<MissionAcceptedEvent>()
        .add_event::<MissionCompletedEvent>()
        .add_event::<MissionFailedEvent>()
        .add_event::<ItemPickupEvent>()
        .add_event::<RankUpEvent>()
        .add_event::<LicenseEarnedEvent>()
        .add_event::<AchievementUnlockedEvent>()
        .add_event::<XpGainEvent>()
        .add_event::<ToastEvent>()
        .add_event::<PlaySfxEvent>()
        .add_event::<PlayMusicEvent>()
        .add_event::<ScreenFadeEvent>()
        .add_event::<WeatherChangeEvent>()
        .add_event::<SaveRequestEvent>()
        .add_event::<LoadRequestEvent>()
        .add_event::<SaveCompleteEvent>()
        .add_event::<LoadCompleteEvent>()
        .add_event::<CutsceneStartEvent>()
        .add_event::<HintEvent>()
        .add_event::<economy::shop::SellItemEvent>()
        // ── Plugins (order matters: input first) ─────────────────────────
        .add_plugins(input::InputPlugin)
        .add_plugins(data::DataPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(airports::AirportPlugin)
        .add_plugins(flight::FlightPlugin)
        .add_plugins(aircraft::AircraftPlugin)
        .add_plugins(missions::MissionPlugin)
        .add_plugins(crew::CrewPlugin)
        .add_plugins(weather::WeatherPlugin)
        .add_plugins(economy::EconomyPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(save::SavePlugin)
        .add_plugins(world::WorldPlugin)
        // ── Startup ──────────────────────────────────────────────────────
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    ));
}
