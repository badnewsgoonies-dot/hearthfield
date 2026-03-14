use super::hud::ItemAtlasData;
use super::item_icon_index;
use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct CraftingScreenRoot;

#[derive(Component)]
pub struct CraftingRecipeIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct CraftingRecipeRow {
    pub index: usize,
}

#[derive(Component)]
pub struct CraftingRecipeName {
    pub index: usize,
}

#[derive(Component)]
pub struct CraftingRecipeMaterials {
    pub index: usize,
}

#[derive(Component)]
pub struct CraftingStatusText;

#[derive(Component)]
pub struct CraftingFeaturedIcon;

#[derive(Component)]
pub struct CraftingFeaturedName;

#[derive(Component)]
pub struct CraftingFeaturedDescription;

#[derive(Component)]
pub struct CraftingFeaturedMaterials;

/// Tracks crafting UI state
#[derive(Resource)]
pub struct CraftingUiState {
    pub cursor: usize,
    /// Scroll offset for the 6-row viewport
    pub scroll_offset: usize,
    /// Sorted list of recipe IDs to display
    pub visible_recipes: Vec<String>,
    pub status_message: String,
    pub status_timer: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_crafting_screen(
    mut commands: Commands,
    recipe_registry: Res<RecipeRegistry>,
    unlocked_recipes: Res<UnlockedRecipes>,
    item_registry: Res<ItemRegistry>,
    atlas_data: Res<ItemAtlasData>,
    font_handle: Res<UiFontHandle>,
) {
    // Gather visible recipes: show all unlocked (or all if unlocked list is empty for development)
    let visible: Vec<String> = if unlocked_recipes.ids.is_empty() {
        recipe_registry.recipes.keys().cloned().collect()
    } else {
        unlocked_recipes
            .ids
            .iter()
            .filter(|id| recipe_registry.recipes.contains_key(*id))
            .cloned()
            .collect()
    };

    commands.insert_resource(CraftingUiState {
        cursor: 0,
        scroll_offset: 0,
        visible_recipes: visible.clone(),
        status_message: String::new(),
        status_timer: 0.0,
    });

    commands
        .spawn((
            CraftingScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(760.0),
                        height: Val::Px(560.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(6.0),
                        border: UiRect::all(Val::Px(3.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.95)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.25)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("CRAFTING"),
                        TextFont {
                            font: font_handle.0.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.6)),
                    ));

                    // Status / feedback text
                    panel.spawn((
                        CraftingStatusText,
                        Text::new(""),
                        TextFont {
                            font: font_handle.0.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.9, 0.5)),
                    ));

