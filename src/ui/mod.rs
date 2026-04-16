mod audio;
pub mod building_upgrade_menu;
pub mod calendar_screen;
mod chest_screen;
mod crafting_screen;
pub mod cursor;
pub mod cutscene_runner;
mod debug_overlay;
pub mod dialogue_box;
mod fish_encyclopedia;
mod hud;
// (input.rs removed — all input routing via src/input/mod.rs + menu_input.rs)
pub mod intro_sequence;
mod inventory_screen;
pub mod journal_screen;
mod main_menu;
pub mod map_screen;
pub mod menu_input;
pub mod menu_kit;
mod minimap;
mod pause_menu;
pub mod relationships_screen;
pub mod settings_screen;
mod shop_screen;
pub mod stats_screen;
mod toast;
pub mod tool_tutorial;
pub mod transitions;
pub mod tutorial;

use crate::shared::*;
use bevy::prelude::*;

pub const ITEM_ATLAS_COLUMNS: usize = 13;
pub const ITEM_ATLAS_ROWS: usize = 20;

pub fn item_icon_index(sprite_index: u32) -> usize {
    let idx = sprite_index as usize;
    let max = ITEM_ATLAS_COLUMNS * ITEM_ATLAS_ROWS; // 13 × 19 = 247
    if idx < max {
        idx
    } else {
        0
    }
}

/// Build an ImageNode for an item, using per-crop Pickup icons when available.
pub fn item_image_node(
    atlas_data: &hud::ItemAtlasData,
    item_id: Option<&str>,
    sprite_index: u32,
) -> ImageNode {
    if let Some(id) = item_id {
        if let Some(icon) = atlas_data.crop_overrides.get(id) {
            return ImageNode {
                image: icon.clone(),
                ..default()
            };
        }
    }
    ImageNode {
        image: atlas_data.image.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: atlas_data.layout.clone(),
            index: item_icon_index(sprite_index),
        }),
        ..default()
    }
}

/// Update an existing ImageNode for an item, swapping to per-crop icon if available.
pub fn apply_item_icon(
    img: &mut ImageNode,
    atlas_data: &hud::ItemAtlasData,
    item_id: &str,
    sprite_index: u32,
) {
    if let Some(icon) = atlas_data.crop_overrides.get(item_id) {
        img.image = icon.clone();
        img.texture_atlas = None;
    } else {
        img.image = atlas_data.image.clone();
        img.texture_atlas = Some(TextureAtlas {
            layout: atlas_data.layout.clone(),
            index: item_icon_index(sprite_index),
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ICON HELPERS — build ImageNodes from shared icon atlases
// ═══════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════
// UI ICON ATLASES — preloaded at startup, available to all screens
// ═══════════════════════════════════════════════════════════════════════

/// Preloaded icon atlas handles — avoids needing layouts parameter in screens.
/// [Observed] layouts verified via image inspection.
#[derive(Resource, Default)]
pub struct UiIconAtlases {
    /// icons.png: 18 cols × 3 rows general UI icons
    pub icons_image: Handle<Image>,
    pub icons_layout: Handle<TextureAtlasLayout>,
    /// icons_special.png: 7 cols × 4 rows (stars, hearts, coins, gems)
    pub special_image: Handle<Image>,
    pub special_layout: Handle<TextureAtlasLayout>,
    /// icons_white.png: 6 cols × 3 rows monochrome UI icons (disabled/hover states)
    pub icons_white_image: Handle<Image>,
    pub icons_white_layout: Handle<TextureAtlasLayout>,
    /// ui_spritesheet.png: 56 cols × 15 rows comprehensive UI element atlas
    /// Contains buttons, sliders, checkboxes, panels, symbols, and decorative elements.
    pub ui_sheet_image: Handle<Image>,
    pub ui_sheet_layout: Handle<TextureAtlasLayout>,
    /// inventory_spritesheet.png: 23 cols × 21 rows inventory UI elements
    /// Contains hearts, health bars, inventory slot backgrounds, button panels.
    pub inventory_sheet_image: Handle<Image>,
    pub inventory_sheet_layout: Handle<TextureAtlasLayout>,
    /// buttons_26x26.png: 2 cols × 8 rows of 26×26 button backgrounds (light/dark pairs).
    /// Rows graduate from light (top) to dark (bottom) for state styling.
    /// Note: non-16×16 tile size — use buttons_image/layout specifically.
    pub buttons_image: Handle<Image>,
    pub buttons_layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

fn preload_ui_icons(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlases: ResMut<UiIconAtlases>,
) {
    if atlases.loaded {
        return;
    }
    atlases.icons_image = asset_server.load("ui/icons.png");
    atlases.icons_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 18, 3, None, None,
    ));
    atlases.special_image = asset_server.load("ui/icons_special.png");
    atlases.special_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 7, 4, None, None,
    ));
    // icons_white.png: 96×48 → 6 cols × 3 rows of 16×16 monochrome icons
    atlases.icons_white_image = asset_server.load("ui/icons_white.png");
    atlases.icons_white_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 6, 3, None, None,
    ));
    // ui_spritesheet.png: 896×240 → 56 cols × 15 rows of 16×16 UI elements
    atlases.ui_sheet_image = asset_server.load("ui/ui_spritesheet.png");
    atlases.ui_sheet_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 56, 15, None, None,
    ));
    // inventory_spritesheet.png: 368×336 → 23 cols × 21 rows of 16×16 elements
    atlases.inventory_sheet_image = asset_server.load("ui/inventory_spritesheet.png");
    atlases.inventory_sheet_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 23, 21, None, None,
    ));
    // buttons_26x26.png: 96×192, tile size 48×24 → 2 cols × 8 rows of button bg
    atlases.buttons_image = asset_server.load("ui/buttons_26x26.png");
    atlases.buttons_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 24), 2, 8, None, None,
    ));
    atlases.loaded = true;
}

