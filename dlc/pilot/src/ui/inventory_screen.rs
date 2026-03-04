//! Grid inventory screen — displays item slots with selection, use/equip, and detail panel.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct InventoryScreenRoot;

/// Tracks which inventory slot is currently selected (by index).
#[derive(Resource, Default)]
pub struct SelectedSlot(pub Option<usize>);

/// Marker for a clickable inventory slot button, storing the slot index.
#[derive(Component)]
pub struct InventorySlotButton(pub usize);

/// Tag for which detail text field a text entity represents.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DetailField {
    Name,
    Description,
    Quantity,
    ButtonLabel,
}

/// Marker for the Use/Equip action button.
#[derive(Component)]
pub struct UseEquipButton;

/// Event fired when the player uses an item from inventory.
#[derive(Event, Clone, Debug)]
pub struct UseItemEvent {
    pub slot: usize,
}

pub fn spawn_inventory_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    inventory: Res<Inventory>,
) {
    commands.insert_resource(SelectedSlot(None));

    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 24.0,
        ..default()
    };
    let slot_style = TextFont {
        font: font.0.clone(),
        font_size: 13.0,
        ..default()
    };
    let detail_name_style = TextFont {
        font: font.0.clone(),
        font_size: 18.0,
        ..default()
    };
    let detail_desc_style = TextFont {
        font: font.0.clone(),
        font_size: 13.0,
        ..default()
    };
    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 13.0,
        ..default()
    };

    commands
        .spawn((
            InventoryScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(24.0)),
                column_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
        ))
        .with_children(|root| {
            // Left column: title + grid
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    Text::new("INVENTORY"),
                    title_style,
                    TextColor(Color::srgb(0.9, 0.8, 0.3)),
                ));

                // Grid container
                left.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|grid| {
                    for i in 0..inventory.max_slots {
                        let label = inventory
                            .slots
                            .get(i)
                            .map(|s| format!("{} x{}", s.item_id, s.quantity))
                            .unwrap_or_else(|| "\u{2014}".to_string());

                        let bg = if i < inventory.slots.len() {
                            Color::srgb(0.2, 0.2, 0.3)
                        } else {
                            Color::srgb(0.12, 0.12, 0.15)
                        };

                        grid.spawn((
                            Button,
                            InventorySlotButton(i),
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ))
                        .with_children(|slot| {
                            slot.spawn((
                                Text::new(label),
                                slot_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
            });

            // Right column: detail panel
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(220.0),
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn((
                    DetailField::Name,
                    Text::new("Select an item"),
                    detail_name_style,
                    TextColor(Color::srgb(0.9, 0.8, 0.3)),
                ));
                panel.spawn((
                    DetailField::Description,
                    Text::new(""),
                    detail_desc_style.clone(),
                    TextColor(Color::srgb(0.75, 0.75, 0.75)),
                ));
                panel.spawn((
                    DetailField::Quantity,
                    Text::new(""),
                    detail_desc_style,
                    TextColor(Color::srgb(0.6, 0.8, 0.6)),
                ));
                // Use/Equip button (hidden until item selected)
                panel
                    .spawn((
                        Button,
                        UseEquipButton,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            margin: UiRect::top(Val::Px(8.0)),
                            display: Display::None,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            DetailField::ButtonLabel,
                            Text::new("Use"),
                            btn_style,
                            TextColor(Color::WHITE),
                        ));
                    });
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
    commands.remove_resource::<SelectedSlot>();
}

/// Click handler: clicking a slot selects it.
pub fn handle_slot_click(
    mut interaction_q: Query<
        (&Interaction, &InventorySlotButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut selected: ResMut<SelectedSlot>,
    inventory: Res<Inventory>,
) {
    for (interaction, slot_btn, mut bg) in &mut interaction_q {
        match *interaction {
            Interaction::Pressed => {
                selected.0 = Some(slot_btn.0);
                *bg = BackgroundColor(Color::srgb(0.15, 0.15, 0.25));
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            }
            Interaction::None => {
                if selected.0 == Some(slot_btn.0) || slot_btn.0 < inventory.slots.len() {
                    *bg = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                } else {
                    *bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
                }
            }
        }
    }
}

/// Updates the visual highlight (border) on slots and the detail panel when selection changes.
pub fn update_selection_visuals(
    selected: Res<SelectedSlot>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut slot_q: Query<(&InventorySlotButton, &mut BorderColor)>,
    mut detail_q: Query<(&DetailField, &mut Text)>,
    mut btn_node_q: Query<&mut Node, With<UseEquipButton>>,
) {
    if !selected.is_changed() {
        return;
    }

    // Update slot border highlights
    for (slot_btn, mut border) in &mut slot_q {
        if selected.0 == Some(slot_btn.0) {
            *border = BorderColor(Color::srgb(0.9, 0.8, 0.3));
        } else {
            *border = BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0));
        }
    }

    // Update detail panel text
    let (name, desc, qty, btn_label, show_btn) = match selected.0 {
        Some(idx) => match inventory.slots.get(idx) {
            Some(slot) => {
                let def = item_registry.get(&slot.item_id);
                let n = def.map_or_else(|| slot.item_id.clone(), |d| d.name.clone());
                let d = def.map_or_else(String::new, |d| d.description.clone());
                let is_equip = def.is_some_and(|d| {
                    matches!(
                        d.category,
                        ItemCategory::Tool | ItemCategory::Part | ItemCategory::Cosmetic
                    )
                });
                let label = if is_equip { "Equip" } else { "Use" };
                (n, d, format!("Qty: {}", slot.quantity), label, true)
            }
            None => ("Empty slot".to_string(), String::new(), String::new(), "Use", false),
        },
        None => ("Select an item".to_string(), String::new(), String::new(), "Use", false),
    };

    for (field, mut text) in &mut detail_q {
        match field {
            DetailField::Name => **text = name.clone(),
            DetailField::Description => **text = desc.clone(),
            DetailField::Quantity => **text = qty.clone(),
            DetailField::ButtonLabel => **text = btn_label.to_string(),
        }
    }

    for mut node in &mut btn_node_q {
        node.display = if show_btn { Display::Flex } else { Display::None };
    }
}

