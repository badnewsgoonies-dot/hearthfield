use super::menu_kit::{self, set_button_visual_animated, MenuAssets, MenuButtonText};
use super::UiFontHandle;
use crate::save::{
    LoadCompleteEvent, LoadRequestEvent, NewGameEvent, SaveSlotInfoCache, NUM_SAVE_SLOTS,
};
use crate::shared::*;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MainMenuTitle;

/// Tracks main menu selection
#[derive(Resource)]
pub struct MainMenuState {
    pub mode: MainMenuMode,
    pub cursor: usize,
    pub status_message: String,
    pub pending_load_slot: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuMode {
    Root,
    LoadSlots,
}

#[cfg(not(target_arch = "wasm32"))]
const MAIN_MENU_OPTIONS: &[&str] = &[
    "New Game",
    "Load Game",
    "Fishing Encyclopedia",
    "Skywarden",
    "City Office",
    "Quit",
];

#[cfg(target_arch = "wasm32")]
const MAIN_MENU_OPTIONS: &[&str] = &["New Game", "Load Game", "Fishing Encyclopedia"];
const LOAD_MENU_BACK_INDEX: usize = NUM_SAVE_SLOTS;
const LOAD_MENU_OPTION_COUNT: usize = NUM_SAVE_SLOTS + 1;
const ROOT_MENU_OPTION_COUNT: usize = MAIN_MENU_OPTIONS.len();
const MENU_MODE_FADE_DURATION: f32 = 0.22;
const TITLE_BOB_SPEED: f32 = 1.35;
const TITLE_BOB_AMOUNT: f32 = 5.0;
const MAIN_MENU_MAX_ITEMS: usize = if ROOT_MENU_OPTION_COUNT > LOAD_MENU_OPTION_COUNT {
    ROOT_MENU_OPTION_COUNT
} else {
    LOAD_MENU_OPTION_COUNT
};

#[cfg(not(target_arch = "wasm32"))]
struct DlcLaunchTarget {
    menu_label: &'static str,
    binary_stem: &'static str,
    working_dir: &'static str,
    build_command: &'static str,
}

#[cfg(not(target_arch = "wasm32"))]
const SKYWARDEN_TARGET: DlcLaunchTarget = DlcLaunchTarget {
    menu_label: "Skywarden",
    binary_stem: "skywarden",
    working_dir: "dlc/pilot",
    build_command: "cargo build -p skywarden",
};

#[cfg(not(target_arch = "wasm32"))]
const CITY_OFFICE_TARGET: DlcLaunchTarget = DlcLaunchTarget {
    menu_label: "City Office",
    binary_stem: "city_office_worker_dlc",
    working_dir: "dlc/city",
    build_command: "cargo build -p city_office_worker_dlc",
};

#[cfg(not(target_arch = "wasm32"))]
fn sibling_dlc_binary_path(binary_stem: &str) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join(format!("{binary_stem}{}", std::env::consts::EXE_SUFFIX))
}

