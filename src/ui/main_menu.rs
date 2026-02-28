use super::UiFontHandle;
use super::menu_kit::{self, MenuAssets, MenuButtonText, set_button_visual};
use crate::save::{
    LoadCompleteEvent, LoadRequestEvent, NewGameEvent, SaveSlotInfoCache, NUM_SAVE_SLOTS,
};
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct MainMenuRoot;

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
const MAIN_MENU_OPTIONS: &[&str] = &["New Game", "Load Game", "Quit"];

#[cfg(target_arch = "wasm32")]
const MAIN_MENU_OPTIONS: &[&str] = &["New Game", "Load Game"];
const LOAD_MENU_BACK_INDEX: usize = NUM_SAVE_SLOTS;
const MAIN_MENU_MAX_ITEMS: usize = NUM_SAVE_SLOTS + 1;

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
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.18, 0.08)),
        ))
        .with_children(|parent| {
            // Game title
            menu_kit::spawn_menu_title(parent, "HEARTHFIELD", &theme, &font);

            // Subtitle
            parent.spawn((
                Text::new("A Farming & Life Simulator"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.8, 0.6)),
            ));

            // Menu options container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
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

fn current_option_count(mode: MainMenuMode) -> usize {
    match mode {
        MainMenuMode::Root => MAIN_MENU_OPTIONS.len(),
        MainMenuMode::LoadSlots => MAIN_MENU_MAX_ITEMS,
    }
}

fn load_slot_label(slot_info: Option<&crate::save::SaveSlotInfo>) -> String {
    if let Some(info) = slot_info {
        if info.exists {
            format!(
                "Slot {}  Day {} {:?} Y{}  {}g",
                info.slot + 1,
                info.day,
                info.season,
                info.year,
                info.gold
            )
        } else {
            format!("Slot {}  (Empty)", info.slot + 1)
        }
    } else {
        "Slot ?  (Unavailable)".to_string()
    }
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<MainMenuState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE / INTERACTION
// ═══════════════════════════════════════════════════════════════════════

pub fn update_main_menu_visuals(
    state: Option<Res<MainMenuState>>,
    cache: Option<Res<SaveSlotInfoCache>>,
    mut item_query: Query<(&MenuItem, &mut ImageNode, &mut Visibility)>,
    mut text_query: Query<
        (&MenuButtonText, &mut Text, &mut TextColor),
        Without<MainMenuStatusText>,
    >,
    mut status_query: Query<&mut Text, (With<MainMenuStatusText>, Without<MenuButtonText>)>,
) {
    let Some(state) = state else { return };
    let option_count = current_option_count(state.mode);

    for (item, mut image_node, mut visibility) in &mut item_query {
        if item.index >= option_count {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        set_button_visual(&mut image_node, item.index == state.cursor);
    }

    for (btn_text, mut text, mut color) in &mut text_query {
        if btn_text.index >= option_count {
            text.0.clear();
            continue;
        }

        match state.mode {
            MainMenuMode::Root => {
                text.0 = MAIN_MENU_OPTIONS[btn_text.index].to_string();
                color.0 = Color::WHITE;
            }
            MainMenuMode::LoadSlots => {
                if btn_text.index == LOAD_MENU_BACK_INDEX {
                    text.0 = "Back".to_string();
                    color.0 = Color::WHITE;
                } else {
                    let slot_info = cache.as_ref().and_then(|c| c.slots.get(btn_text.index));
                    let slot_exists = slot_info.map(|s| s.exists).unwrap_or(false);
                    text.0 = load_slot_label(slot_info);
                    color.0 = if slot_exists {
                        Color::WHITE
                    } else {
                        Color::srgb(0.6, 0.6, 0.6)
                    };
                }
            }
        }
    }

    let mut status = status_query.single_mut();
    status.0 = state.status_message.clone();
}

pub fn main_menu_navigation(
    action: Res<MenuAction>,
    mut state: Option<ResMut<MainMenuState>>,
    cache: Option<Res<SaveSlotInfoCache>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut load_events: EventWriter<LoadRequestEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    let Some(ref mut state) = state else { return };
    let option_count = current_option_count(state.mode);

    // Pointer hover → set cursor
    if let Some(idx) = action.set_cursor {
        if idx < option_count {
            state.cursor = idx;
        }
    }

    if action.move_down {
        if state.cursor + 1 < option_count {
            state.cursor += 1;
        }
    }
    if action.move_up {
        if state.cursor > 0 {
            state.cursor -= 1;
        }
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
                    next_state.set(GameState::Playing);
                }
                1 => {
                    state.mode = MainMenuMode::LoadSlots;
                    state.cursor = 0;
                    state.status_message.clear();
                }
                #[cfg(not(target_arch = "wasm32"))]
                2 => {
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
