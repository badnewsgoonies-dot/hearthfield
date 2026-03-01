use bevy::prelude::*;
use crate::shared::*;
use super::UiFontHandle;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS — used to query and update HUD elements
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct HudRoot;

/// Marker for the map name text node at bottom-left.
#[derive(Component)]
pub struct HudMapName;

/// Resource: tracks fade-in/out for the map name indicator.
#[derive(Resource)]
pub struct MapNameFadeTimer {
    /// How long the name stays fully visible before fading out.
    pub display_timer: Timer,
    /// How long the fade-out takes.
    pub fade_timer: Timer,
    /// Current alpha of the map name text (0.0 = invisible, 1.0 = fully visible).
    pub alpha: f32,
    /// Last map that was displayed (to detect changes).
    pub last_map: Option<MapId>,
}

#[derive(Component)]
pub struct HudTimeText;

#[derive(Component)]
pub struct HudWeatherText;

#[derive(Component)]
pub struct HudGoldText;

#[derive(Component)]
pub struct HudStaminaBar;

#[derive(Component)]
pub struct HudStaminaFill;

#[derive(Component)]
pub struct HudToolText;

#[derive(Component)]
pub struct HotbarRoot;

#[derive(Component)]
pub struct HotbarSlot {
    pub index: usize,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct HotbarSlotBackground {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarItemText {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarQuantityText {
    pub index: usize,
}

/// Marker for the item icon ImageNode inside each hotbar slot.
#[derive(Component, Debug)]
pub struct HotbarItemIcon {
    pub index: usize,
}

/// Lazily loaded atlas for item icons in the hotbar.
#[derive(Resource, Default)]
pub struct ItemAtlasData {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

/// Marker for the tutorial objective text below the top bar.
#[derive(Component)]
pub struct HudObjective;

/// Marker for the "[F] Interact" prompt above the hotbar.
#[derive(Component, Debug)]
pub struct HudInteractionPrompt;

// ═══════════════════════════════════════════════════════════════════════
// SPAWN HUD
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_hud(mut commands: Commands, font_handle: Res<UiFontHandle>) {
    let font = font_handle.0.clone();

    // Root container — full screen overlay, no interaction blocking
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            // Use PickingBehavior::IGNORE so HUD doesn't block game clicks
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            // ─── TOP BAR ───
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(44.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    PickingBehavior::IGNORE,
                ))
                .with_children(|top_bar| {
                    // Left group: time + weather
                    top_bar
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(16.0),
                                ..default()
                            },
                            PickingBehavior::IGNORE,
                        ))
                        .with_children(|left| {
                            left.spawn((
                                HudTimeText,
                                Text::new("Spring 1, Year 1 - 6:00 AM"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                PickingBehavior::IGNORE,
                            ));

                            left.spawn((
                                HudWeatherText,
                                Text::new("Sunny"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.9, 0.5)),
                                PickingBehavior::IGNORE,
                            ));
                        });

                    // Center: tool name
                    top_bar.spawn((
                        HudToolText,
                        Text::new("Hoe"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.85, 1.0)),
                        PickingBehavior::IGNORE,
                    ));

                    // Right group: gold + stamina
                    top_bar
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(16.0),
                                ..default()
                            },
                            PickingBehavior::IGNORE,
                        ))
                        .with_children(|right| {
                            // Gold
                            right.spawn((
                                HudGoldText,
                                Text::new("500 G"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.84, 0.0)),
                                PickingBehavior::IGNORE,
                            ));

                            // Stamina bar container
                            right
                                .spawn((
                                    HudStaminaBar,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(14.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                                    BorderColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                                    PickingBehavior::IGNORE,
                                ))
                                .with_children(|bar| {
                                    bar.spawn((
                                        HudStaminaFill,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.2, 0.85, 0.3)),
                                        PickingBehavior::IGNORE,
                                    ));
                                });
                        });
                });

            // ─── INTERACTION PROMPT — centered above hotbar ───
            parent.spawn((
                HudInteractionPrompt,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.95, 0.7, 0.0)),
                Node {
                    align_self: AlignSelf::Center,
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
                PickingBehavior::IGNORE,
            ));

            // ─── BOTTOM: HOTBAR ───
            spawn_hotbar(parent, &font);
        });

    // ─── MAP NAME — absolute position, bottom-left ───
    commands.spawn((
        HudMapName,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(70.0),
            left: Val::Px(12.0),
            padding: UiRect {
                left: Val::Px(8.0),
                right: Val::Px(8.0),
                top: Val::Px(4.0),
                bottom: Val::Px(4.0),
            },
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        PickingBehavior::IGNORE,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
            PickingBehavior::IGNORE,
        ));
    });

    // ─── OBJECTIVE — absolute position, top-left below top bar ───
    commands.spawn((
        HudObjective,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(52.0),
            left: Val::Px(12.0),
            padding: UiRect {
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        PickingBehavior::IGNORE,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.95, 0.7)),
            PickingBehavior::IGNORE,
        ));
    });

    // Initialise the fade timer resource every time the HUD spawns.
    commands.insert_resource(MapNameFadeTimer {
        display_timer: Timer::from_seconds(2.0, TimerMode::Once),
        fade_timer: Timer::from_seconds(0.8, TimerMode::Once),
        alpha: 0.0,
        last_map: None,
    });
}