/// Handle the Use/Equip button press.
#[allow(clippy::type_complexity)]
pub fn handle_use_equip_button(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<UseEquipButton>),
    >,
    selected: Res<SelectedSlot>,
    inventory: Res<Inventory>,
    mut use_events: EventWriter<UseItemEvent>,
) {
    for (interaction, mut bg) in &mut interaction_q {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.35, 0.2));
                if let Some(idx) = selected.0 {
                    if idx < inventory.slots.len() {
                        use_events.send(UseItemEvent { slot: idx });
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.3, 0.6, 0.4));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.2, 0.5, 0.3));
            }
        }
    }
}

/// Keyboard navigation: arrows to move selection, Enter/Space to use, Esc to close.
pub fn handle_inventory_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected: ResMut<SelectedSlot>,
    inventory: Res<Inventory>,
    mut use_events: EventWriter<UseItemEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
        return;
    }

    let max = inventory.max_slots;
    if max == 0 {
        return;
    }

    // Grid columns for arrow key navigation (based on 80px slot width)
    let cols: usize = 5;
    let mut current = selected.0.unwrap_or(0);
    let mut moved = false;

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        current = (current + 1).min(max - 1);
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        current = current.saturating_sub(1);
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        current = (current + cols).min(max - 1);
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        current = current.saturating_sub(cols);
        moved = true;
    }

    if moved {
        selected.0 = Some(current);
    }

    if (keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space))
        && selected.0.is_some_and(|idx| idx < inventory.slots.len())
    {
        use_events.send(UseItemEvent {
            slot: selected.0.unwrap(),
        });
    }
}