/// Premium non-atlas UI images — dialog backgrounds, panel art, etc.
/// Loaded once at startup, available to all screens.
#[derive(Resource, Default)]
pub struct PremiumUiImages {
    /// dialog_box.png: 48×48 9-slice tile for custom-sized panels
    pub dialog_9slice: Handle<Image>,
    /// dialog_box_big.png: 176×48 pre-rendered clean dialog panel
    pub dialog_big: Handle<Image>,
    /// dialog_box_medium.png: 128×48 pre-rendered clean dialog panel
    pub dialog_medium: Handle<Image>,
    /// dialog_box_small.png: 112×48 pre-rendered clean dialog panel
    pub dialog_small: Handle<Image>,
    /// inventory_hearts_light.png: 7 cols × 21 rows — light theme heart icons
    pub hearts_light_image_x: Handle<Image>,
    pub hearts_light_layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

fn preload_premium_ui(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut images: ResMut<PremiumUiImages>,
) {
    if images.loaded {
        return;
    }
    images.dialog_9slice = asset_server.load("ui/dialog_box.png");
    images.dialog_big = asset_server.load("ui/dialog_box_big.png");
    images.dialog_medium = asset_server.load("ui/dialog_box_medium.png");
    images.dialog_small = asset_server.load("ui/dialog_box_small.png");
    images.hearts_light_image_x = asset_server.load("ui/inventory_hearts_light.png");
    images.hearts_light_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 7, 21, None, None,
    ));
    images.loaded = true;
}

/// Build an ImageNode from icons.png by atlas index.
pub fn icon_node(atlases: &UiIconAtlases, index: usize) -> ImageNode {
    ImageNode {
        image: atlases.icons_image.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: atlases.icons_layout.clone(),
            index,
        }),
        ..default()
    }
}

/// Build an ImageNode from icons_special.png by atlas index.
pub fn special_icon_node(atlases: &UiIconAtlases, index: usize) -> ImageNode {
    ImageNode {
        image: atlases.special_image.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: atlases.special_layout.clone(),
            index,
        }),
        ..default()
    }
}

