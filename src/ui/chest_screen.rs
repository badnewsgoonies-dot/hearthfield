//! Chest UI overlay -- displays a split view of the player inventory (left)
//! and the open chest contents (right). Transfers happen via Tab, arrow keys,
//! and Enter. Escape closes the chest.
//!
//! The world domain owns `ChestInteraction` which tracks the currently open
//! chest entity. We import it so the UI knows which chest to display.

use bevy::prelude::*;
use crate::shared::*;
use crate::world::chests::ChestInteraction;
use super::UiFontHandle;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct ChestScreenRoot;

#[derive(Component)]
pub struct ChestInvSlotText {
    pub index: usize,
}

#[derive(Component)]
pub struct ChestStorageSlotText {
    pub index: usize,
}

#[derive(Component)]
pub struct ChestInvSlotBg {
    pub index: usize,
}

#[derive(Component)]
pub struct ChestStorageSlotBg {
    pub index: usize,
}

/// Local UI state for the chest screen.
#[derive(Resource)]
pub struct ChestUiState {
    /// false = inventory panel (left), true = chest panel (right).
    pub on_chest_side: bool,
    /// Current cursor position (slot index) within the active panel.
    pub cursor: usize,
}

const VISIBLE_SLOTS: usize = 12;

// ═══════════════════════════════════════════════════════════════════════
// SPAWN
// ═══════════════════════════════════════════════════════════════════════

