use bevy::prelude::*;
use crate::shared::*;
use crate::economy::blacksmith::ToolUpgradeRequestEvent;
use super::hud::ItemAtlasData;

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
pub struct ShopItemIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopHintText;

/// Display data for a single tool upgrade option in the Blacksmith upgrade mode.
pub struct ToolUpgradeDisplayEntry {
    pub tool: ToolKind,
    pub current_tier: ToolTier,
    pub target_tier: ToolTier,
    pub gold_cost: u32,
    pub bar_name: String,
    pub bar_qty: u8,
    pub can_afford: bool,
    pub has_bars: bool,
    pub is_upgrading: bool,
}

/// Tracks which shop is open, selection, and buy/sell mode
#[derive(Resource)]
pub struct ShopUiState {
    pub shop_id: ShopId,
    pub cursor: usize,
    pub is_buy_mode: bool,
    /// True when the Blacksmith upgrade tab is active.
    pub upgrade_mode: bool,
    /// Cached list of available items (filtered by season for buy mode)
    pub buy_items: Vec<ShopListing>,
    /// Player items available to sell
    pub sell_items: Vec<(ItemId, String, u32, u8)>, // id, name, sell_price, quantity
    /// Upgrade entries (only populated for ShopId::Blacksmith)
    pub upgrade_entries: Vec<ToolUpgradeDisplayEntry>,
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
    active_shop: Res<crate::economy::shop::ActiveShop>,
    upgrade_queue: Res<crate::economy::blacksmith::ToolUpgradeQueue>,
    atlas_data: Res<ItemAtlasData>,
) {
    // Use the shop_id set by the economy system when opening the shop.
    let shop_id = active_shop.shop_id.unwrap_or(ShopId::GeneralStore);

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

    // Build upgrade entries when at the Blacksmith.
    let upgrade_entries = if shop_id == ShopId::Blacksmith {
        build_upgrade_entries(&player, &inventory, &upgrade_queue)
    } else {
        Vec::new()
    };

    commands.insert_resource(ShopUiState {
        shop_id,
        cursor: 0,
        is_buy_mode: true,
        upgrade_mode: false,
        buy_items: buy_items.clone(),
        sell_items: sell_items.clone(),
        upgrade_entries,
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
                                    // Item icon
                                    if atlas_data.loaded {
                                        row.spawn((
                                            ShopItemIcon { index: i },
                                            ImageNode {
                                                image: atlas_data.image.clone(),
                                                texture_atlas: Some(TextureAtlas {
                                                    layout: atlas_data.layout.clone(),
                                                    index: 0,
                                                }),
                                                ..default()
                                            },
                                            Node {
                                                width: Val::Px(22.0),
                                                height: Val::Px(22.0),
                                                margin: UiRect::right(Val::Px(4.0)),
                                                ..default()
                                            },
                                            Visibility::Hidden,
                                        ));
                                    }
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
                        Text::new("Up/Down: Select | Enter: Confirm | Tab: Toggle Mode | Esc: Close"),
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

fn build_upgrade_entries(
    player: &PlayerState,
    inventory: &Inventory,
    queue: &crate::economy::blacksmith::ToolUpgradeQueue,
) -> Vec<ToolUpgradeDisplayEntry> {
    let upgradeable = [
        ToolKind::Hoe,
        ToolKind::WateringCan,
        ToolKind::Axe,
        ToolKind::Pickaxe,
        ToolKind::FishingRod,
    ];
    upgradeable
        .iter()
        .filter_map(|&tool| {
            let current_tier = *player.tools.get(&tool)?;
            let target_tier = current_tier.next()?;
            let gold_cost = target_tier.upgrade_cost();
            let bar_qty = current_tier.upgrade_bars_needed();
            let bar_id = current_tier.upgrade_bar_item()?;
            let can_afford = player.gold >= gold_cost;
            let has_bars = inventory.has(bar_id, bar_qty);
            let is_upgrading = queue.is_upgrading(tool);
            Some(ToolUpgradeDisplayEntry {
                tool,
                current_tier,
                target_tier,
                gold_cost,
                bar_name: bar_id.replace('_', " "),
                bar_qty,
                can_afford,
                has_bars,
                is_upgrading,
            })
        })
        .collect()
}

pub fn despawn_shop_screen(
    mut commands: Commands,
    query: Query<Entity, With<ShopScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
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
    mut icon_query: Query<(&ShopItemIcon, &mut ImageNode, &mut Visibility)>,
) {
    let Some(ui_state) = ui_state else { return };

    // Gold
    for mut text in &mut gold_query {
        **text = format!("{} G", player.gold);
    }

    // Mode
    for mut text in &mut mode_query {
        if ui_state.upgrade_mode {
            **text = "[Tab] Mode: UPGRADE".to_string();
        } else if ui_state.is_buy_mode {
            **text = "[Tab] Mode: BUY".to_string();
        } else {
            **text = "[Tab] Mode: SELL".to_string();
        }
    }

    // Items list
    if ui_state.upgrade_mode {
        for (name_comp, mut text) in &mut name_query {
            let idx = name_comp.index;
            if idx < ui_state.upgrade_entries.len() {
                let entry = &ui_state.upgrade_entries[idx];
                let status = if entry.is_upgrading { " [IN PROGRESS]" } else { "" };
                **text = format!("{:?}: {:?} → {:?}{}", entry.tool, entry.current_tier, entry.target_tier, status);
            } else {
                **text = String::new();
            }
        }
        for (price_comp, mut text, mut color) in &mut price_query {
            let idx = price_comp.index;
            if idx < ui_state.upgrade_entries.len() {
                let entry = &ui_state.upgrade_entries[idx];
                **text = format!("{}G + {}x {}", entry.gold_cost, entry.bar_qty, entry.bar_name);
                if entry.is_upgrading {
                    *color = TextColor(Color::srgb(0.6, 0.6, 0.6));
                } else if entry.can_afford && entry.has_bars {
                    *color = TextColor(Color::srgb(0.5, 0.9, 0.5));
                } else {
                    *color = TextColor(Color::srgb(0.8, 0.3, 0.3));
                }
            } else {
                **text = String::new();
            }
        }
    } else if ui_state.is_buy_mode {
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

    // Update item icons
    for (icon, mut img, mut vis) in &mut icon_query {
        let idx = icon.index;
        let item_id: Option<&str> = if ui_state.upgrade_mode {
            None // No item icon for tool upgrades
        } else if ui_state.is_buy_mode {
            ui_state.buy_items.get(idx).map(|l| l.item_id.as_str())
        } else {
            ui_state.sell_items.get(idx).map(|(id, _, _, _)| id.as_str())
        };
        if let Some(id) = item_id {
            if let Some(def) = item_registry.get(id) {
                if let Some(ref mut atlas) = img.texture_atlas {
                    atlas.index = def.sprite_index as usize;
                }
                *vis = Visibility::Inherited;
                continue;
            }
        }
        *vis = Visibility::Hidden;
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
    action: Res<MenuAction>,
    player_input: Res<PlayerInput>,
    mut ui_state: Option<ResMut<ShopUiState>>,
    mut player: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    upgrade_queue: Res<crate::economy::blacksmith::ToolUpgradeQueue>,
    mut tx_events: EventWriter<ShopTransactionEvent>,
    mut upgrade_events: EventWriter<ToolUpgradeRequestEvent>,
) {
    let Some(ref mut ui_state) = ui_state else { return };

    let max_items = if ui_state.upgrade_mode {
        ui_state.upgrade_entries.len()
    } else if ui_state.is_buy_mode {
        ui_state.buy_items.len()
    } else {
        ui_state.sell_items.len()
    };

    // Navigation
    if action.move_down {
        if max_items > 0 && ui_state.cursor < max_items - 1 {
            ui_state.cursor += 1;
        }
    }
    if action.move_up {
        if ui_state.cursor > 0 {
            ui_state.cursor -= 1;
        }
    }

    // Cycle modes: Tab
    // Blacksmith: buy → sell → upgrade → buy
    // Other shops: buy ↔ sell
    if player_input.open_inventory {
        if ui_state.shop_id == ShopId::Blacksmith {
            if ui_state.is_buy_mode && !ui_state.upgrade_mode {
                // buy → sell
                ui_state.is_buy_mode = false;
                ui_state.upgrade_mode = false;
                ui_state.sell_items = build_sell_list(&inventory, &item_registry);
            } else if !ui_state.is_buy_mode && !ui_state.upgrade_mode {
                // sell → upgrade
                ui_state.upgrade_mode = true;
                ui_state.upgrade_entries = build_upgrade_entries(&player, &inventory, &upgrade_queue);
            } else {
                // upgrade → buy
                ui_state.upgrade_mode = false;
                ui_state.is_buy_mode = true;
            }
        } else {
            ui_state.is_buy_mode = !ui_state.is_buy_mode;
            if !ui_state.is_buy_mode {
                ui_state.sell_items = build_sell_list(&inventory, &item_registry);
            }
        }
        ui_state.cursor = 0;
    }

    // Execute transaction
    if action.activate {
        if ui_state.upgrade_mode {
            // Upgrade
            if ui_state.cursor < ui_state.upgrade_entries.len() {
                let tool = ui_state.upgrade_entries[ui_state.cursor].tool;
                upgrade_events.send(ToolUpgradeRequestEvent { tool });
                // Refresh entries after submitting request
                ui_state.upgrade_entries = build_upgrade_entries(&player, &inventory, &upgrade_queue);
            }
        } else if ui_state.is_buy_mode {
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
