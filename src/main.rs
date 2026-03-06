mod animals;
mod calendar;
mod crafting;
mod data;
mod economy;
mod farming;
mod fishing;
mod input;
mod mining;
mod npcs;
mod player;
mod save;
mod shared;
mod ui;
mod world;

use bevy::asset::AssetMetaCheck;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use shared::*;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .to_string_lossy()
        .into_owned();

    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            #[cfg(not(target_arch = "wasm32"))]
            file_path: asset_path,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hearthfield".into(),
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
        // Clear color (dark navy, close to HTML background)
        .insert_resource(ClearColor(Color::srgb(0.15, 0.22, 0.12)))
        // Game state
        .init_state::<GameState>()
        // Shared resources
        .init_resource::<Calendar>()
        .init_resource::<PlayerState>()
        .init_resource::<Inventory>()
        .init_resource::<FarmState>()
        .init_resource::<AnimalState>()
        .init_resource::<Relationships>()
        .init_resource::<MineState>()
        .init_resource::<UnlockedRecipes>()
        .init_resource::<ShippingBin>()
        .init_resource::<ItemRegistry>()
        .init_resource::<CropRegistry>()
        .init_resource::<FishRegistry>()
        .init_resource::<RecipeRegistry>()
        .init_resource::<NpcRegistry>()
        .init_resource::<ShopData>()
        // Phase 4 resources
        .init_resource::<HouseState>()
        .init_resource::<MarriageState>()
        .init_resource::<QuestLog>()
        .init_resource::<SprinklerState>()
        .init_resource::<ActiveBuffs>()
        .init_resource::<EvaluationScore>()
        .init_resource::<RelationshipStages>()
        // Phase 3 resources
        .init_resource::<Achievements>()
        .init_resource::<ShippingLog>()
        .init_resource::<TutorialState>()
        .init_resource::<PlayStats>()
        .init_resource::<InputBlocks>()
        .init_resource::<InteractionClaimed>()
        .init_resource::<CutsceneQueue>()
        // Input & menu abstraction
        .init_resource::<PlayerInput>()
        .init_resource::<InputContext>()
        .init_resource::<KeyBindings>()
        .init_resource::<MenuTheme>()
        .init_resource::<MenuAction>()
        // Events
        .add_event::<DayEndEvent>()
        .add_event::<SeasonChangeEvent>()
        .add_event::<ItemPickupEvent>()
        .add_event::<ItemRemovedEvent>()
        .add_event::<DialogueStartEvent>()
        .add_event::<DialogueEndEvent>()
        .add_event::<ShopTransactionEvent>()
        .add_event::<ToolUseEvent>()
        .add_event::<MapTransitionEvent>()
        .add_event::<StaminaDrainEvent>()
        .add_event::<GoldChangeEvent>()
        .add_event::<GiftGivenEvent>()
        .add_event::<CropHarvestedEvent>()
        .add_event::<AnimalProductEvent>()
        .add_event::<PlaySfxEvent>()
        .add_event::<PlayMusicEvent>()
        // Phase 4 events
        .add_event::<BouquetGivenEvent>()
        .add_event::<ProposalEvent>()
        .add_event::<WeddingEvent>()
        .add_event::<SpouseActionEvent>()
        .add_event::<QuestPostedEvent>()
        .add_event::<QuestAcceptedEvent>()
        .add_event::<QuestCompletedEvent>()
        .add_event::<PlaceSprinklerEvent>()
        .add_event::<EatFoodEvent>()
        .add_event::<EvaluationTriggerEvent>()
        // Phase 3 events
        .add_event::<HintEvent>()
        .add_event::<AchievementUnlockedEvent>()
        .add_event::<BuildingUpgradeEvent>()
        .add_event::<ToolImpactEvent>()
        .add_event::<ToastEvent>()
        // Input plugin (before all domain plugins)
        .add_plugins(input::InputPlugin)
        // Domain plugins
        .add_plugins(calendar::CalendarPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(farming::FarmingPlugin)
        .add_plugins(animals::AnimalPlugin)
        .add_plugins(world::WorldPlugin)
        .add_plugins(npcs::NpcPlugin)
        .add_plugins(economy::EconomyPlugin)
        .add_plugins(crafting::CraftingPlugin)
        .add_plugins(fishing::FishingPlugin)
        .add_plugins(mining::MiningPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(save::SavePlugin)
        // Data loading
        .add_plugins(data::DataPlugin)
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