#[cfg(not(target_arch = "wasm32"))]
fn dlc_working_dir(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

#[cfg(not(target_arch = "wasm32"))]
fn launch_dlc(target: &DlcLaunchTarget) -> Result<(), String> {
    let binary_path = sibling_dlc_binary_path(target.binary_stem);
    let working_dir = dlc_working_dir(target.working_dir);
    if !binary_path.is_file() {
        return Err(format!(
            "{} not installed. Build with: {}",
            target.menu_label, target.build_command
        ));
    }
    if !working_dir.is_dir() {
        return Err(format!(
            "{} launch directory missing. Build with: {}",
            target.menu_label, target.build_command
        ));
    }

    std::process::Command::new(&binary_path)
        .current_dir(&working_dir)
        .spawn()
        .map(|_| ())
        .map_err(|err| {
            format!(
                "Failed to launch {}. Build with: {} ({err})",
                target.menu_label, target.build_command
            )
        })
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_main_menu(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    assets: Res<MenuAssets>,
    theme: Res<MenuTheme>,
) {
    commands.insert_resource(MainMenuState {
        mode: MainMenuMode::Root,
        cursor: 0,
        status_message: String::new(),
        pending_load_slot: None,
    });
    commands.insert_resource(MainMenuVisualState {
        previous_mode: MainMenuMode::Root,
        transition_t: 1.0,
    });

    let font = font_handle.0.clone();

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(22.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.18, 0.08)),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|title_block| {
                    title_block.spawn((
                        MainMenuTitle,
                        Node {
                            margin: UiRect::top(Val::Px(0.0)),
                            ..default()
                        },
                        Text::new("HEARTHFIELD"),
                        TextFont {
                            font: font.clone(),
                            font_size: theme.title_font_size + 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.94, 0.72)),
                        PickingBehavior::IGNORE,
                    ));

                    title_block.spawn((
                        Text::new("A Farming & Life Simulator"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.76, 0.84, 0.67)),
                        PickingBehavior::IGNORE,
                    ));
                });

            // Menu options container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: Val::Px(theme.button_width + 36.0),
                    row_gap: Val::Px(12.0),
                    margin: UiRect::vertical(Val::Px(6.0)),
                    ..default()
                })
                .with_children(|menu| {
                    for i in 0..MAIN_MENU_MAX_ITEMS {
                        menu_kit::spawn_menu_button(menu, i, "", &assets, &theme, &font);
                    }
                });

            // Status line (used for load failures, etc.)
            parent.spawn((
                MainMenuStatusText,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.75, 0.45)),
            ));

            // Version text
            menu_kit::spawn_menu_footer(parent, "v0.1.0 - Early Development", &theme, &font);
        });
}

#[derive(Component)]
pub struct MainMenuStatusText;

#[derive(Resource)]
pub struct MainMenuVisualState {
    pub previous_mode: MainMenuMode,
    pub transition_t: f32,
}

fn current_option_count(mode: MainMenuMode) -> usize {
    match mode {
        MainMenuMode::Root => MAIN_MENU_OPTIONS.len(),
        MainMenuMode::LoadSlots => MAIN_MENU_MAX_ITEMS,
    }
}