fn spawn_hotbar(parent: &mut ChildBuilder, font: &Handle<Font>) {
    parent
        .spawn((
            HotbarRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(3.0),
                padding: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|hotbar| {
            for i in 0..HOTBAR_SLOTS {
                hotbar
                    .spawn((
                        HotbarSlot { index: i },
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.12, 0.1, 0.85)),
                        BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.8)),
                        PickingBehavior::IGNORE,
                    ))
                    .with_children(|slot| {
                        // Slot key number (1-9, 0 for slot 10+)
                        let key_label = if i < 9 {
                            format!("{}", i + 1)
                        } else {
                            String::new()
                        };
                        slot.spawn((
                            Text::new(key_label),
                            TextFont {
                                font: font.clone(),
                                font_size: 7.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.5, 0.5, 0.45, 0.7)),
                            Node {
                                align_self: AlignSelf::FlexStart,
                                margin: UiRect::left(Val::Px(2.0)),
                                ..default()
                            },
                            PickingBehavior::IGNORE,
                        ));
                        // Item icon (shown when atlas is loaded)
                        slot.spawn((
                            HotbarItemIcon { index: i },
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                ..default()
                            },
                            // ImageNode will be set dynamically in update_hotbar
                            Visibility::Hidden,
                            PickingBehavior::IGNORE,
                        ));
                        // Item name fallback (shown when atlas not loaded)
                        slot.spawn((
                            HotbarItemText { index: i },
                            Text::new(""),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            PickingBehavior::IGNORE,
                        ));
                        // Quantity
                        slot.spawn((
                            HotbarQuantityText { index: i },
                            Text::new(""),
                            TextFont {
                                font: font.clone(),
                                font_size: 9.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            PickingBehavior::IGNORE,
                        ));
                    });
            }
        });
}

// ═══════════════════════════════════════════════════════════════════════
// DESPAWN HUD
// ═══════════════════════════════════════════════════════════════════════

pub fn despawn_hud(
    mut commands: Commands,
    hud_query: Query<Entity, With<HudRoot>>,
    map_name_query: Query<Entity, With<HudMapName>>,
    objective_query: Query<Entity, With<HudObjective>>,
) {
    for entity in &hud_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &map_name_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &objective_query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<MapNameFadeTimer>();
}

// ─── MAP NAME DISPLAY HELPER ──────────────────────────────────────────

fn map_display_name(map: MapId) -> &'static str {
    match map {
        MapId::Farm => "The Farm",
        MapId::Town => "Pelican Town",
        MapId::Beach => "The Beach",
        MapId::Forest => "Cindersap Forest",
        MapId::MineEntrance => "The Mines",
        MapId::Mine => "Mine Floor",
        MapId::PlayerHouse => "Home",
        MapId::GeneralStore => "Pierre's Shop",
        MapId::AnimalShop => "Marnie's Ranch",
        MapId::Blacksmith => "Clint's Blacksmith",
    }
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS — real-time HUD data binding
// ═══════════════════════════════════════════════════════════════════════

