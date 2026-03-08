#![allow(dead_code)]
#![allow(clippy::derivable_impls)]

mod domains;
mod shared;

use bevy::asset::AssetMetaCheck;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use domains::calendar::CalendarPlugin;
use domains::cases::CasesPlugin;
use domains::economy::EconomyPlugin;
use domains::evidence::EvidencePlugin;
use domains::npcs::NpcsPlugin;
use domains::patrol::PatrolPlugin;
use domains::player::PlayerPlugin;
use domains::precinct::PrecinctPlugin;
use domains::save::SavePlugin;
use domains::skills::SkillsPlugin;
use domains::ui::UiPlugin;
use domains::world::WorldPlugin;
use shared::*;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .to_string_lossy()
        .into_owned();

    let default_plugins = DefaultPlugins
        .set(bevy::asset::AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            #[cfg(not(target_arch = "wasm32"))]
            file_path: asset_path,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Precinct — A Police Sim".into(),
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#game-canvas".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest());

    App::new()
        .add_plugins(default_plugins)
        .insert_resource(ClearColor(Color::srgb(0.10, 0.12, 0.18)))
        .insert_resource(Time::<Fixed>::from_hz(5.0))
        // Game state
        .init_state::<GameState>()
        // Shared update-phase ordering
        .configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        )
        // Shared resources
        .init_resource::<ShiftClock>()
        .init_resource::<PlayerState>()
        .init_resource::<Inventory>()
        .init_resource::<CaseBoard>()
        .init_resource::<EvidenceLocker>()
        .init_resource::<NpcRegistry>()
        .init_resource::<Economy>()
        .init_resource::<Skills>()
        .init_resource::<PatrolState>()
        .init_resource::<PartnerArc>()
        .init_resource::<PlayerInput>()
        .init_resource::<InputContext>()
        // Events
        .add_event::<ShiftEndEvent>()
        .add_event::<CaseAssignedEvent>()
        .add_event::<CaseSolvedEvent>()
        .add_event::<CaseFailedEvent>()
        .add_event::<EvidenceCollectedEvent>()
        .add_event::<EvidenceProcessedEvent>()
        .add_event::<InterrogationStartEvent>()
        .add_event::<InterrogationEndEvent>()
        .add_event::<DispatchCallEvent>()
        .add_event::<DispatchResolvedEvent>()
        .add_event::<PromotionEvent>()
        .add_event::<NpcTrustChangeEvent>()
        .add_event::<DialogueStartEvent>()
        .add_event::<DialogueEndEvent>()
        .add_event::<MapTransitionEvent>()
        .add_event::<FatigueChangeEvent>()
        .add_event::<StressChangeEvent>()
        .add_event::<GoldChangeEvent>()
        .add_event::<XpGainedEvent>()
        .add_event::<SkillPointSpentEvent>()
        .add_event::<PlaySfxEvent>()
        .add_event::<PlayMusicEvent>()
        .add_event::<ToastEvent>()
        .add_event::<SaveRequestEvent>()
        .add_event::<LoadRequestEvent>()
        // Wave 1 domain plugins
        .add_plugins(CalendarPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(CasesPlugin)
        .add_plugins(EvidencePlugin)
        .add_plugins(PatrolPlugin)
        .add_plugins(PrecinctPlugin)
        .add_plugins(SkillsPlugin)
        .add_plugins(EconomyPlugin)
        .add_plugins(NpcsPlugin)
        .add_plugins(SavePlugin)
        // Future domain plugins (Wave 2+)
        // Camera
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Msaa::Off,
        Tonemapping::None,
        Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    ));
}