fn format_play_time(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

fn format_last_saved(timestamp: u64) -> String {
    if timestamp == 0 {
        return "Never".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now <= timestamp {
        return "Just now".to_string();
    }
    let diff = now - timestamp;
    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn load_slot_label(slot_info: Option<&crate::save::SaveSlotInfo>) -> String {
    if let Some(info) = slot_info {
        if info.exists {
            format!(
                "Slot {}  {}\nDay {} {:?} Y{}  {}g\nPlayed {}  Saved {}",
                info.slot + 1,
                info.farm_name,
                info.day,
                info.season,
                info.year,
                info.gold,
                format_play_time(info.play_time_seconds),
                format_last_saved(info.save_timestamp)
            )
        } else {
            format!("Slot {}  (Empty)\nStart a new farm here", info.slot + 1)
        }
    } else {
        "Slot ?  (Unavailable)".to_string()
    }
}

fn menu_option_label(
    mode: MainMenuMode,
    index: usize,
    cache: Option<&SaveSlotInfoCache>,
) -> Option<(String, bool)> {
    match mode {
        MainMenuMode::Root => MAIN_MENU_OPTIONS
            .get(index)
            .map(|label| ((*label).to_string(), true)),
        MainMenuMode::LoadSlots => {
            if index == LOAD_MENU_BACK_INDEX {
                Some(("Back".to_string(), true))
            } else {
                let slot_info = cache.and_then(|c| c.slots.get(index));
                let slot_exists = slot_info.map(|s| s.exists).unwrap_or(false);
                Some((load_slot_label(slot_info), slot_exists))
            }
        }
    }
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<MainMenuState>();
    commands.remove_resource::<MainMenuVisualState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE / INTERACTION
// ═══════════════════════════════════════════════════════════════════════

pub fn update_main_menu_visuals(
    time: Res<Time>,
    state: Option<Res<MainMenuState>>,
    mut visual_state: Option<ResMut<MainMenuVisualState>>,
    cache: Option<Res<SaveSlotInfoCache>>,
    mut item_query: Query<(&MenuItem, &mut ImageNode, &mut Node, &mut Visibility), Without<MainMenuTitle>>,
    mut title_query: Query<(&mut Node, &mut TextColor, &mut TextFont), (With<MainMenuTitle>, Without<MenuItem>)>,
    mut text_query: Query<
        (&MenuButtonText, &mut Text, &mut TextColor, &mut TextFont),
        (Without<MainMenuStatusText>, Without<MainMenuTitle>),
    >,
    mut status_query: Query<&mut Text, (With<MainMenuStatusText>, Without<MenuButtonText>)>,
) {
    let Some(state) = state else { return };
    let Some(ref mut visual_state) = visual_state else {
        return;
    };
    if visual_state.previous_mode != state.mode {
        visual_state.previous_mode = state.mode;
        visual_state.transition_t = 0.0;
    } else {
        visual_state.transition_t = (visual_state.transition_t
            + time.delta_secs() / MENU_MODE_FADE_DURATION)
            .clamp(0.0, 1.0);
    }
    let fade_in = visual_state.transition_t
        * visual_state.transition_t
        * (3.0 - 2.0 * visual_state.transition_t);
    let pulse = ((time.elapsed_secs() * 4.5).sin() * 0.5 + 0.5) * 0.75;
    let title_bob = (time.elapsed_secs() * TITLE_BOB_SPEED).sin();
    let option_count = current_option_count(state.mode);

    if let Ok((mut title_node, mut title_color, mut title_font)) = title_query.get_single_mut() {
        title_node.margin.top = Val::Px(title_bob * TITLE_BOB_AMOUNT);
        title_font.font_size = 50.0 + (title_bob * 0.5 + 0.5) * 2.0;
        title_color.0 = Color::srgb(
            1.0,
            0.92 + (title_bob * 0.5 + 0.5) * 0.04,
            0.70 + (title_bob * 0.5 + 0.5) * 0.05,
        );
    }

    for (item, mut image_node, mut node, mut visibility) in &mut item_query {
        if item.index >= option_count {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        let selected = item.index == state.cursor;
        let selected_pulse = if selected { pulse } else { 0.0 };
        node.margin.left = if selected {
            Val::Px(8.0 + pulse * 4.0)
        } else {
            Val::Px(0.0)
        };
        set_button_visual_animated(
            &mut image_node,
            selected,
            selected_pulse,
            0.85 + fade_in * 0.15,
        );
    }

    for (btn_text, mut text, mut color, mut font) in &mut text_query {
        if btn_text.index >= option_count {
            text.0.clear();
            continue;
        }

        let Some((label, enabled)) =
            menu_option_label(state.mode, btn_text.index, cache.as_deref())
        else {
            text.0.clear();
            continue;
        };
        text.0 = label;
        let selected = btn_text.index == state.cursor;
        let alpha = 0.75 + fade_in * 0.25;
        font.font_size = if selected { 16.5 + pulse * 1.0 } else { 14.5 };
        color.0 = if selected {
            let glow = 0.85 + pulse * 0.15;
            Color::srgba(1.0, glow, 0.72, alpha)
        } else if enabled {
            Color::srgba(0.92, 0.92, 0.92, alpha * 0.92)
        } else {
            Color::srgba(0.58, 0.58, 0.58, alpha * 0.82)
        }
    }

    let Ok(mut status) = status_query.get_single_mut() else {
        return;
    };
    status.0 = state.status_message.clone();
}

#[allow(clippy::too_many_arguments)]
pub fn main_menu_navigation(
    action: Res<MenuAction>,
    mut state: Option<ResMut<MainMenuState>>,
    cache: Option<Res<SaveSlotInfoCache>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut load_events: EventWriter<LoadRequestEvent>,
    mut app_exit: EventWriter<AppExit>,
    mut cutscene_queue: ResMut<CutsceneQueue>,
    mut fade: ResMut<super::transitions::ScreenFade>,
) {
    let Some(ref mut state) = state else { return };
    let option_count = current_option_count(state.mode);

    // Pointer hover → set cursor
    if let Some(idx) = action.set_cursor {
        if idx < option_count {
            state.cursor = idx;
        }
    }

    if action.move_down && state.cursor + 1 < option_count {
        state.cursor += 1;
    }
    if action.move_up && state.cursor > 0 {
        state.cursor -= 1;
    }

    if action.activate {
        // Guard: don't spam requests while load is in-flight
        if state.pending_load_slot.is_some() {
            return;
        }
        match state.mode {
            MainMenuMode::Root => match state.cursor {
                0 => {
                    new_game_events.send(NewGameEvent {
                        farm_name: "Hearthfield Farm".to_string(),
                        active_slot: 0,
                    });
                    // Set screen to black before entering Playing so the
                    // farm spawns invisibly behind the fade overlay.
                    fade.alpha = 1.0;
                    fade.target_alpha = 1.0;
                    fade.active = false;
                    // Pre-populate the cutscene queue with the intro sequence.
                    // start_pending_cutscene (OnEnter Playing) will detect this
                    // and redirect to Cutscene state.
                    cutscene_queue.steps = super::intro_sequence::build_intro_sequence();
                    cutscene_queue.active = true;
                    cutscene_queue.step_timer = 0.0;
                    next_state.set(GameState::Playing);
                }
                1 => {
                    state.mode = MainMenuMode::LoadSlots;
                    state.cursor = 0;
                    state.status_message.clear();
                }
                2 => {
                    next_state.set(GameState::FishEncyclopedia);
                }
                #[cfg(not(target_arch = "wasm32"))]
                3 => match launch_dlc(&SKYWARDEN_TARGET) {
                    Ok(()) => {
                        state.status_message = "Launching Skywarden...".to_string();
                        app_exit.send(AppExit::Success);
                    }
                    Err(err) => {
                        state.status_message = err;
                    }
                },
                #[cfg(not(target_arch = "wasm32"))]
                4 => match launch_dlc(&CITY_OFFICE_TARGET) {
                    Ok(()) => {
                        state.status_message = "Launching City Office...".to_string();
                        app_exit.send(AppExit::Success);
                    }
                    Err(err) => {
                        state.status_message = err;
                    }
                },
                #[cfg(not(target_arch = "wasm32"))]
                5 => {
                    app_exit.send(AppExit::Success);
                }
                _ => {}
            },
            MainMenuMode::LoadSlots => {
                if state.cursor == LOAD_MENU_BACK_INDEX {
                    state.mode = MainMenuMode::Root;
                    state.cursor = 0;
                    state.status_message.clear();
                    state.pending_load_slot = None;
                } else {
                    let slot = state.cursor as u8;
                    let slot_exists = cache
                        .as_ref()
                        .and_then(|c| c.slots.get(state.cursor))
                        .map(|s| s.exists)
                        .unwrap_or(false);

                    if slot_exists {
                        state.pending_load_slot = Some(slot);
                        state.status_message = format!("Loading Slot {}...", slot + 1);
                        load_events.send(LoadRequestEvent { slot });
                    } else {
                        state.status_message = format!("Slot {} is empty.", slot + 1);
                    }
                }
            }
        }
    }

    if action.cancel && state.mode == MainMenuMode::LoadSlots {
        state.mode = MainMenuMode::Root;
        state.cursor = 0;
        state.status_message.clear();
        state.pending_load_slot = None;
    }
}

pub fn handle_load_complete_in_main_menu(
    mut load_complete_events: EventReader<LoadCompleteEvent>,
    mut state: Option<ResMut<MainMenuState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut state) = state else { return };

    for ev in load_complete_events.read() {
        if state.pending_load_slot != Some(ev.slot) {
            continue;
        }

        state.pending_load_slot = None;
        if ev.success {
            state.status_message.clear();
            next_state.set(GameState::Playing);
        } else {
            state.status_message = ev
                .error_message
                .clone()
                .unwrap_or_else(|| "Failed to load save slot.".to_string());
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn sibling_dlc_binary_path_uses_platform_suffix() {
        let path = sibling_dlc_binary_path("skywarden");
        let expected = format!("skywarden{}", std::env::consts::EXE_SUFFIX);
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some(expected.as_str())
        );
    }

    #[test]
    fn menu_button_pool_covers_root_and_load_menus() {
        assert!(MAIN_MENU_MAX_ITEMS >= MAIN_MENU_OPTIONS.len());
        assert!(MAIN_MENU_MAX_ITEMS >= LOAD_MENU_OPTION_COUNT);
    }

    #[test]
    fn dlc_working_dir_is_repo_relative() {
        let dir = dlc_working_dir("dlc/pilot");
        assert!(dir.ends_with(Path::new("dlc").join("pilot")));
    }
}