pub fn update_time_display(
    calendar: Res<Calendar>,
    mut query: Query<&mut Text, With<HudTimeText>>,
) {
    if !calendar.is_changed() {
        return;
    }
    for mut text in &mut query {
        let season_name = match calendar.season {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        };
        let (display_hour, am_pm) = if calendar.hour == 0 || calendar.hour == 24 {
            (12, "AM")
        } else if calendar.hour < 12 {
            (calendar.hour, "AM")
        } else if calendar.hour == 12 {
            (12, "PM")
        } else if calendar.hour < 24 {
            (calendar.hour - 12, "PM")
        } else {
            // 25 = 1:00 AM (past midnight)
            (calendar.hour - 24, "AM")
        };
        **text = format!(
            "{} {}, Year {} - {}:{:02} {}",
            season_name, calendar.day, calendar.year, display_hour, calendar.minute, am_pm
        );
    }
}

pub fn update_weather_display(
    calendar: Res<Calendar>,
    mut query: Query<(&mut Text, &mut TextColor), With<HudWeatherText>>,
) {
    if !calendar.is_changed() {
        return;
    }
    for (mut text, mut color) in &mut query {
        let (label, col) = match calendar.weather {
            Weather::Sunny => ("Sunny", Color::srgb(1.0, 0.9, 0.3)),
            Weather::Rainy => ("Rainy", Color::srgb(0.5, 0.7, 1.0)),
            Weather::Stormy => ("Stormy", Color::srgb(0.6, 0.5, 0.8)),
            Weather::Snowy => ("Snowy", Color::srgb(0.85, 0.9, 1.0)),
        };
        **text = label.to_string();
        *color = TextColor(col);
    }
}

pub fn update_gold_display(
    player: Res<PlayerState>,
    mut query: Query<&mut Text, With<HudGoldText>>,
) {
    if !player.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("{} G", player.gold);
    }
}

pub fn update_stamina_bar(
    player: Res<PlayerState>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<HudStaminaFill>>,
) {
    if !player.is_changed() {
        return;
    }
    for (mut node, mut bg) in &mut query {
        let ratio = (player.stamina / player.max_stamina).clamp(0.0, 1.0);
        node.width = Val::Percent(ratio * 100.0);
        // Color gradient: green > yellow > red as stamina decreases
        let color = if ratio > 0.5 {
            // Green to yellow
            let t = (ratio - 0.5) * 2.0;
            Color::srgb(0.2 + 0.8 * (1.0 - t), 0.85, 0.3 * t)
        } else {
            // Yellow to red
            let t = ratio * 2.0;
            Color::srgb(0.9, 0.85 * t, 0.1 * t)
        };
        *bg = BackgroundColor(color);
    }
}

pub fn update_tool_display(
    player: Res<PlayerState>,
    mut query: Query<&mut Text, With<HudToolText>>,
) {
    if !player.is_changed() {
        return;
    }
    for mut text in &mut query {
        let tool_name = match player.equipped_tool {
            ToolKind::Hoe => "Hoe",
            ToolKind::WateringCan => "Watering Can",
            ToolKind::Axe => "Axe",
            ToolKind::Pickaxe => "Pickaxe",
            ToolKind::FishingRod => "Fishing Rod",
            ToolKind::Scythe => "Scythe",
        };
        let tier = player.tools.get(&player.equipped_tool).copied().unwrap_or(ToolTier::Basic);
        let tier_prefix = match tier {
            ToolTier::Basic => "",
            ToolTier::Copper => "Copper ",
            ToolTier::Iron => "Iron ",
            ToolTier::Gold => "Gold ",
            ToolTier::Iridium => "Iridium ",
        };
        **text = format!("{}{}", tier_prefix, tool_name);
    }
}

