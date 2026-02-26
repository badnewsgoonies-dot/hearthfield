use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct ShopScreenRoot;

#[derive(Component)]
pub struct ShopTitle;

#[derive(Component)]
pub struct ShopGoldDisplay;

#[derive(Component)]
pub struct ShopModeText;

#[derive(Component)]
pub struct ShopListContainer;

#[derive(Component)]
pub struct ShopListItem {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopItemName {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopItemPrice {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopHintText;

/// Tracks which shop is open, selection, and buy/sell mode
#[derive(Resource)]
pub struct ShopUiState {
    pub shop_id: ShopId,
    pub cursor: usize,
    pub is_buy_mode: bool,
    /// Cached list of available items (filtered by season for buy mode)
    pub buy_items: Vec<ShopListing>,
    /// Player items available to sell
    pub sell_items: Vec<(ItemId, String, u32, u8)>, // id, name, sell_price, quantity
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_shop_screen(
    mut commands: Commands,
    shop_data: Res<ShopData>,
    calendar: Res<Calendar>,
    player: Res<PlayerState>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
) {
    // Determine which shop to open — default to GeneralStore
    // Other domains set the shop_id via a resource before transitioning state.
    // We'll check if ShopUiState already exists (set by another domain).
    let shop_id = ShopId::GeneralStore;

    let buy_items: Vec<ShopListing> = shop_data
        .listings
        .get(&shop_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|listing| {
            listing
                .season_available
                .map_or(true, |s| s == calendar.season)
        })
        .collect();

    let sell_items = build_sell_list(&inventory, &item_registry);

    commands.insert_resource(ShopUiState {
        shop_id,
        cursor: 0,
        is_buy_mode: true,
        buy_items: buy_items.clone(),
        sell_items: sell_items.clone(),
    });

    commands
        .spawn((
            ShopScreenRoot,
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
            // Main shop panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(500.0),
                        height: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(3.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.1, 0.08, 0.95)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.25)),
                ))
                .with_children(|panel| {
                    // Title row
                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|title_row| {
                            title_row.spawn((
                                ShopTitle,
                                Text::new("GENERAL STORE"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.9, 0.6)),
                            ));

                            title_row.spawn((
                                ShopGoldDisplay,
                                Text::new(format!("{} G", player.gold)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.84, 0.0)),
                            ));
                        });

                    // Mode toggle
                    panel.spawn((
                        ShopModeText,
                        Text::new("[Tab] Mode: BUY"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));

                    // Item list container
                    panel
                        .spawn((
                            ShopListContainer,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                row_gap: Val::Px(2.0),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                        ))
                        .with_children(|list| {
                            // We'll create up to 10 visible slots; actual items come from update
                            for i in 0..10 {
                                list.spawn((
                                    ShopListItem { index: i },
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(28.0),
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
                                        ShopItemName { index: i },
                                        Text::new(""),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    row.spawn((
                                        ShopItemPrice { index: i },
                                        Text::new(""),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(1.0, 0.84, 0.0)),
                                    ));
                                });
                            }
                        });