/// Standard 20×20 Node for screen title icons.
pub fn icon_size_node() -> Node {
    Node {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        ..Default::default()
    }
}

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
        // ─── CUSTOM CURSOR — load images + hide OS cursor at Startup ───
        app.add_systems(Startup, cursor::setup_cursor);
        app.add_systems(
            Update,
            cursor::update_cursor_sprite.in_set(UpdatePhase::Presentation),
        );

        // ─── FONT LOADING + MENU ASSETS + ICON ATLASES — runs at Startup ───
        app.init_resource::<UiIconAtlases>();
        app.init_resource::<PremiumUiImages>();
        app.add_systems(Startup, (load_ui_font, menu_kit::load_menu_assets, preload_ui_icons, preload_premium_ui));

        // ─── AUDIO — music state resource + event handlers ───
        app.init_resource::<audio::MusicState>();
        app.init_resource::<audio::MusicFade>();
        app.init_resource::<hud::ItemAtlasData>();
        app.init_resource::<hud::WeatherIconAtlas>();
        app.add_systems(
            Update,
            (
                audio::handle_play_sfx,
                audio::handle_play_music,
                audio::toast_sfx,
                audio::tick_music_fade,
            )
                .in_set(UpdatePhase::Reactions),
        );
        app.add_systems(OnEnter(GameState::Playing), audio::start_game_music);
        app.add_systems(OnEnter(GameState::MainMenu), audio::start_menu_music);
        app.add_systems(
            Update,
            (
                audio::switch_music_on_season_change,
                audio::switch_music_on_map_change,
                audio::door_sfx_on_map_change,
            )
                .in_set(UpdatePhase::Reactions)
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
                .chain()
                .in_set(UpdatePhase::Presentation),
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
                .in_set(UpdatePhase::Simulation)
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
            cutscene_runner::activate_pending_cutscene.run_if(in_state(GameState::Playing)),
        );

        // ─── DIALOGUE LISTENER — runs in Playing AND Cutscene to catch events ───
        app.add_systems(
            Update,
            (
                dialogue_box::listen_for_dialogue_start,
                dialogue_box::handle_dialogue_end,
            )
                .in_set(UpdatePhase::Reactions)
                .run_if(in_state(GameState::Playing).or(in_state(GameState::Cutscene))),
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
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::MainMenu)),
        );
        app.add_systems(
            OnEnter(GameState::FishEncyclopedia),
            fish_encyclopedia::spawn_fish_encyclopedia_screen,
        );
        app.add_systems(
            OnExit(GameState::FishEncyclopedia),
            fish_encyclopedia::despawn_fish_encyclopedia_screen,
        );
        app.add_systems(
            Update,
            (
                fish_encyclopedia::update_fish_encyclopedia_visuals,
                fish_encyclopedia::fish_encyclopedia_navigation,
            )
                .run_if(in_state(GameState::FishEncyclopedia)),
        );

        // ─── HUD — visible during Playing state ───
        app.insert_resource(hud::FloatingGoldCooldown {
            timer: {
                let mut t = Timer::from_seconds(0.5, TimerMode::Once);
                // Start finished so the first gold event can fire immediately.
                t.tick(std::time::Duration::from_millis(501));
                t
            },
        });
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                hud::preload_item_atlas,
                hud::preload_weather_icon_atlas,
                hud::spawn_hud,
                hud::spawn_touch_overlay,
            ),
        );
        app.add_systems(
            OnExit(GameState::Playing),
            (
                hud::despawn_hud,
                hud::despawn_floating_gold_text,
                hud::despawn_touch_overlay,
            ),
        );
        app.add_systems(
            Update,
            (
                hud::update_time_display,
                hud::update_weather_display,
                hud::update_weather_icon,
                hud::update_gold_display,
                hud::update_stamina_bar,
                hud::update_health_bar,
                hud::update_tool_display,
                hud::update_hotbar,
                hud::hydrate_hotbar_icons,
                hud::update_hotbar_icons,
                hud::bob_selected_hotbar_icon,
                minimap::update_minimap,
                hud::update_map_name,
                hud::update_objective_display,
                hud::update_interaction_prompt,
                hud::update_controls_hint,
                hud::update_touch_overlay,
                hud::spawn_floating_gold_text,
                hud::update_floating_gold_text,
            )
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::Playing)),
        );

        // ─── TOAST NOTIFICATIONS ───
        app.add_systems(OnEnter(GameState::Playing), toast::spawn_toast_container);
        app.add_systems(OnExit(GameState::Playing), toast::despawn_toast_container);
        app.add_systems(OnEnter(GameState::Playing), minimap::spawn_minimap);
        app.add_systems(OnExit(GameState::Playing), minimap::despawn_minimap);
        app.add_systems(
            Update,
            (
                toast::handle_toast_events,
                toast::update_toasts,
                toast::wire_gold_toasts,
                toast::wire_season_toasts,
                toast::wire_pickup_toasts,
            )
                .in_set(UpdatePhase::Presentation)
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
                .in_set(UpdatePhase::Reactions)
                .run_if(in_state(GameState::Playing)),
        );

        // ─── MENU ACTION RESET (PreUpdate, after input reader) ───
        app.add_systems(PreUpdate, menu_input::reset_menu_action);

        // ─── GLOBAL INPUT — unified via PlayerInput / MenuAction ───
        app.add_systems(
            Update,
            (
                menu_input::merge_keyboard_to_menu_action,
                menu_input::gameplay_state_transitions.run_if(in_state(GameState::Playing)),
                menu_input::hotbar_input_handler.run_if(in_state(GameState::Playing)),
                menu_input::menu_cancel_transitions.run_if(
                    in_state(GameState::Inventory)
                        .or(in_state(GameState::Shop))
                        .or(in_state(GameState::Crafting))
                        .or(in_state(GameState::Dialogue))
                        .or(in_state(GameState::FishEncyclopedia))
                        .or(in_state(GameState::Journal))
                        .or(in_state(GameState::RelationshipsView))
                        .or(in_state(GameState::MapView)),
                ),
            )
                .in_set(UpdatePhase::Input),
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

        // ─── JOURNAL SCREEN ───
        app.add_systems(
            OnEnter(GameState::Journal),
            journal_screen::spawn_journal_screen,
        );
        app.add_systems(
            OnExit(GameState::Journal),
            journal_screen::despawn_journal_screen,
        );
        app.add_systems(
            Update,
            (
                journal_screen::update_quest_display,
                journal_screen::update_cursor_highlight,
                journal_screen::journal_navigation,
            )
                .run_if(in_state(GameState::Journal)),
        );

        // ─── RELATIONSHIPS SCREEN ───
        app.init_resource::<relationships_screen::HeartIconAtlas>();
        app.add_systems(
            OnEnter(GameState::RelationshipsView),
            relationships_screen::spawn_relationships_screen,
        );
        app.add_systems(
            OnExit(GameState::RelationshipsView),
            relationships_screen::despawn_relationships_screen,
        );
        app.add_systems(
            Update,
            (
                relationships_screen::update_relationships_cursor,
                relationships_screen::relationships_navigation,
            )
                .run_if(in_state(GameState::RelationshipsView)),
        );

        // ─── MAP SCREEN ───
        app.add_systems(OnEnter(GameState::MapView), map_screen::spawn_map_screen);
        app.add_systems(OnExit(GameState::MapView), map_screen::despawn_map_screen);

        // ─── DIALOGUE BOX ───
        app.add_systems(
            OnEnter(GameState::Dialogue),
            dialogue_box::spawn_dialogue_box,
        );
        app.add_systems(
            OnExit(GameState::Dialogue),
            (
                dialogue_box::despawn_dialogue_box,
                tool_tutorial::despawn_tool_tutorial_overlay,
            ),
        );
        app.add_systems(
            Update,
            (
                dialogue_box::typewriter_update,
                dialogue_box::advance_dialogue,
            )
                .chain()
                .run_if(in_state(GameState::Dialogue)),
        );

        // ─── TOOL TUTORIAL OVERLAY — runs during Dialogue state ───
        app.add_systems(
            Update,
            tool_tutorial::update_tool_tutorial_overlay
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::Dialogue)),
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
        app.add_systems(
            Startup,
            debug_overlay::spawn_debug_overlay.after(load_ui_font),
        );
        app.add_systems(
            Update,
            (
                debug_overlay::toggle_debug_overlay,
                debug_overlay::update_debug_overlay,
            ),
        );

        // ─── CHEST SCREEN (reactive overlay during Playing state) ───
        app.add_systems(
            Update,
            (
                chest_screen::update_chest_ui_lifecycle,
                chest_screen::update_chest_inv_display,
                chest_screen::update_chest_storage_display,
                chest_screen::update_chest_cursor,
                chest_screen::update_chest_quality_borders,
                chest_screen::handle_chest_input,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ─── CALENDAR OVERLAY (F1 toggle during Playing) ───
        app.init_resource::<calendar_screen::CalendarOverlayState>();
        app.add_systems(
            Update,
            (
                calendar_screen::toggle_calendar_overlay,
                calendar_screen::calendar_close_on_escape,
                calendar_screen::update_calendar_lifecycle,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        // ─── STATISTICS OVERLAY (F2 toggle during Playing) ───
        app.init_resource::<stats_screen::StatsOverlayState>();
        app.add_systems(
            Update,
            (
                stats_screen::toggle_stats_overlay,
                stats_screen::stats_close_on_escape,
                stats_screen::update_stats_lifecycle,
                stats_screen::refresh_stats_display,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        // ─── SETTINGS OVERLAY (F4 toggle during Playing) ───
        app.init_resource::<settings_screen::SettingsOverlayState>();
        app.init_resource::<settings_screen::AudioVolume>();
        app.add_systems(
            Update,
            (
                settings_screen::toggle_settings_overlay,
                settings_screen::settings_close_on_escape,
                settings_screen::update_settings_lifecycle,
                settings_screen::settings_volume_input,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}