fn spawn_chest_ui(commands: &mut Commands, font: Handle<Font>) {
    commands.insert_resource(ChestUiState {
        on_chest_side: false,
        cursor: 0,
    });

    commands
        .spawn((
            ChestScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(100),
        ))
        .with_children(|root| {
            // Two panels side by side
            root.spawn(Node {
                width: Val::Px(720.0),
                height: Val::Px(380.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Stretch,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                // LEFT: Player inventory
                spawn_inventory_panel(row, font.clone());
                // RIGHT: Chest contents
                spawn_chest_panel(row, font.clone());
            });

            // Hint text
            root.spawn((
                Text::new(
                    "Tab: Switch panel | Up/Down: Select | Enter: Transfer | Esc: Close",
                ),
                TextFont {
                    font,
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));
        });
}

fn spawn_inventory_panel(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Px(340.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.95)),
            BorderColor(Color::srgb(0.5, 0.4, 0.25)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("INVENTORY"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.6)),
            ));

            for i in 0..VISIBLE_SLOTS {
                panel
                    .spawn((
                        ChestInvSlotBg { index: i },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9)),
                        BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.5)),
                    ))
                    .with_children(|slot_row| {
                        slot_row.spawn((
                            ChestInvSlotText { index: i },
                            Text::new("---"),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn spawn_chest_panel(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Px(340.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.95)),
            BorderColor(Color::srgb(0.5, 0.4, 0.25)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("CHEST"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.6)),
            ));

            for i in 0..VISIBLE_SLOTS {
                panel
                    .spawn((
                        ChestStorageSlotBg { index: i },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9)),
                        BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.5)),
                    ))
                    .with_children(|slot_row| {
                        slot_row.spawn((
                            ChestStorageSlotText { index: i },
                            Text::new("---"),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

// ═══════════════════════════════════════════════════════════════════════
// DESPAWN
// ═══════════════════════════════════════════════════════════════════════

fn despawn_chest_ui(commands: &mut Commands, query: &Query<Entity, With<ChestScreenRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ChestUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// LIFECYCLE -- reactive spawn/despawn based on ChestInteraction
// ═══════════════════════════════════════════════════════════════════════

/// Each frame, check whether the chest UI needs to be spawned or despawned
/// based on the `ChestInteraction` resource.
pub fn update_chest_ui_lifecycle(
    mut commands: Commands,
    interaction: Res<ChestInteraction>,
    font_handle: Res<UiFontHandle>,
    existing_ui: Query<Entity, With<ChestScreenRoot>>,
) {
    let is_open = interaction.is_open();
    let ui_exists = !existing_ui.is_empty();

    if is_open && !ui_exists {
        spawn_chest_ui(&mut commands, font_handle.0.clone());
    } else if !is_open && ui_exists {
        despawn_chest_ui(&mut commands, &existing_ui);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE DISPLAY
// ═══════════════════════════════════════════════════════════════════════

/// Update the inventory-side slot texts.
pub fn update_chest_inv_display(
    interaction: Res<ChestInteraction>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut text_query: Query<(&ChestInvSlotText, &mut Text)>,
) {
    if !interaction.is_open() {
        return;
    }
    for (slot_comp, mut text) in &mut text_query {
        let idx = slot_comp.index;
        if idx < inventory.slots.len() {
            if let Some(ref slot_data) = inventory.slots[idx] {
                let name = item_registry
                    .get(&slot_data.item_id)
                    .map(|def| def.name.clone())
                    .unwrap_or_else(|| slot_data.item_id.clone());
                let truncated = if name.len() > 20 {
                    format!("{}...", &name[..17])
                } else {
                    name
                };
                **text = format!("{} x{}", truncated, slot_data.quantity);
            } else {
                **text = "---".to_string();
            }
        } else {
            **text = "---".to_string();
        }
    }
}

/// Update the chest-side slot texts.
pub fn update_chest_storage_display(
    interaction: Res<ChestInteraction>,
    chest_query: Query<&StorageChest>,
    item_registry: Res<ItemRegistry>,
    mut text_query: Query<(&ChestStorageSlotText, &mut Text)>,
) {
    let Some(entity) = interaction.entity else {
        return;
    };
    let Ok(chest) = chest_query.get(entity) else {
        return;
    };

    for (slot_comp, mut text) in &mut text_query {
        let idx = slot_comp.index;
        if idx < chest.slots.len() {
            if let Some(ref stack) = chest.slots[idx] {
                let name = item_registry
                    .get(&stack.item_id)
                    .map(|def| def.name.clone())
                    .unwrap_or_else(|| stack.item_id.clone());
                let truncated = if name.len() > 20 {
                    format!("{}...", &name[..17])
                } else {
                    name
                };
                let quality_str = match stack.quality {
                    ItemQuality::Normal => "",
                    ItemQuality::Silver => " [S]",
                    ItemQuality::Gold => " [G]",
                    ItemQuality::Iridium => " [I]",
                };
                **text = format!("{}{} x{}", truncated, quality_str, stack.quantity);
            } else {
                **text = "---".to_string();
            }
        } else {
            **text = "---".to_string();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CURSOR HIGHLIGHT
// ═══════════════════════════════════════════════════════════════════════

pub fn update_chest_cursor(
    ui_state: Option<Res<ChestUiState>>,
    mut inv_bg_query: Query<
        (&ChestInvSlotBg, &mut BackgroundColor, &mut BorderColor),
        Without<ChestStorageSlotBg>,
    >,
    mut chest_bg_query: Query<
        (&ChestStorageSlotBg, &mut BackgroundColor, &mut BorderColor),
        Without<ChestInvSlotBg>,
    >,
) {
    let Some(ui_state) = ui_state else { return };

    for (slot, mut bg, mut border) in &mut inv_bg_query {
        if !ui_state.on_chest_side && slot.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.5));
        }
    }

    for (slot, mut bg, mut border) in &mut chest_bg_query {
        if ui_state.on_chest_side && slot.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.9));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.5));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INPUT HANDLING
// ═══════════════════════════════════════════════════════════════════════

/// Handles navigation and item transfer within the chest screen.
pub fn handle_chest_input(
    player_input: Res<PlayerInput>,
    interaction: Res<ChestInteraction>,
    mut ui_state: Option<ResMut<ChestUiState>>,
    mut inventory: ResMut<Inventory>,
    mut chest_query: Query<&mut StorageChest>,
    item_registry: Res<ItemRegistry>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if !interaction.is_open() {
        return;
    }
    let Some(ref mut ui_state) = ui_state else {
        return;
    };
    let Some(entity) = interaction.entity else {
        return;
    };

    // Tab: switch panels (Tab is mapped to ui_left in Gameplay context)
    if player_input.ui_left {
        ui_state.on_chest_side = !ui_state.on_chest_side;
        ui_state.cursor = 0;
    }

    // Up/Down navigation
    if player_input.ui_up && ui_state.cursor > 0 {
        ui_state.cursor -= 1;
    }
    if player_input.ui_down && ui_state.cursor < VISIBLE_SLOTS - 1 {
        ui_state.cursor += 1;
    }

    // Enter: transfer item
    if player_input.ui_confirm {
        let Ok(mut chest) = chest_query.get_mut(entity) else {
            return;
        };
        let idx = ui_state.cursor;

        if ui_state.on_chest_side {
            transfer_from_chest_to_inventory(idx, &mut chest, &mut inventory, &item_registry);
        } else {
            transfer_from_inventory_to_chest(idx, &mut inventory, &mut chest);
        }

        sfx_events.send(PlaySfxEvent {
            sfx_id: "item_pickup".to_string(),
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TRANSFER HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn transfer_from_chest_to_inventory(
    idx: usize,
    chest: &mut StorageChest,
    inventory: &mut Inventory,
    item_registry: &ItemRegistry,
) {
    if idx >= chest.slots.len() {
        return;
    }
    let Some(stack) = chest.slots[idx].take() else {
        return;
    };

    let max_stack = item_registry
        .get(&stack.item_id)
        .map(|def| def.stack_size)
        .unwrap_or(99);

    let remaining = inventory.try_add(&stack.item_id, stack.quantity, max_stack);
    if remaining > 0 {
        chest.slots[idx] = Some(QualityStack {
            item_id: stack.item_id,
            quantity: remaining,
            quality: stack.quality,
        });
    }
}

fn transfer_from_inventory_to_chest(
    idx: usize,
    inventory: &mut Inventory,
    chest: &mut StorageChest,
) {
    if idx >= inventory.slots.len() {
        return;
    }
    let Some(inv_slot) = inventory.slots[idx].take() else {
        return;
    };

    // Try to stack onto existing chest slots with same item
    let mut remaining = inv_slot.quantity;
    for chest_slot in chest.slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if let Some(ref mut stack) = chest_slot {
            if stack.item_id == inv_slot.item_id {
                let space = 99u8.saturating_sub(stack.quantity);
                let add = remaining.min(space);
                stack.quantity += add;
                remaining -= add;
            }
        }
    }

    // Place remainder in first empty chest slot
    if remaining > 0 {
        for chest_slot in chest.slots.iter_mut() {
            if chest_slot.is_none() {
                *chest_slot = Some(QualityStack {
                    item_id: inv_slot.item_id.clone(),
                    quantity: remaining,
                    quality: ItemQuality::Normal,
                });
                remaining = 0;
                break;
            }
        }
    }

    // If chest was full, put items back into inventory
    if remaining > 0 {
        inventory.slots[idx] = Some(InventorySlot {
            item_id: inv_slot.item_id,
            quantity: remaining,
        });
    }
}