pub fn update_hotbar(
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut item_text_query: Query<(&HotbarItemText, &mut Text, &mut TextColor), Without<HotbarQuantityText>>,
    mut qty_text_query: Query<(&HotbarQuantityText, &mut Text), Without<HotbarItemText>>,
    mut icon_query: Query<(&HotbarItemIcon, &mut Visibility), Without<HotbarSlot>>,
) {
    // Update slot backgrounds (highlight selected)
    for (slot, mut bg, mut border) in &mut slot_query {
        if slot.index == inventory.selected_slot {
            *bg = BackgroundColor(Color::srgba(0.3, 0.25, 0.15, 0.95));
            *border = BorderColor(Color::srgb(1.0, 0.84, 0.0));
        } else {
            *bg = BackgroundColor(Color::srgba(0.15, 0.12, 0.1, 0.85));
            *border = BorderColor(Color::srgba(0.4, 0.35, 0.3, 0.8));
        }
    }

    // Update icons — show if slot has item and icon has been hydrated with ImageNode
    for (icon, mut vis) in &mut icon_query {
        let idx = icon.index;
        let has_item = idx < inventory.slots.len() && inventory.slots[idx].is_some();
        *vis = if has_item { Visibility::Inherited } else { Visibility::Hidden };
    }

    // Update item text
    for (item_text, mut text, mut tc) in &mut item_text_query {
        let idx = item_text.index;
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
                tc.0 = Color::srgb(1.0, 1.0, 1.0);
            } else {
                **text = String::new();
            }
        }
    }

    // Update quantity text
    for (qty_text, mut text) in &mut qty_text_query {
        let idx = qty_text.index;
        if idx < inventory.slots.len() {
            if let Some(ref slot_data) = inventory.slots[idx] {
                if slot_data.quantity > 1 {
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

/// One-shot system: once the item atlas is loaded, inserts ImageNode on all
/// HotbarItemIcon entities so they display the correct sprite frame.
pub fn hydrate_hotbar_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_data: ResMut<ItemAtlasData>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    icon_query: Query<(Entity, &HotbarItemIcon), Without<ImageNode>>,
) {
    if icon_query.is_empty() {
        return; // Already hydrated or no icons spawned yet
    }

    // Lazy-load atlas on first call
    if !atlas_data.loaded {
        atlas_data.image = asset_server.load("sprites/items_atlas.png");
        atlas_data.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            13,
            10,
            None,
            None,
        ));
        atlas_data.loaded = true;
    }

    for (entity, icon) in &icon_query {
        let idx = icon.index;
        let atlas_index = if idx < inventory.slots.len() {
            inventory.slots[idx]
                .as_ref()
                .and_then(|s| item_registry.get(&s.item_id))
                .map(|def| def.sprite_index as usize)
                .unwrap_or(0)
        } else {
            0
        };

        commands.entity(entity).insert(ImageNode {
            image: atlas_data.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: atlas_data.layout.clone(),
                index: atlas_index,
            }),
            ..default()
        });
    }
}

