use bevy::prelude::*;
use crate::shared::*;
use super::UiFontHandle;
use super::hud::ItemAtlasData;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct InventoryScreenRoot;

#[derive(Component)]
pub struct InventoryGridSlot {
    pub index: usize,
}

#[derive(Component)]
pub struct InventorySlotItemName {
    pub index: usize,
}

#[derive(Component)]
pub struct InventorySlotQuantity {
    pub index: usize,
}

#[derive(Component)]
pub struct InventorySlotIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct InventorySlotBg {
    pub index: usize,
}

/// Tracks which slot is currently selected/hovered in the inventory UI
#[derive(Resource, Default)]
pub struct InventoryUiState {
    pub cursor_slot: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_inventory_screen(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    atlas_data: Res<ItemAtlasData>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
) {
    commands.insert_resource(InventoryUiState { cursor_slot: 0 });

    let font = font_handle.0.clone();

    commands
        .spawn((
            InventoryScreenRoot,
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
            // Main inventory panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(620.0),
                        height: Val::Px(320.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.95)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.25)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("INVENTORY"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.6)),
                    ));

                    // Hint text
                    panel.spawn((
                        Text::new("WASD/Arrows: Move | Esc: Close"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));

                    // Grid: 3 rows x 12 columns = 36 slots
                    for row in 0..3 {
                        panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    column_gap: Val::Px(3.0),
                                    ..default()
                                },
                            ))
                            .with_children(|row_node| {
                                for col in 0..12 {
                                    let index = row * 12 + col;
                                    row_node
                                        .spawn((
                                            InventoryGridSlot { index },
                                            InventorySlotBg { index },
                                            Node {
                                                width: Val::Px(46.0),
                                                height: Val::Px(54.0),
                                                flex_direction: FlexDirection::Column,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                padding: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9)),
                                            BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.7)),
                                        ))
                                        .with_children(|slot| {
                                            // Item icon
                                            let atlas_index = if index < inventory.slots.len() {
                                                inventory.slots[index]
                                                    .as_ref()
                                                    .and_then(|s| item_registry.get(&s.item_id))
                                                    .map(|def| def.sprite_index as usize)
                                                    .unwrap_or(0)
                                            } else { 0 };
                                            let has_item = index < inventory.slots.len()
                                                && inventory.slots[index].is_some();
                                            if atlas_data.loaded {
                                                slot.spawn((
                                                    InventorySlotIcon { index },
                                                    ImageNode {
                                                        image: atlas_data.image.clone(),
                                                        texture_atlas: Some(TextureAtlas {
                                                            layout: atlas_data.layout.clone(),
                                                            index: atlas_index,
                                                        }),
                                                        ..default()
                                                    },
                                                    Node {
                                                        width: Val::Px(28.0),
                                                        height: Val::Px(28.0),
                                                        ..default()
                                                    },
                                                    if has_item { Visibility::Inherited } else { Visibility::Hidden },
                                                ));
                                            }
                                            // Item name
                                            slot.spawn((
                                                InventorySlotItemName { index },
                                                Text::new(""),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 9.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                            // Quantity
                                            slot.spawn((
                                                InventorySlotQuantity { index },
                                                Text::new(""),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 8.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                            ));
                                        });
                                }
                            });
                    }
                });
        });
}

pub fn despawn_inventory_screen(
    mut commands: Commands,
    query: Query<Entity, With<InventoryScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<InventoryUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

pub fn update_inventory_slots(
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    _atlas_data: Res<ItemAtlasData>,
    mut item_text_query: Query<(&InventorySlotItemName, &mut Text), Without<InventorySlotQuantity>>,
    mut qty_text_query: Query<(&InventorySlotQuantity, &mut Text), Without<InventorySlotItemName>>,
    mut icon_query: Query<(&InventorySlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    // Update icons
    for (icon, mut img, mut vis) in &mut icon_query {
        let idx = icon.index;
        if idx < inventory.slots.len() {
            if let Some(ref slot_data) = inventory.slots[idx] {
                if let Some(def) = item_registry.get(&slot_data.item_id) {
                    if let Some(ref mut atlas) = img.texture_atlas {
                        atlas.index = def.sprite_index as usize;
                    }
                    *vis = Visibility::Inherited;
                    continue;
                }
            }
        }
        *vis = Visibility::Hidden;
    }
    for (item_name, mut text) in &mut item_text_query {
        let idx = item_name.index;
        if idx < inventory.slots.len() {
            if let Some(ref slot_data) = inventory.slots[idx] {
                let name = item_registry
                    .get(&slot_data.item_id)
                    .map(|def| {
                        if def.name.len() > 6 {
                            format!("{}.", &def.name[..5])
                        } else {
                            def.name.clone()
                        }
                    })
                    .unwrap_or_else(|| slot_data.item_id.chars().take(6).collect());
                **text = name;
            } else {
                **text = String::new();
            }
        }
    }

    for (qty_text, mut text) in &mut qty_text_query {
        let idx = qty_text.index;
        if idx < inventory.slots.len() {
            if let Some(ref slot_data) = inventory.slots[idx] {
                if slot_data.quantity > 0 {
                    **text = format!("x{}", slot_data.quantity);
                } else {
                    **text = String::new();
                }
            } else {
                **text = String::new();
            }
        }
    }
}

pub fn update_inventory_cursor(
    ui_state: Option<Res<InventoryUiState>>,
    mut slot_query: Query<(&InventorySlotBg, &mut BackgroundColor, &mut BorderColor)>,
) {
    let Some(ui_state) = ui_state else { return };
    for (slot, mut bg, mut border) in &mut slot_query {
        if slot.index == ui_state.cursor_slot {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.7));
        }
    }
}

pub fn inventory_navigation(
    action: Res<MenuAction>,
    mut ui_state: Option<ResMut<InventoryUiState>>,
) {
    let Some(ref mut ui_state) = ui_state else { return };
    let cur = ui_state.cursor_slot;
    let col = cur % 12;
    let row = cur / 12;

    if action.move_right {
        if col < 11 {
            ui_state.cursor_slot = row * 12 + col + 1;
        }
    }
    if action.move_left {
        if col > 0 {
            ui_state.cursor_slot = row * 12 + col - 1;
        }
    }
    if action.move_down {
        if row < 2 {
            ui_state.cursor_slot = (row + 1) * 12 + col;
        }
    }
    if action.move_up {
        if row > 0 {
            ui_state.cursor_slot = (row - 1) * 12 + col;
        }
    }
}
