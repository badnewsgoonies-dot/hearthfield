use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod systems;

/// Top-level state machine for the Greenfield DLC.
///
/// Pruned from the original scaffold to seven coherent states that
/// reflect the farming-defense gameplay loop. Matches the shape of
/// `OfficeGameState` in the city DLC — boot, menu, in-game phases,
/// pause, end.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GreenfieldState {
    /// Initial state on app boot. Loads sprites and resources, then
    /// transitions to MainMenu.
    #[default]
    Boot,
    /// Title screen. Press any key to start.
    MainMenu,
    /// Active play phase — farmer is tending crops; no critters
    /// currently on the field.
    Tending,
    /// Active play phase — critters present; the farmer must defend.
    Defending,
    /// Active play phase — harvest cycle; crops have matured and
    /// the farmer is collecting them.
    Harvesting,
    /// Game paused (Esc).
    Paused,
    /// Run ended (farmer hp hit zero, or all crops eaten).
    GameOver,
}

// Update-phase set ordering. Greenfield is a Hearthfield DLC
// sibling, so it consumes the host's shared UpdatePhase from
// hearthfield::shared::schedule rather than defining its own.
// (I3 of RUBRIC.md.)
pub use hearthfield::shared::UpdatePhase;

pub struct GreenfieldPlugin;

impl Plugin for GreenfieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GreenfieldState>()
            // ─── Hearthfield host integration (v12) ──────────────────
            .init_resource::<hearthfield::shared::Calendar>()
            .add_event::<hearthfield::shared::CropHarvestedEvent>()
            // ─────────────────────────────────────────────────────────
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
            .init_resource::<resources::TickCounter>()
            .init_resource::<systems::wave_address_sys::WaveSeed>()
            .add_systems(Update, systems::wave_address_sys::spawn_addressed_wave_system)
            // Lifecycle events emitted by the state machine (v16) but never registered.
            .add_event::<events::GameStartedEvent>()
            .add_event::<events::GameEndedEvent>()
            .add_event::<events::GamePausedEvent>()
            .add_event::<events::GameResumedEvent>()
            .add_event::<events::MainMenuEnteredEvent>()
            .add_event::<events::MainMenuExitedEvent>()
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
            .add_event::<events::EnemyDefeatedEvent>()
            .add_systems(Update, systems::update_hud_sys::update_hud_system)
            .add_systems(Update, systems::handle_config_sys::handle_config_system)
            .add_systems(Update, systems::record_tick_sys::record_tick_system)
            .add_systems(Update, systems::tick_clock_sys::tick_clock_system)
            .add_event::<events::PlayerMovedEvent>()
            .add_event::<events::ScoreChangedEvent>()
            .add_event::<events::GameLoadedEvent>()
            .init_resource::<resources::GameConfig>()
            .init_resource::<resources::RecordingBuffer>()
            .init_resource::<resources::TurnClock>()
            .init_resource::<resources::AudioManager>()
            .add_systems(Update, systems::sim_tick::sim_tick)
            .add_systems(Update, systems::input_tick::input_tick)
            .add_systems(Update, systems::boot_tick::boot_tick)
            // ─── v15: previously-orphaned systems wired in ───────────
            .add_systems(Update, systems::play_audio_sys::play_audio_system)
            .add_systems(Update, systems::render_tick::render_tick)
            // ─── v15: McpPlugin01..10 scaffolds registered ──────────
            .add_plugins((
                plugins::McpPlugin01, plugins::McpPlugin02,
                plugins::McpPlugin03, plugins::McpPlugin04,
                plugins::McpPlugin05, plugins::McpPlugin06,
                plugins::McpPlugin07, plugins::McpPlugin08,
                plugins::McpPlugin09, plugins::McpPlugin10,
            ))
            // ─── v16: state machine wiring ──────────────────────────
            .init_resource::<systems::state_machine::PrePauseState>()
            .add_systems(Update, (
                systems::state_machine::main_menu_to_tending
                    .run_if(in_state(GreenfieldState::MainMenu)),
                systems::state_machine::tending_to_defending
                    .run_if(in_state(GreenfieldState::Tending)),
                systems::state_machine::defending_to_tending
                    .run_if(in_state(GreenfieldState::Defending)),
                systems::state_machine::pause_toggle,
                systems::state_machine::check_game_over,
                systems::state_machine::game_over_to_main_menu
                    .run_if(in_state(GreenfieldState::GameOver)),
            ))
            .add_systems(OnEnter(GreenfieldState::MainMenu),
                         systems::state_machine::spawn_main_menu_hud)
            .add_systems(OnExit(GreenfieldState::MainMenu),
                         systems::state_machine::despawn_main_menu_hud)
            .add_systems(OnEnter(GreenfieldState::Paused),
                         systems::state_machine::spawn_pause_hud)
            .add_systems(OnExit(GreenfieldState::Paused),
                         systems::state_machine::despawn_pause_hud)
            .add_systems(OnEnter(GreenfieldState::GameOver),
                         systems::state_machine::spawn_game_over_hud)
            .add_systems(OnExit(GreenfieldState::GameOver),
                         systems::state_machine::despawn_game_over_hud)
            // ─── v17: Harvest rhythm ────────────────────────────────
            .init_resource::<systems::state_machine::HarvestCycle>()
            .add_systems(Update, (
                systems::state_machine::tending_to_harvesting
                    .run_if(in_state(GreenfieldState::Tending)),
                systems::state_machine::harvesting_to_tending
                    .run_if(in_state(GreenfieldState::Harvesting)),
                systems::state_machine::harvest_payout
                    .run_if(in_state(GreenfieldState::Harvesting)),
            ))
            .add_systems(OnEnter(GreenfieldState::Harvesting),
                         systems::state_machine::spawn_harvesting_hud)
            .add_systems(OnExit(GreenfieldState::Harvesting),
                         systems::state_machine::despawn_harvesting_hud)
            // ─── v17b: emit lifecycle events on state transitions ───
            .add_systems(OnEnter(GreenfieldState::MainMenu),
                         systems::state_machine::emit_main_menu_entered)
            .add_systems(OnExit(GreenfieldState::MainMenu), (
                systems::state_machine::emit_main_menu_exited,
                systems::state_machine::emit_game_started,
            ))
            .add_systems(OnEnter(GreenfieldState::Paused),
                         systems::state_machine::emit_game_paused)
            .add_systems(OnExit(GreenfieldState::Paused),
                         systems::state_machine::emit_game_resumed)
            .add_systems(OnEnter(GreenfieldState::GameOver),
                         systems::state_machine::emit_game_ended)
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
pub mod crafting_data;
pub mod crafting_lookup;
pub mod crafting_validator;