                    // Hint text
                    panel.spawn((
                        ShopHintText,
                        Text::new("Up/Down: Select | Enter: Buy | Tab: Toggle Buy/Sell | Esc: Close"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

fn build_sell_list(
    inventory: &Inventory,
    item_registry: &ItemRegistry,
) -> Vec<(ItemId, String, u32, u8)> {
    let mut result = Vec::new();
    for slot in &inventory.slots {
        if let Some(ref s) = slot {
            // Check if we already have this item in the sell list
            if result.iter().any(|(id, _, _, _): &(ItemId, String, u32, u8)| id == &s.item_id) {
                continue;
            }
            let name = item_registry
                .get(&s.item_id)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| s.item_id.clone());
            let price = item_registry.get(&s.item_id).map(|d| d.sell_price).unwrap_or(1);
            let total_qty = inventory.count(&s.item_id) as u8;
            result.push((s.item_id.clone(), name, price, total_qty));
        }
    }
    result
}

pub fn despawn_shop_screen(
    mut commands: Commands,
    query: Query<Entity, With<ShopScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ShopUiState>();
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

pub fn update_shop_display(
    ui_state: Option<Res<ShopUiState>>,
    item_registry: Res<ItemRegistry>,
    player: Res<PlayerState>,
    mut gold_query: Query<&mut Text, With<ShopGoldDisplay>>,
    mut mode_query: Query<&mut Text, (With<ShopModeText>, Without<ShopGoldDisplay>, Without<ShopItemName>, Without<ShopItemPrice>)>,
    mut name_query: Query<(&ShopItemName, &mut Text), (Without<ShopGoldDisplay>, Without<ShopModeText>, Without<ShopItemPrice>)>,
    mut price_query: Query<(&ShopItemPrice, &mut Text, &mut TextColor), (Without<ShopGoldDisplay>, Without<ShopModeText>, Without<ShopItemName>)>,
    mut row_query: Query<(&ShopListItem, &mut BackgroundColor)>,
) {
    let Some(ui_state) = ui_state else { return };

    // Gold
    for mut text in &mut gold_query {
        **text = format!("{} G", player.gold);
    }

    // Mode
    for mut text in &mut mode_query {
        if ui_state.is_buy_mode {
            **text = "[Tab] Mode: BUY".to_string();
        } else {
            **text = "[Tab] Mode: SELL".to_string();
        }
    }

    // Items list
    if ui_state.is_buy_mode {
        for (name_comp, mut text) in &mut name_query {
            let idx = name_comp.index;
            if idx < ui_state.buy_items.len() {
                let listing = &ui_state.buy_items[idx];
                let name = item_registry
                    .get(&listing.item_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| listing.item_id.clone());
                **text = name;
            } else {
                **text = String::new();
            }
        }
        for (price_comp, mut text, mut color) in &mut price_query {
            let idx = price_comp.index;
            if idx < ui_state.buy_items.len() {
                let listing = &ui_state.buy_items[idx];
                **text = format!("{} G", listing.price);
                // Red if can't afford, gold if can
                if listing.price > player.gold {
                    *color = TextColor(Color::srgb(0.8, 0.3, 0.3));
                } else {
                    *color = TextColor(Color::srgb(1.0, 0.84, 0.0));
                }
            } else {
                **text = String::new();
            }
        }
    } else {
        for (name_comp, mut text) in &mut name_query {
            let idx = name_comp.index;
            if idx < ui_state.sell_items.len() {
                let (_, ref name, _, qty) = ui_state.sell_items[idx];
                **text = format!("{} (x{})", name, qty);
            } else {
                **text = String::new();
            }
        }
        for (price_comp, mut text, mut color) in &mut price_query {
            let idx = price_comp.index;
            if idx < ui_state.sell_items.len() {
                let (_, _, price, _) = ui_state.sell_items[idx];
                **text = format!("{} G", price);
                *color = TextColor(Color::srgb(0.5, 0.9, 0.5));
            } else {
                **text = String::new();
            }
        }
    }

    // Highlight cursor row
    for (item, mut bg) in &mut row_query {
        if item.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.9));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6));
        }
    }
}

pub fn shop_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: Option<ResMut<ShopUiState>>,
    mut player: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut tx_events: EventWriter<ShopTransactionEvent>,
) {
    let Some(ref mut ui_state) = ui_state else { return };

    let max_items = if ui_state.is_buy_mode {
        ui_state.buy_items.len()
    } else {
        ui_state.sell_items.len()
    };

    // Navigation
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if max_items > 0 && ui_state.cursor < max_items - 1 {
            ui_state.cursor += 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if ui_state.cursor > 0 {
            ui_state.cursor -= 1;
        }
    }

    // Toggle buy/sell
    if keyboard.just_pressed(KeyCode::Tab) {
        ui_state.is_buy_mode = !ui_state.is_buy_mode;
        ui_state.cursor = 0;
        // Refresh sell list
        if !ui_state.is_buy_mode {
            ui_state.sell_items = build_sell_list(&inventory, &item_registry);
        }
    }

    // Execute transaction
    if keyboard.just_pressed(KeyCode::Enter) {
        if ui_state.is_buy_mode {
            // Buy
            if ui_state.cursor < ui_state.buy_items.len() {
                let listing = ui_state.buy_items[ui_state.cursor].clone();
                if player.gold >= listing.price {
                    let max_stack = item_registry
                        .get(&listing.item_id)
                        .map(|d| d.stack_size)
                        .unwrap_or(99);
                    let overflow = inventory.try_add(&listing.item_id, 1, max_stack);
                    if overflow == 0 {
                        player.gold -= listing.price;
                        tx_events.send(ShopTransactionEvent {
                            shop_id: ui_state.shop_id,
                            item_id: listing.item_id,
                            quantity: 1,
                            total_cost: listing.price,
                            is_purchase: true,
                        });
                    }
                }
            }
        } else {
            // Sell
            if ui_state.cursor < ui_state.sell_items.len() {
                let (ref item_id, _, price, _) = ui_state.sell_items[ui_state.cursor];
                let removed = inventory.try_remove(item_id, 1);
                if removed > 0 {
                    player.gold += price;
                    tx_events.send(ShopTransactionEvent {
                        shop_id: ui_state.shop_id,
                        item_id: item_id.clone(),
                        quantity: 1,
                        total_cost: price,
                        is_purchase: false,
                    });
                    // Refresh sell list
                    ui_state.sell_items = build_sell_list(&inventory, &item_registry);
                    if ui_state.cursor >= ui_state.sell_items.len() && ui_state.cursor > 0 {
                        ui_state.cursor -= 1;
                    }
                }
            }
        }
    }
}
