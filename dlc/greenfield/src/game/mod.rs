use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod systems;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GreenfieldState {
    #[default]
    Boot,
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Loading,
    Settings,
    Credits,
    Combat,
    Inventory,
    Dialog,
    Trading,
    Crafting,
    Upgrading,
    Resting,
    Exploring,
    Benchmarking,
    McpScaleVar01,
    McpScaleVar02,
    McpScaleVar03,
    McpScaleVar04,
    BigBatchVar01,
    BigBatchVar02,
    BigBatchVar03,
    BigBatchVar04,
    BigBatchVar05,
    BigBatchVar06,
    BigBatchVar07,
    BigBatchVar08,
    BigBatchVar09,
    BigBatchVar10,
    Crafting__Idle,
    Crafting__Active,
    Crafting__Cooldown,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GreenfieldSet {
    Input,
    Simulation,
    Render,
}

pub struct GreenfieldPlugin;

impl Plugin for GreenfieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GreenfieldState>()
            .add_systems(Update, systems::music_tick_sys::music_tick_system)
            .init_resource::<resources::MusicState>()
            .add_event::<events::MusicCrossfadeRequestedEvent>()
            .add_event::<events::MusicTrackEndedEvent>()
            .add_event::<events::MusicTrackStartedEvent>()
            .add_systems(Update, systems::checkpoint_activate_sys::checkpoint_activate_system)
            .init_resource::<resources::CheckpointState>()
            .add_event::<events::CheckpointRegistryUpdatedEvent>()
            .add_event::<events::CheckpointRestoredEvent>()
            .add_event::<events::CheckpointActivatedEvent>()
            .add_systems(Update, systems::quest_complete_sys::quest_complete_system)
            .add_systems(Update, systems::quest_accept_sys::quest_accept_system)
            .init_resource::<resources::QuestLog>()
            .add_event::<events::QuestProgressEvent>()
            .add_event::<events::QuestAbandonedEvent>()
            .add_event::<events::QuestCompletedEvent>()
            .add_event::<events::QuestAcceptedEvent>()
            .add_systems(Update, systems::cooldown_tracker::cooldown_tick_system)
            .init_resource::<resources::CooldownClock>()
            .add_systems(Update, systems::regen_health::regen_health)
            .add_systems(Update, systems::apply_damage::apply_damage)
            .add_systems(Startup, systems::init_player_health::init_player_health)
            .add_event::<events::PlayerDamage>()
            .init_resource::<resources::PlayerHealth>()
            .add_systems(Update, systems::tick_observer_sys::tick_observer_system)
            .add_systems(Update, systems::tick_advance_sys::tick_advance_system)
            .add_event::<events::TickAdvancedEvent>()
            .add_systems(Update, systems::frame_telemetry_sys::frame_telemetry_system)
            .add_event::<events::FrameSampledEvent>()
            .init_resource::<resources::FrameTelemetry>()
            .add_systems(Update, systems::crafting_cancel_sys::crafting_cancel_system)
            .add_systems(Update, systems::crafting_cleanup_sys::crafting_cleanup_system)
            .add_systems(Update, systems::crafting_output_sys::crafting_output_system)
            .add_systems(Update, systems::crafting_complete_sys::crafting_complete_system)
            .add_systems(Update, systems::crafting_progress_sys::crafting_progress_system)
            .add_systems(Update, systems::crafting_consume_sys::crafting_consume_system)
            .add_systems(Update, systems::crafting_validate_sys::crafting_validate_system)
            .add_systems(Update, systems::crafting_intake_sys::crafting_intake_system)
            .init_resource::<resources::CraftingConfig>()
            .init_resource::<resources::RecipeBook>()
            .init_resource::<resources::ActiveCrafting>()
            .add_event::<events::OutputProducedEvent>()
            .add_event::<events::MaterialConsumedEvent>()
            .add_event::<events::RecipeUnlockedEvent>()
            .add_event::<events::CraftingFailedEvent>()
            .add_event::<events::CraftingCompletedEvent>()
            .add_event::<events::CraftingStartedEvent>()
            .add_systems(Update, systems::inventory_sync_sys::inventory_sync_system)
            .add_systems(Update, systems::inventory_overflow_sys::inventory_overflow_system)
            .add_systems(Update, systems::inventory_stack_sys::inventory_stack_system)
            .add_systems(Update, systems::inventory_consume_sys::inventory_consume_system)
            .add_systems(Update, systems::inventory_equip_sys::inventory_equip_system)
            .add_systems(Update, systems::inventory_drop_sys::inventory_drop_system)
            .add_systems(Update, systems::inventory_pickup_sys::inventory_pickup_system)
            .init_resource::<resources::ItemCatalog>()
            .init_resource::<resources::InventoryConfig>()
            .init_resource::<resources::ActiveInventory>()
            .add_event::<events::ItemStackedEvent>()
            .add_event::<events::ItemConsumedEvent>()
            .add_event::<events::InventoryOverflowEvent>()
            .add_event::<events::ItemUnequippedInvEvent>()
            .add_event::<events::ItemEquippedInvEvent>()
            .add_event::<events::ItemDroppedInvEvent>()
            .add_event::<events::ItemPickedUpInvEvent>()
            .add_systems(Update, systems::combat_resolve_sys::combat_resolve_system)
            .add_systems(Update, systems::combat_damage_sys::combat_damage_system)
            .add_systems(Update, systems::combat_attack_sys::combat_attack_system)
            .add_systems(Update, systems::combat_initiate_sys::combat_initiate_system)
            .init_resource::<resources::CurrentCombatant>()
            .init_resource::<resources::CombatClock>()
            .add_event::<events::CombatResolvedEvent>()
            .add_event::<events::DamageAppliedEvent>()
            .add_event::<events::AttackStartedEvent>()
            .add_event::<events::CombatInitiatedEvent>()
            .add_systems(Update, systems::heartbeat_pulse::heartbeat_pulse_system)
            .add_event::<events::HeartbeatPulseEvent>()
            .add_systems(Update, systems::score_broadcast_tick::score_broadcast_tick_system)
            .add_event::<events::ScoreBroadcastEvent>()
            .init_resource::<resources::BroadcastScore>()
            .add_event::<events::TrialAchainEvent>()
            .init_resource::<resources::TrialA3Res>()
            .add_event::<events::TrialA3Event>()
            .add_systems(Update, systems::mcp_sys_10::mcp_sys_10_system)
            .add_systems(Update, systems::mcp_sys_09::mcp_sys_09_system)
            .add_systems(Update, systems::mcp_sys_08::mcp_sys_08_system)
            .add_systems(Update, systems::mcp_sys_07::mcp_sys_07_system)
            .add_systems(Update, systems::mcp_sys_06::mcp_sys_06_system)
            .add_systems(Update, systems::mcp_sys_05::mcp_sys_05_system)
            .add_systems(Update, systems::mcp_sys_04::mcp_sys_04_system)
            .add_systems(Update, systems::mcp_sys_03::mcp_sys_03_system)
            .add_systems(Update, systems::mcp_sys_02::mcp_sys_02_system)
            .add_systems(Update, systems::mcp_sys_01::mcp_sys_01_system)
            .add_plugins(plugins::McpPlugin10)
            .add_plugins(plugins::McpPlugin09)
            .add_plugins(plugins::McpPlugin08)
            .add_plugins(plugins::McpPlugin07)
            .add_plugins(plugins::McpPlugin06)
            .add_plugins(plugins::McpPlugin05)
            .add_plugins(plugins::McpPlugin04)
            .add_plugins(plugins::McpPlugin03)
            .add_plugins(plugins::McpPlugin02)
            .add_plugins(plugins::McpPlugin01)
            .init_resource::<resources::BareRes10>()
            .init_resource::<resources::BareRes09>()
            .init_resource::<resources::BareRes08>()
            .init_resource::<resources::BareRes07>()
            .init_resource::<resources::BareRes06>()
            .init_resource::<resources::BareRes05>()
            .init_resource::<resources::BareRes04>()
            .init_resource::<resources::BareRes03>()
            .init_resource::<resources::BareRes02>()
            .init_resource::<resources::BareRes01>()
            .add_event::<events::BareEvent10>()
            .add_event::<events::BareEvent09>()
            .add_event::<events::BareEvent08>()
            .add_event::<events::BareEvent07>()
            .add_event::<events::BareEvent06>()
            .add_event::<events::BareEvent05>()
            .add_event::<events::BareEvent04>()
            .add_event::<events::BareEvent03>()
            .add_event::<events::BareEvent02>()
            .add_event::<events::BareEvent01>()
            .init_resource::<resources::FinisherRes03>()
            .init_resource::<resources::FinisherRes02>()
            .init_resource::<resources::FinisherRes01>()
            .init_resource::<resources::BigBatchRes10>()
            .init_resource::<resources::BigBatchRes09>()
            .init_resource::<resources::BigBatchRes08>()
            .init_resource::<resources::BigBatchRes07>()
            .init_resource::<resources::BigBatchRes06>()
            .init_resource::<resources::BigBatchRes05>()
            .init_resource::<resources::BigBatchRes04>()
            .init_resource::<resources::BigBatchRes03>()
            .init_resource::<resources::BigBatchRes02>()
            .init_resource::<resources::BigBatchRes01>()
            .add_event::<events::BigBatchEvent10>()
            .add_event::<events::BigBatchEvent09>()
            .add_event::<events::BigBatchEvent08>()
            .add_event::<events::BigBatchEvent07>()
            .add_event::<events::BigBatchEvent06>()
            .add_event::<events::BigBatchEvent05>()
            .add_event::<events::BigBatchEvent04>()
            .add_event::<events::BigBatchEvent03>()
            .add_event::<events::BigBatchEvent02>()
            .add_event::<events::BigBatchEvent01>()
            .init_resource::<resources::McpScaleRes04>()
            .init_resource::<resources::McpScaleRes03>()
            .init_resource::<resources::McpScaleRes02>()
            .init_resource::<resources::McpScaleRes01>()
            .add_event::<events::McpScaleEvent04>()
            .add_event::<events::McpScaleEvent03>()
            .add_event::<events::McpScaleEvent02>()
            .add_event::<events::McpScaleEvent01>()
            .init_resource::<resources::BenchmarkBudget>()
            .add_event::<events::BenchmarkStartedEvent>()
            .add_systems(Update, systems::drain_score_changes::drain_score_changes_system)
            .add_systems(Update, systems::drain_damage::drain_damage_system)
            .add_systems(Update, systems::emit_game_loaded_tick::emit_game_loaded_tick_system)
            .add_systems(Update, systems::award_xp::award_xp_system)
            .add_systems(Update, systems::increment_score::increment_score_system)
            .add_systems(Update, systems::advance_turn::advance_turn_system)
            .add_systems(Update, systems::log_score_changes::log_score_changes_system)
            .add_systems(Update, systems::loot_drop_sys::drop_loot_system)
            .add_systems(Update, systems::combat_resolver_sys::resolve_combat_system)
            .add_systems(Update, systems::enemy_spawner_sys::spawn_enemies_system)
            .add_systems(Update, systems::damage_tick_sys::damage_tick_system)
            .init_resource::<resources::LevelProgress>()
            .init_resource::<resources::GameScore>()
            .init_resource::<resources::SpawnerRegistry>()
            .init_resource::<resources::LootTable>()
            .init_resource::<resources::EnemyCatalog>()
            .add_event::<events::EnemySpawnedEvent>()
            .add_event::<events::ExperienceGainedEvent>()
            .add_event::<events::ItemPickedUpEvent>()
            .add_event::<events::DamageDealtEvent>()
            .add_event::<events::KeyCollectedEvent>()
            .add_event::<events::DoorOpenedEvent>()
            .add_event::<events::GoalReachedEvent>()
            .add_event::<events::CheckpointReachedEvent>()
            .add_event::<events::PortalActivatedEvent>()
            .add_event::<events::ChestOpenedEvent>()
            .add_event::<events::AllyDownedEvent>()
            .add_event::<events::EnemyDefeatedEvent>()
            .add_systems(Update, systems::update_hud_sys::update_hud_system)
            .add_systems(Update, systems::play_audio_sys::play_audio_system)
            .add_systems(Update, systems::handle_config_sys::handle_config_system)
            .add_systems(Update, systems::record_tick_sys::record_tick_system)
            .add_systems(Update, systems::tick_clock_sys::tick_clock_system)
            .add_event::<events::PlayerMovedEvent>()
            .add_event::<events::ScoreChangedEvent>()
            .add_event::<events::TimerElapsedEvent>()
            .add_event::<events::ButtonPressedEvent>()
            .add_event::<events::ConfigurationChangedEvent>()
            .add_event::<events::GameUnloadedEvent>()
            .add_event::<events::GameLoadedEvent>()
            .init_resource::<resources::GameConfig>()
            .init_resource::<resources::RecordingBuffer>()
            .init_resource::<resources::TurnClock>()
            .init_resource::<resources::SettingsCache>()
            .init_resource::<resources::AudioManager>()
            .add_systems(Update, systems::render_tick::render_tick)
            .add_systems(Update, systems::sim_tick::sim_tick)
            .add_systems(Update, systems::input_tick::input_tick)
            .add_systems(Update, systems::boot_tick::boot_tick)
            .add_event::<events::RenderTickedEvent>()
            .add_event::<events::SimulationTickedEvent>()
            .add_event::<events::InputTickedEvent>()
            .add_event::<events::TurnRecordedEvent>()
            .add_event::<events::TurnEndedEvent>()
            .add_event::<events::TurnBeganEvent>()
            .add_event::<events::GameEndedEvent>()
            .add_event::<events::GameResumedEvent>()
            .add_event::<events::GamePausedEvent>()
            .add_event::<events::GameStartedEvent>()
            .add_event::<events::MainMenuExitedEvent>()
            .add_event::<events::MainMenuEnteredEvent>()
            .add_event::<events::BootCompletedEvent>()
            .add_event::<events::BootStartedEvent>()
            ;
    }
}
pub mod scene;
pub mod audio;
pub mod input;
pub mod combat;
pub mod inventory;
pub mod ai_behavior;
pub mod dialogue;
pub mod quests;
pub mod scaffold_01;
pub mod scaffold_02;
pub mod scaffold_03;
pub mod scaffold_04;
pub mod scaffold_05;
pub mod scaffold_06;
pub mod scaffold_11;
pub mod scaffold_12;
pub mod scaffold_13;
pub mod scaffold_14;
pub mod scaffold_15;
pub mod crafting_data;
pub mod crafting_lookup;
pub mod crafting_validator;
