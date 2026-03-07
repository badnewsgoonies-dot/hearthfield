mod shared;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

use shared::*;

fn main() {
    let default_plugins = DefaultPlugins
        .set(bevy::asset::AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            file_path: "assets".to_string(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Precinct — A Police Sim".into(),
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
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
        // Domain plugins — workers will implement these
        // .add_plugins(calendar::CalendarPlugin)
        // .add_plugins(player::PlayerPlugin)
        // .add_plugins(world::WorldPlugin)
        // .add_plugins(cases::CasePlugin)
        // .add_plugins(evidence::EvidencePlugin)
        // .add_plugins(npcs::NpcPlugin)
        // .add_plugins(economy::EconomyPlugin)
        // .add_plugins(skills::SkillPlugin)
        // .add_plugins(patrol::PatrolPlugin)
        // .add_plugins(precinct::PrecinctPlugin)
        // .add_plugins(ui::UiPlugin)
        // .add_plugins(save::SavePlugin)
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
