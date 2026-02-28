//! Shared menu builder helpers and atlas assets.
//!
//! Provides `MenuAssets` (loaded once at Startup), builder functions for
//! atlas-backed menu buttons with Bevy 0.15 pointer observers, and common
//! title/footer text spawners.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS — atlas frame indices for play_button.png (192×64, 2×2 grid)
// ═══════════════════════════════════════════════════════════════════════

pub const BTN_NORMAL: usize = 0;
pub const BTN_SELECTED: usize = 1;
#[allow(dead_code)]
pub const BTN_PRESSED: usize = 2;
#[allow(dead_code)]
pub const BTN_DISABLED: usize = 3;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Shared button atlas handles, loaded once at Startup.
#[derive(Resource)]
pub struct MenuAssets {
    pub button_image: Handle<Image>,
    pub button_layout: Handle<TextureAtlasLayout>,
}

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker for the text child inside a menu button.
#[derive(Component)]
pub struct MenuButtonText {
    pub index: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// STARTUP SYSTEM
// ═══════════════════════════════════════════════════════════════════════

/// Loads the play_button.png atlas once at Startup.
pub fn load_menu_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = asset_server.load("ui/play_button.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(96, 32),
        2,
        2,
        None,
        None,
    ));
    commands.insert_resource(MenuAssets {
        button_image: image,
        button_layout: layout,
    });
}

// ═══════════════════════════════════════════════════════════════════════
// BUILDER HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Spawns an atlas-backed menu button with pointer observers for
/// mouse/touch support. Returns the button Entity.
pub fn spawn_menu_button(
    parent: &mut ChildBuilder,
    index: usize,
    label: &str,
    assets: &MenuAssets,
    theme: &MenuTheme,
    font: &Handle<Font>,
) -> Entity {
    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(theme.button_width),
                height: Val::Px(theme.button_height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                image: assets.button_image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.button_layout.clone(),
                    index: BTN_NORMAL,
                }),
                ..default()
            },
        ))
        .observe(on_button_over)
        .observe(on_button_click)
        .with_children(|btn| {
            btn.spawn((
                MenuButtonText { index },
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: theme.button_font_size,
                    ..default()
                },
                TextColor(theme.text_color),
                PickingBehavior::IGNORE,
            ));
        })
        .id()
}

/// Spawns a title text node styled by the theme.
pub fn spawn_menu_title(
    parent: &mut ChildBuilder,
    text: &str,
    theme: &MenuTheme,
    font: &Handle<Font>,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font: font.clone(),
            font_size: theme.title_font_size,
            ..default()
        },
        TextColor(theme.text_color_selected),
        PickingBehavior::IGNORE,
    ));
}

/// Spawns a hint/footer text node.
pub fn spawn_menu_footer(
    parent: &mut ChildBuilder,
    text: &str,
    theme: &MenuTheme,
    font: &Handle<Font>,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font: font.clone(),
            font_size: theme.hint_font_size,
            ..default()
        },
        TextColor(theme.text_color_disabled),
        PickingBehavior::IGNORE,
    ));
}

/// Updates a button's atlas index based on selection state.
pub fn set_button_visual(image_node: &mut ImageNode, selected: bool) {
    if let Some(ref mut atlas) = image_node.texture_atlas {
        atlas.index = if selected { BTN_SELECTED } else { BTN_NORMAL };
    }
}

// ═══════════════════════════════════════════════════════════════════════
// POINTER OBSERVERS — set MenuAction from mouse/touch
// ═══════════════════════════════════════════════════════════════════════

fn on_button_over(
    trigger: Trigger<Pointer<Over>>,
    query: Query<&MenuItem>,
    mut action: ResMut<MenuAction>,
) {
    if let Ok(item) = query.get(trigger.entity()) {
        action.set_cursor = Some(item.index);
    }
}

fn on_button_click(
    trigger: Trigger<Pointer<Click>>,
    query: Query<&MenuItem>,
    mut action: ResMut<MenuAction>,
) {
    if query.get(trigger.entity()).is_ok() {
        action.activate = true;
    }
}
