use bevy::prelude::*;
use crate::shared::*;
use super::UiFontHandle;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MainMenuItem {
    pub index: usize,
}

#[derive(Component)]
pub struct MainMenuItemText {
    pub index: usize,
}

/// Tracks main menu selection
#[derive(Resource)]
pub struct MainMenuState {
    pub cursor: usize,
}

/// Stores the play button atlas layout for button backgrounds
#[derive(Resource)]
pub struct PlayButtonAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

const MAIN_MENU_OPTIONS: &[&str] = &["New Game", "Load Game", "Quit"];

// Play button atlas: 192x64px image, 2 columns x 2 rows of 96x32 button states
// Index 0 = normal, 1 = hovered/selected, 2 = pressed, 3 = disabled
const PLAY_BUTTON_NORMAL: usize = 0;
const PLAY_BUTTON_SELECTED: usize = 1;

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_main_menu(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.insert_resource(MainMenuState { cursor: 0 });

    // Load the play button sprite sheet (192x64, 2x2 grid of 96x32 buttons)
    let button_image = asset_server.load("ui/play_button.png");
    let button_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 32),
        2,
        2,
        None,
        None,
    ));
    commands.insert_resource(PlayButtonAtlas {
        image: button_image.clone(),
        layout: button_layout.clone(),
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
            parent.spawn((
                Text::new("HEARTHFIELD"),
                TextFont {
                    font: font.clone(),
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));

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
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(|menu| {
                    for (i, label) in MAIN_MENU_OPTIONS.iter().enumerate() {
                        // Each menu item uses the play button sprite as background
                        menu.spawn((
                                MainMenuItem { index: i },
                                Node {
                                    width: Val::Px(240.0),
                                    height: Val::Px(42.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ImageNode {
                                    image: button_image.clone(),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: button_layout.clone(),
                                        index: PLAY_BUTTON_NORMAL,
                                    }),
                                    ..default()
                                },
                            ))
                            .with_children(|item| {
                                item.spawn((
                                    MainMenuItemText { index: i },
                                    Text::new(*label),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

            // Version text
            parent.spawn((
                Text::new("v0.1.0 - Early Development"),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.45, 0.35)),
            ));
        });
}

pub fn despawn_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<MainMenuState>();
    commands.remove_resource::<PlayButtonAtlas>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE / INTERACTION
// ═══════════════════════════════════════════════════════════════════════

pub fn update_main_menu_visuals(
    state: Option<Res<MainMenuState>>,
    mut query: Query<(&MainMenuItem, &mut ImageNode)>,
) {
    let Some(state) = state else { return };
    for (item, mut image_node) in &mut query {
        if let Some(ref mut atlas) = image_node.texture_atlas {
            if item.index == state.cursor {
                atlas.index = PLAY_BUTTON_SELECTED;
            } else {
                atlas.index = PLAY_BUTTON_NORMAL;
            }
        }
    }
}

pub fn main_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<MainMenuState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
) {
    let Some(ref mut state) = state else { return };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if state.cursor < MAIN_MENU_OPTIONS.len() - 1 {
            state.cursor += 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if state.cursor > 0 {
            state.cursor -= 1;
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        match state.cursor {
            0 => {
                // New Game — data is already loaded, go straight to Playing
                next_state.set(GameState::Playing);
            }
            1 => {
                // Load Game — for now, same as New Game
                next_state.set(GameState::Playing);
            }
            2 => {
                // Quit
                app_exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}