                    // Selected recipe feature card
                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(170.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(16.0),
                                padding: UiRect::all(Val::Px(14.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::bottom(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.18, 0.13, 0.09, 0.92)),
                            BorderColor(Color::srgb(0.56, 0.39, 0.19)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                CraftingFeaturedIcon,
                                Node {
                                    width: Val::Px(48.0),
                                    height: Val::Px(48.0),
                                    margin: UiRect::top(Val::Px(2.0)),
                                    ..default()
                                },
                            ));

                            card.spawn((Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                flex_grow: 1.0,
                                ..default()
                            },))
                                .with_children(|details| {
                                    details.spawn((
                                        CraftingFeaturedName,
                                        Text::new(""),
                                        TextFont {
                                            font: font_handle.0.clone(),
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(1.0, 0.9, 0.72)),
                                    ));
                                    details.spawn((
                                        CraftingFeaturedDescription,
                                        Text::new(""),
                                        TextFont {
                                            font: font_handle.0.clone(),
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.76, 0.68)),
                                    ));
                                    details.spawn((
                                        CraftingFeaturedMaterials,
                                        Text::new(""),
                                        TextFont {
                                            font: font_handle.0.clone(),
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.72, 0.68, 0.62)),
                                    ));
                                });
                        });

                    panel
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },))
                        .with_children(|list| {
                            // Recipe list (up to 6 visible rows)
                            for i in 0..6 {
                                list.spawn((
                                    CraftingRecipeRow { index: i },
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(40.0),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::horizontal(Val::Px(8.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6)),
                                ))
                                .with_children(|row| {
                                    // Item icon
                                    let mut icon_cmd = row.spawn((
                                        CraftingRecipeIcon { index: i },
                                        Node {
                                            width: Val::Px(24.0),
                                            height: Val::Px(24.0),
                                            margin: UiRect::right(Val::Px(6.0)),
                                            ..default()
                                        },
                                    ));
                                    if atlas_data.loaded {
                                        if let Some(recipe_id) = visible.get(i) {
                                            if let Some(recipe) =
                                                recipe_registry.recipes.get(recipe_id.as_str())
                                            {
                                                if let Some(item_def) =
                                                    item_registry.get(&recipe.result)
                                                {
                                                    icon_cmd.insert(ImageNode {
                                                        image: atlas_data.image.clone(),
                                                        texture_atlas: Some(TextureAtlas {
                                                            layout: atlas_data.layout.clone(),
                                                            index: item_icon_index(
                                                                item_def.sprite_index,
                                                            ),
                                                        }),
                                                        ..default()
                                                    });
                                                }
                                            }
                                        }
                                    }

                                    row.spawn((
                                        CraftingRecipeName { index: i },
                                        Text::new(""),
                                        TextFont {
                                            font: font_handle.0.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    row.spawn((
                                        CraftingRecipeMaterials { index: i },
                                        Text::new(""),
                                        TextFont {
                                            font: font_handle.0.clone(),
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                    ));
                                });
                            }
                        });

                    // Hint
                    panel.spawn((
                        Text::new("Up/Down: Select | Enter: Craft | Esc: Close"),
                        TextFont {
                            font: font_handle.0.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

pub fn despawn_crafting_screen(
    mut commands: Commands,
    query: Query<Entity, With<CraftingScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<CraftingUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_crafting_display(
    mut commands: Commands,
    ui_state: Option<Res<CraftingUiState>>,
    recipe_registry: Res<RecipeRegistry>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    atlas_data: Res<ItemAtlasData>,
    mut name_query: Query<
        (&CraftingRecipeName, &mut Text, &mut TextColor),
        Without<CraftingRecipeMaterials>,
    >,
    mut mat_query: Query<
        (&CraftingRecipeMaterials, &mut Text, &mut TextColor),
        Without<CraftingRecipeName>,
    >,
    mut row_query: Query<(&CraftingRecipeRow, &mut BackgroundColor)>,
    mut status_query: Query<
        &mut Text,
        (
            With<CraftingStatusText>,
            Without<CraftingRecipeName>,
            Without<CraftingRecipeMaterials>,
        ),
    >,
    mut featured_name_query: Query<
        &mut Text,
        (
            With<CraftingFeaturedName>,
            Without<CraftingFeaturedDescription>,
            Without<CraftingFeaturedMaterials>,
        ),
    >,
    mut featured_description_query: Query<
        &mut Text,
        (
            With<CraftingFeaturedDescription>,
            Without<CraftingFeaturedName>,
            Without<CraftingFeaturedMaterials>,
        ),
    >,
    mut featured_materials_query: Query<
        &mut Text,
        (
            With<CraftingFeaturedMaterials>,
            Without<CraftingFeaturedName>,
            Without<CraftingFeaturedDescription>,
        ),
    >,
    featured_icon_query: Query<Entity, With<CraftingFeaturedIcon>>,
    icon_query: Query<(Entity, &CraftingRecipeIcon)>,
) {
    let Some(ui_state) = ui_state else { return };

    // Status message
    for mut text in &mut status_query {
        **text = ui_state.status_message.clone();
    }

    let selected_recipe = ui_state
        .visible_recipes
        .get(ui_state.cursor)
        .and_then(|recipe_id| recipe_registry.recipes.get(recipe_id));

    let featured_description = selected_recipe
        .and_then(|recipe| item_registry.get(&recipe.result))
        .map(|item| item.description.clone())
        .unwrap_or_default();
    let featured_materials = selected_recipe
        .map(|recipe| {
            let mut parts = Vec::new();
            for (item_id, qty) in &recipe.ingredients {
                let name = item_registry
                    .get(item_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| item_id.clone());
                let has = inventory.count(item_id) as u8;
                parts.push(format!("{} ({}/{})", name, has, qty));
            }
            if parts.is_empty() {
                "Materials: None".to_string()
            } else {
                format!("Materials: {}", parts.join(", "))
            }
        })
        .unwrap_or_default();

    for mut text in &mut featured_name_query {
        **text = selected_recipe
            .map(|recipe| recipe.name.clone())
            .unwrap_or_default();
    }

    for mut text in &mut featured_description_query {
        **text = featured_description.clone();
    }

    for mut text in &mut featured_materials_query {
        **text = featured_materials.clone();
    }

    for entity in &featured_icon_query {
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<ImageNode>();
        if atlas_data.loaded {
            if let Some(recipe) = selected_recipe {
                if let Some(item_def) = item_registry.get(&recipe.result) {
                    entity_commands.insert(ImageNode {
                        image: atlas_data.image.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_data.layout.clone(),
                            index: item_icon_index(item_def.sprite_index),
                        }),
                        ..default()
                    });
                }
            }
        }
    }

    for (name_comp, mut text, mut color) in &mut name_query {
        let idx = name_comp.index + ui_state.scroll_offset;
        if idx < ui_state.visible_recipes.len() {
            let recipe_id = &ui_state.visible_recipes[idx];
            if let Some(recipe) = recipe_registry.recipes.get(recipe_id) {
                let can_craft = can_craft_recipe(recipe, &inventory);
                **text = recipe.name.clone();
                if can_craft {
                    *color = TextColor(Color::srgb(0.5, 1.0, 0.5));
                } else {
                    *color = TextColor(Color::srgb(0.6, 0.4, 0.4));
                }
            }
        } else {
            **text = String::new();
        }
    }

    for (mat_comp, mut text, mut color) in &mut mat_query {
        let idx = mat_comp.index + ui_state.scroll_offset;
        if idx < ui_state.visible_recipes.len() {
            let recipe_id = &ui_state.visible_recipes[idx];
            if let Some(recipe) = recipe_registry.recipes.get(recipe_id) {
                let mut parts = Vec::new();
                for (item_id, qty) in &recipe.ingredients {
                    let name = item_registry
                        .get(item_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| item_id.clone());
                    let has = inventory.count(item_id) as u8;
                    parts.push(format!("{} ({}/{})", name, has, qty));
                }
                **text = parts.join(", ");
                let can = can_craft_recipe(recipe, &inventory);
                if can {
                    *color = TextColor(Color::srgb(0.6, 0.7, 0.6));
                } else {
                    *color = TextColor(Color::srgb(0.5, 0.4, 0.4));
                }
            }
        } else {
            **text = String::new();
        }
    }

    // Cursor highlight
    for (row, mut bg) in &mut row_query {
        if row.index + ui_state.scroll_offset == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.42, 0.26, 0.12, 0.96));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6));
        }
    }

    // Update recipe row icons based on scroll offset
    for (entity, icon) in &icon_query {
        let idx = icon.index + ui_state.scroll_offset;
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<ImageNode>();
        if atlas_data.loaded {
            if let Some(recipe_id) = ui_state.visible_recipes.get(idx) {
                if let Some(recipe) = recipe_registry.recipes.get(recipe_id.as_str()) {
                    if let Some(item_def) = item_registry.get(&recipe.result) {
                        entity_commands.insert(ImageNode {
                            image: atlas_data.image.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_data.layout.clone(),
                                index: item_icon_index(item_def.sprite_index),
                            }),
                            ..default()
                        });
                    }
                }
            }
        }
    }
}

