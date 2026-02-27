use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct CraftingScreenRoot;

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

/// Tracks crafting UI state
#[derive(Resource)]
pub struct CraftingUiState {
    pub cursor: usize,
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
        visible_recipes: visible,
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
                        width: Val::Px(550.0),
                        height: Val::Px(420.0),
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
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.9, 0.5)),
                    ));

                    // Recipe list (up to 10 visible rows)
                    for i in 0..10 {
                        panel
                            .spawn((
                                CraftingRecipeRow { index: i },
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(32.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6)),
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    CraftingRecipeName { index: i },
                                    Text::new(""),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                                row.spawn((
                                    CraftingRecipeMaterials { index: i },
                                    Text::new(""),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                ));
                            });
                    }

                    // Hint
                    panel.spawn((
                        Text::new("Up/Down: Select | Enter: Craft | Esc: Close"),
                        TextFont {
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
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<CraftingUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

pub fn update_crafting_display(
    ui_state: Option<Res<CraftingUiState>>,
    recipe_registry: Res<RecipeRegistry>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut name_query: Query<(&CraftingRecipeName, &mut Text, &mut TextColor), Without<CraftingRecipeMaterials>>,
    mut mat_query: Query<(&CraftingRecipeMaterials, &mut Text, &mut TextColor), Without<CraftingRecipeName>>,
    mut row_query: Query<(&CraftingRecipeRow, &mut BackgroundColor)>,
    mut status_query: Query<&mut Text, (With<CraftingStatusText>, Without<CraftingRecipeName>, Without<CraftingRecipeMaterials>)>,
) {
    let Some(ui_state) = ui_state else { return };

    // Status message
    for mut text in &mut status_query {
        **text = ui_state.status_message.clone();
    }

    for (name_comp, mut text, mut color) in &mut name_query {
        let idx = name_comp.index;
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
        let idx = mat_comp.index;
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
        if row.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.9));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6));
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
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
) {
    let Some(ref mut ui_state) = ui_state else { return };

    let max = ui_state.visible_recipes.len();

    if action.move_down {
        if max > 0 && ui_state.cursor < max - 1 {
            ui_state.cursor += 1;
        }
    }
    if action.move_up {
        if ui_state.cursor > 0 {
            ui_state.cursor -= 1;
        }
    }

    if action.activate {
        if ui_state.cursor < ui_state.visible_recipes.len() {
            let recipe_id = ui_state.visible_recipes[ui_state.cursor].clone();
            if let Some(recipe) = recipe_registry.recipes.get(&recipe_id) {
                if can_craft_recipe(recipe, &inventory) {
                    // Consume materials
                    for (item_id, qty) in &recipe.ingredients {
                        inventory.try_remove(item_id, *qty);
                    }
                    // Add result
                    let max_stack = item_registry
                        .get(&recipe.result)
                        .map(|d| d.stack_size)
                        .unwrap_or(99);
                    let overflow = inventory.try_add(&recipe.result, recipe.result_quantity, max_stack);
                    if overflow > 0 {
                        ui_state.status_message = "Crafted! (some items didn't fit)".to_string();
                    } else {
                        ui_state.status_message = format!("Crafted {}!", recipe.name);
                    }
                    ui_state.status_timer = 2.0;
                } else {
                    ui_state.status_message = "Not enough materials!".to_string();
                    ui_state.status_timer = 2.0;
                }
            }
        }
    }
}

pub fn crafting_status_timer(
    time: Res<Time>,
    mut ui_state: Option<ResMut<CraftingUiState>>,
) {
    let Some(ref mut ui_state) = ui_state else { return };
    if ui_state.status_timer > 0.0 {
        ui_state.status_timer -= time.delta_secs();
        if ui_state.status_timer <= 0.0 {
            ui_state.status_message.clear();
        }
    }
}