/// Update icon atlas indices when inventory changes (runs after hydration).
pub fn update_hotbar_icons(
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    atlas_data: Res<ItemAtlasData>,
    mut icon_query: Query<(&HotbarItemIcon, &mut ImageNode)>,
) {
    if !atlas_data.loaded {
        return;
    }
    for (icon, mut img) in &mut icon_query {
        let idx = icon.index;
        if idx < inventory.slots.len() {
            if let Some(ref slot_data) = inventory.slots[idx] {
                if let Some(def) = item_registry.get(&slot_data.item_id) {
                    if let Some(ref mut atlas) = img.texture_atlas {
                        atlas.index = def.sprite_index as usize;
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MAP NAME UPDATE
// ═══════════════════════════════════════════════════════════════════════

/// Watches PlayerState.current_map for changes, displays a map name
/// indicator that holds visible then fades out.
pub fn update_map_name(
    time: Res<Time>,
    player: Res<PlayerState>,
    mut fade: ResMut<MapNameFadeTimer>,
    mut container_query: Query<(&Children, &mut BackgroundColor), With<HudMapName>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    // Detect map change.
    let map_changed = fade.last_map != Some(player.current_map);

    if map_changed {
        fade.last_map = Some(player.current_map);
        fade.display_timer.reset();
        fade.fade_timer.reset();
        fade.alpha = 1.0;
    }

    // Tick hold timer, then fade-out timer.
    if fade.alpha > 0.0 {
        if !fade.display_timer.finished() {
            fade.display_timer.tick(time.delta());
        } else {
            fade.fade_timer.tick(time.delta());
            let elapsed = fade.fade_timer.elapsed_secs();
            let duration = fade.fade_timer.duration().as_secs_f32();
            fade.alpha = (1.0 - elapsed / duration).clamp(0.0, 1.0);
        }
    }

    // Apply alpha and update text if map just changed.
    let bg_alpha = fade.alpha * 0.65;
    let current_alpha = fade.alpha;
    let current_map = player.current_map;

    for (children, mut bg_color) in &mut container_query {
        bg_color.0 = Color::srgba(0.0, 0.0, 0.0, bg_alpha);
        for &child in children.iter() {
            if let Ok((mut text, mut tc)) = text_query.get_mut(child) {
                if map_changed {
                    **text = map_display_name(current_map).to_string();
                }
                tc.0 = Color::srgba(1.0, 1.0, 1.0, current_alpha);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// OBJECTIVE DISPLAY
// ═══════════════════════════════════════════════════════════════════════

/// Shows or hides the tutorial objective text based on TutorialState.
pub fn update_objective_display(
    tutorial: Res<TutorialState>,
    mut objective_query: Query<(&Children, &mut BackgroundColor), With<HudObjective>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    for (children, mut bg_color) in &mut objective_query {
        if let Some(ref obj_id) = tutorial.current_objective {
            // Find the display text for this objective.
            let display = super::tutorial::OBJECTIVES
                .iter()
                .find(|(id, _)| *id == obj_id.as_str())
                .map(|(_, text)| *text)
                .unwrap_or("...");

            bg_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.65);
            for &child in children.iter() {
                if let Ok((mut text, mut tc)) = text_query.get_mut(child) {
                    **text = format!("> {}", display);
                    tc.0 = Color::srgb(1.0, 0.95, 0.7);
                }
            }
        } else {
            // No objective — hide.
            bg_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            for &child in children.iter() {
                if let Ok((mut text, mut tc)) = text_query.get_mut(child) {
                    text.0.clear();
                    tc.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTION PROMPT — show "[F] label" near interactables/NPCs
// ═══════════════════════════════════════════════════════════════════════

/// Shows a contextual interaction prompt when the player is near an
/// interactable object or NPC.
pub fn update_interaction_prompt(
    player_query: Query<&LogicalPosition, With<Player>>,
    interactable_query: Query<(&Transform, &Interactable)>,
    npc_query: Query<(&Npc, &Transform)>,
    npc_registry: Res<NpcRegistry>,
    mut prompt_query: Query<(&mut Text, &mut TextColor), With<HudInteractionPrompt>>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };
    let range = TILE_SIZE * 1.8;
    let mut best_label: Option<(f32, String)> = None;

    // Check interactable objects.
    for (tf, inter) in &interactable_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range && (best_label.is_none() || d < best_label.as_ref().unwrap().0) {
            best_label = Some((d, format!("[F] {}", inter.label)));
        }
    }

    // Check NPCs (closer NPC takes priority).
    for (npc, tf) in &npc_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range && (best_label.is_none() || d < best_label.as_ref().unwrap().0) {
            let name = npc_registry
                .npcs
                .get(&npc.id)
                .map(|def| def.name.as_str())
                .unwrap_or(&npc.id);
            let has_gift = inventory.slots
                .get(inventory.selected_slot)
                .and_then(|s| s.as_ref())
                .and_then(|slot| item_registry.get(&slot.item_id))
                .map(|def| def.category != ItemCategory::Tool)
                .unwrap_or(false);
            let label = if has_gift {
                format!("[F] Talk to {} | [R] Give Gift", name)
            } else {
                format!("[F] Talk to {}", name)
            };
            best_label = Some((d, label));
        }
    }

    for (mut text, mut tc) in &mut prompt_query {
        if let Some((_, label)) = &best_label {
            **text = label.clone();
            tc.0 = Color::srgb(1.0, 0.95, 0.7);
        } else {
            text.0.clear();
            tc.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
        }
    }
}