fn can_craft_recipe(recipe: &Recipe, inventory: &Inventory) -> bool {
    recipe
        .ingredients
        .iter()
        .all(|(item_id, qty)| inventory.has(item_id, *qty))
}

pub fn crafting_navigation(
    action: Res<MenuAction>,
    mut ui_state: Option<ResMut<CraftingUiState>>,
    recipe_registry: Res<RecipeRegistry>,
    inventory: Res<Inventory>,
    mut craft_events: EventWriter<crate::crafting::CraftItemEvent>,
) {
    let Some(ref mut ui_state) = ui_state else {
        return;
    };

    let max = ui_state.visible_recipes.len();

    if action.move_down && max > 0 && ui_state.cursor < max - 1 {
        ui_state.cursor += 1;
    }
    if action.move_up && ui_state.cursor > 0 {
        ui_state.cursor -= 1;
    }

    // Keep cursor visible in the 6-row viewport
    if ui_state.cursor >= ui_state.scroll_offset + 6 {
        ui_state.scroll_offset = ui_state.cursor - 5;
    }
    if ui_state.cursor < ui_state.scroll_offset {
        ui_state.scroll_offset = ui_state.cursor;
    }

    if action.activate && ui_state.cursor < ui_state.visible_recipes.len() {
        let recipe_id = ui_state.visible_recipes[ui_state.cursor].clone();
        if let Some(recipe) = recipe_registry.recipes.get(&recipe_id) {
            if can_craft_recipe(recipe, &inventory) {
                craft_events.send(crate::crafting::CraftItemEvent {
                    recipe_id: recipe_id.clone(),
                });
                ui_state.status_message = "Crafting...".to_string();
                ui_state.status_timer = 2.0;
            } else {
                ui_state.status_message = "Not enough materials!".to_string();
                ui_state.status_timer = 2.0;
            }
        }
    }
}

pub fn crafting_status_timer(time: Res<Time>, mut ui_state: Option<ResMut<CraftingUiState>>) {
    let Some(ref mut ui_state) = ui_state else {
        return;
    };
    if ui_state.status_timer > 0.0 {
        ui_state.status_timer -= time.delta_secs();
        if ui_state.status_timer <= 0.0 {
            ui_state.status_message.clear();
        }
    }
}
