use super::UiFontHandle;
use crate::input::{TouchZone, TouchZoneState};
use crate::shared::*;
use bevy::prelude::*;

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

/// Marker for the weather icon image node.
#[derive(Component)]
pub struct HudWeatherIcon;

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

/// Lazily-loaded atlas for weather icons (weather_icons.png: 4×6 grid, 16×16).
#[derive(Resource, Default)]
pub struct WeatherIconAtlas {
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

/// Component for floating "+Xg" text that drifts upward and fades out.
#[derive(Component)]
pub struct FloatingGoldText {
    /// Upward velocity in pixels per second.
    pub velocity: f32,
    /// Total lifetime of the animation.
    pub lifetime: f32,
    /// Time elapsed so far.
    pub elapsed: f32,
}

/// Cooldown resource to prevent spamming floating gold text.
#[derive(Resource)]
pub struct FloatingGoldCooldown {
    pub timer: Timer,
}

/// Marker for the early-game controls hint at the bottom of the screen.
#[derive(Component)]
pub struct HudControlsHint;

/// Timer resource that counts down the 60-second controls hint display.
#[derive(Resource)]
pub struct ControlsHintTimer {
    pub timer: Timer,
}

/// Cache for interaction prompt scans so we can skip full proximity rescans
/// when player context hasn't changed.
#[derive(Resource, Default)]
pub struct InteractionPromptCache {
    pub map: Option<MapId>,
    pub player_tile: Option<(i32, i32)>,
    pub best_label: Option<String>,
    pub can_gift: Option<bool>,
    pub interact_key: Option<KeyCode>,
    pub secondary_key: Option<KeyCode>,
}

// ═══════════════════════════════════════════════════════════════════════
// EAGER PRELOADING — runs on OnEnter(Playing) before any HUD update
// ═══════════════════════════════════════════════════════════════════════

/// Eagerly loads the item atlas so hotbar icons are ready on the first frame.
pub fn preload_item_atlas(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_data: ResMut<ItemAtlasData>,
) {
    if atlas_data.loaded {
        return;
    }
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

/// Eagerly loads the weather icon atlas so icons are ready on the first frame.
pub fn preload_weather_icon_atlas(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas: ResMut<WeatherIconAtlas>,
) {
    if atlas.loaded {
        return;
    }
    atlas.image = asset_server.load("ui/weather_icons.png");
    atlas.layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        6,
        None,
        None,
    ));
    atlas.loaded = true;
}

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
                                HudWeatherIcon,
                                ImageNode::default(),
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
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
    commands
        .spawn((
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
    commands
        .spawn((
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

    // ─── CONTROLS HINT — absolute position, bottom-center, above hotbar ───
    commands
        .spawn((
            HudControlsHint,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(64.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("WASD: Move | Space: Use Tool | F: Interact | E: Inventory"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
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

    // Initialise the controls hint timer (60 real seconds).
    commands.insert_resource(ControlsHintTimer {
        timer: Timer::from_seconds(60.0, TimerMode::Once),
    });
    commands.insert_resource(InteractionPromptCache::default());
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
                                font_size: 10.0,
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
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
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
                                font_size: 12.0,
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
    controls_hint_query: Query<Entity, With<HudControlsHint>>,
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
    for entity in &controls_hint_query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<MapNameFadeTimer>();
    commands.remove_resource::<ControlsHintTimer>();
    commands.remove_resource::<InteractionPromptCache>();
}

// ─── MAP NAME DISPLAY HELPER ──────────────────────────────────────────

fn map_display_name(map: MapId) -> &'static str {
    match map {
        MapId::Farm => "Hearthfield Farm",
        MapId::Town => "Willowbrook",
        MapId::Beach => "Tide Pool Beach",
        MapId::Forest => "Briarwood Forest",
        MapId::MineEntrance => "The Mines",
        MapId::Mine => "Mine Floor",
        MapId::PlayerHouse => "Home",
        MapId::GeneralStore => "General Store",
        MapId::AnimalShop => "Animal Shop",
        MapId::Blacksmith => "Elena's Forge",
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

/// Update the weather icon atlas sprite to match current weather.
/// Lazy-loads weather_icons.png (4 cols × 6 rows, 16×16 tiles).
pub fn update_weather_icon(
    mut commands: Commands,
    calendar: Res<Calendar>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas: ResMut<WeatherIconAtlas>,
    query: Query<Entity, With<HudWeatherIcon>>,
) {
    if !calendar.is_changed() {
        return;
    }
    // Lazy-load the weather icons atlas
    if !atlas.loaded {
        atlas.image = asset_server.load("ui/weather_icons.png");
        atlas.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            4,
            6,
            None,
            None,
        ));
        atlas.loaded = true;
    }

    // Map weather to icon index in the sheet
    // Row 0: sun variants, Row 1: cloud/overcast, Row 2: rain, Row 3: storm, Row 4: snow
    let icon_index = match calendar.weather {
        Weather::Sunny => 0,   // sun icon
        Weather::Rainy => 8,   // rain icon
        Weather::Stormy => 12, // storm icon
        Weather::Snowy => 16,  // snow icon
    };

    for entity in &query {
        commands.entity(entity).insert(ImageNode {
            image: atlas.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: atlas.layout.clone(),
                index: icon_index,
            }),
            ..default()
        });
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
    time: Res<Time>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<HudStaminaFill>>,
) {
    let ratio = (player.stamina / player.max_stamina).clamp(0.0, 1.0);
    let is_critical = ratio < 0.25;

    // Skip update if player hasn't changed and we're not in critical pulse mode
    if !player.is_changed() && !is_critical {
        return;
    }

    for (mut node, mut bg) in &mut query {
        node.width = Val::Percent(ratio * 100.0);
        let base_color = if ratio > 0.6 {
            Color::srgb(0.2, 0.8, 0.2)
        } else if ratio >= 0.3 {
            Color::srgb(0.9, 0.7, 0.1)
        } else {
            Color::srgb(0.9, 0.2, 0.2)
        };

        if is_critical {
            // Gentle pulse: alpha oscillates between 0.70 and 1.0
            let pulse = (time.elapsed_secs() * 3.0).sin() * 0.15 + 0.85;
            let Color::Srgba(srgba) = base_color else {
                *bg = BackgroundColor(base_color);
                continue;
            };
            *bg = BackgroundColor(Color::srgba(srgba.red, srgba.green, srgba.blue, pulse));
        } else {
            *bg = BackgroundColor(base_color);
        }
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
        let tier = player
            .tools
            .get(&player.equipped_tool)
            .copied()
            .unwrap_or(ToolTier::Basic);
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
    mut item_text_query: Query<
        (&HotbarItemText, &mut Text, &mut TextColor),
        Without<HotbarQuantityText>,
    >,
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
        *vis = if has_item {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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
// CONTROLS HINT — subtle overlay for Day 1 Year 1, auto-hides after 60s
// ═══════════════════════════════════════════════════════════════════════

/// Shows the controls hint only on Day 1, Year 1 for the first 60 real
/// seconds of gameplay, then fades it out. Hidden entirely outside Day 1.
pub fn update_controls_hint(
    time: Res<Time>,
    calendar: Res<Calendar>,
    mut timer: ResMut<ControlsHintTimer>,
    mut hint_query: Query<(&Children, &mut Visibility), With<HudControlsHint>>,
    mut text_query: Query<&mut TextColor>,
) {
    let is_day1 = calendar.day == 1 && calendar.year == 1;

    for (children, mut vis) in &mut hint_query {
        if !is_day1 || timer.timer.finished() {
            *vis = Visibility::Hidden;
            continue;
        }

        *vis = Visibility::Inherited;
        timer.timer.tick(time.delta());

        // Fade out over the last 5 seconds.
        let remaining = timer.timer.remaining_secs();
        let alpha = if remaining < 5.0 {
            (remaining / 5.0).clamp(0.0, 1.0) * 0.45
        } else {
            0.45
        };

        for &child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                tc.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTION PROMPT — show "[F] label" near interactables/NPCs
// ═══════════════════════════════════════════════════════════════════════

fn key_display(code: KeyCode) -> String {
    match code {
        KeyCode::KeyA => "A".into(),
        KeyCode::KeyB => "B".into(),
        KeyCode::KeyC => "C".into(),
        KeyCode::KeyD => "D".into(),
        KeyCode::KeyE => "E".into(),
        KeyCode::KeyF => "F".into(),
        KeyCode::KeyG => "G".into(),
        KeyCode::KeyH => "H".into(),
        KeyCode::KeyI => "I".into(),
        KeyCode::KeyJ => "J".into(),
        KeyCode::KeyK => "K".into(),
        KeyCode::KeyL => "L".into(),
        KeyCode::KeyM => "M".into(),
        KeyCode::KeyN => "N".into(),
        KeyCode::KeyO => "O".into(),
        KeyCode::KeyP => "P".into(),
        KeyCode::KeyQ => "Q".into(),
        KeyCode::KeyR => "R".into(),
        KeyCode::KeyS => "S".into(),
        KeyCode::KeyT => "T".into(),
        KeyCode::KeyU => "U".into(),
        KeyCode::KeyV => "V".into(),
        KeyCode::KeyW => "W".into(),
        KeyCode::KeyX => "X".into(),
        KeyCode::KeyY => "Y".into(),
        KeyCode::KeyZ => "Z".into(),
        KeyCode::Space => "Space".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Escape => "Esc".into(),
        other => format!("{:?}", other),
    }
}

/// Shows a contextual interaction prompt when the player is near an
/// interactable object or NPC.
#[allow(clippy::too_many_arguments)]
pub fn update_interaction_prompt(
    player_state: Res<PlayerState>,
    player_query: Query<&LogicalPosition, With<Player>>,
    moved_interactable_query: Query<(), (With<Interactable>, Changed<Transform>)>,
    moved_npc_query: Query<(), (With<Npc>, Changed<Transform>)>,
    moved_chest_query: Query<(), (With<crate::world::chests::ChestMarker>, Changed<Transform>)>,
    interactable_query: Query<(&Transform, &Interactable)>,
    npc_query: Query<(&Npc, &Transform)>,
    chest_query: Query<&Transform, With<crate::world::chests::ChestMarker>>,
    npc_registry: Res<NpcRegistry>,
    mut cache: ResMut<InteractionPromptCache>,
    mut prompt_query: Query<(&mut Text, &mut TextColor), With<HudInteractionPrompt>>,
    inventory: Res<Inventory>,
    item_registry: Res<ItemRegistry>,
    bindings: Res<KeyBindings>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };
    let player_tile = world_to_grid(player_pos.0.x, player_pos.0.y);
    let player_tile_key = (player_tile.x, player_tile.y);
    let current_map = player_state.current_map;
    let has_gift = inventory
        .slots
        .get(inventory.selected_slot)
        .and_then(|s| s.as_ref())
        .and_then(|slot| item_registry.get(&slot.item_id))
        .map(|def| def.category != ItemCategory::Tool)
        .unwrap_or(false);
    let can_reuse_cache = cache.map == Some(current_map)
        && cache.player_tile == Some(player_tile_key)
        && cache.can_gift == Some(has_gift)
        && cache.interact_key == Some(bindings.interact)
        && cache.secondary_key == Some(bindings.tool_secondary)
        && !npc_registry.is_changed()
        && moved_interactable_query.is_empty()
        && moved_npc_query.is_empty()
        && moved_chest_query.is_empty();

    if can_reuse_cache {
        return;
    }

    let range = TILE_SIZE * 1.8;
    let mut best_label: Option<(f32, String)> = None;
    let interact_key = key_display(bindings.interact);
    let secondary_key = key_display(bindings.tool_secondary);

    // Check interactable objects.
    for (tf, inter) in &interactable_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range && best_label.as_ref().is_none_or(|b| d < b.0) {
            best_label = Some((d, format!("[{}] {}", interact_key, inter.label)));
        }
    }

    // Check storage chests (use their own component, not Interactable).
    for tf in &chest_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range && best_label.as_ref().is_none_or(|b| d < b.0) {
            best_label = Some((d, format!("[{}] Storage", interact_key)));
        }
    }

    // Check NPCs (closer NPC takes priority).
    for (npc, tf) in &npc_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range && best_label.as_ref().is_none_or(|b| d < b.0) {
            let name = npc_registry
                .npcs
                .get(&npc.id)
                .map(|def| def.name.as_str())
                .unwrap_or(&npc.id);
            let label = if has_gift {
                format!(
                    "[{}] Talk to {} | [{}] Give Gift",
                    interact_key,
                    name,
                    secondary_key
                )
            } else {
                format!("[{}] Talk to {}", interact_key, name)
            };
            best_label = Some((d, label));
        }
    }

    let best_label_text = best_label.map(|(_, label)| label);
    cache.map = Some(current_map);
    cache.player_tile = Some(player_tile_key);
    cache.best_label = best_label_text.clone();
    cache.can_gift = Some(has_gift);
    cache.interact_key = Some(bindings.interact);
    cache.secondary_key = Some(bindings.tool_secondary);

    for (mut text, mut tc) in &mut prompt_query {
        if let Some(label) = &best_label_text {
            **text = label.clone();
            tc.0 = Color::srgb(1.0, 0.95, 0.7);
        } else {
            text.0.clear();
            tc.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FLOATING GOLD TEXT — "+Xg" floats up near gold counter on gold gain
// ═══════════════════════════════════════════════════════════════════════

/// Spawns a floating "+Xg" text entity when gold is gained.
/// Rate-limited to at most one per 0.5 seconds to prevent spam.
pub fn spawn_floating_gold_text(
    mut commands: Commands,
    mut gold_events: EventReader<GoldChangeEvent>,
    font_handle: Res<UiFontHandle>,
    mut cooldown: ResMut<FloatingGoldCooldown>,
    time: Res<Time>,
) {
    cooldown.timer.tick(time.delta());

    for event in gold_events.read() {
        if event.amount <= 0 {
            continue;
        }
        if !cooldown.timer.finished() {
            continue;
        }
        cooldown.timer.reset();

        let font = font_handle.0.clone();
        commands.spawn((
            FloatingGoldText {
                velocity: 13.0, // ~20px over 1.5s
                lifetime: 1.5,
                elapsed: 0.0,
            },
            Text::new(format!("+{}g", event.amount)),
            TextFont {
                font,
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.84, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                // Position near the gold counter in the top-right area.
                top: Val::Px(48.0),
                right: Val::Px(140.0),
                ..default()
            },
            PickingBehavior::IGNORE,
        ));
    }
}

/// Animates floating gold text: moves upward, fades out, then despawns.
pub fn update_floating_gold_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingGoldText, &mut Node, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fgt, mut node, mut tc) in &mut query {
        fgt.elapsed += dt;

        if fgt.elapsed >= fgt.lifetime {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Move upward: reduce top offset.
        let current_top = match node.top {
            Val::Px(v) => v,
            _ => 48.0,
        };
        node.top = Val::Px(current_top - fgt.velocity * dt);

        // Fade out based on progress through lifetime.
        let progress = (fgt.elapsed / fgt.lifetime).clamp(0.0, 1.0);
        // Start fading after 40% of lifetime.
        let alpha = if progress < 0.4 {
            1.0
        } else {
            1.0 - ((progress - 0.4) / 0.6).clamp(0.0, 1.0)
        };
        tc.0 = Color::srgba(1.0, 0.84, 0.0, alpha);
    }
}

/// Despawns all floating gold text entities (called on HUD exit).
pub fn despawn_floating_gold_text(
    mut commands: Commands,
    query: Query<Entity, With<FloatingGoldText>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TOUCH CONTROLS OVERLAY — functional on-screen buttons for mobile
// ═══════════════════════════════════════════════════════════════════════

/// Marker component for the touch controls overlay root node.
#[derive(Component)]
pub struct TouchControlsOverlay;

/// Marker for a touch overlay button that should highlight when its zone
/// is pressed.  The `TouchZone` identifies which zone it maps to.
#[derive(Component)]
pub struct TouchButton(pub TouchZone);

/// Tracks how long since the last keyboard or gamepad input was detected,
/// used to decide when to show the touch overlay for mobile players.
#[derive(Resource)]
pub struct TouchOverlayTimer {
    /// Seconds since last keyboard/gamepad input.
    pub idle_secs: f32,
    /// Whether the overlay is currently visible.
    pub visible: bool,
}

impl Default for TouchOverlayTimer {
    fn default() -> Self {
        Self {
            idle_secs: 0.0,
            visible: false,
        }
    }
}

/// Seconds of keyboard/gamepad inactivity before showing touch overlay.
const TOUCH_OVERLAY_IDLE_THRESHOLD: f32 = 5.0;

/// Normal (unpressed) background alpha for touch buttons.
const TOUCH_BTN_ALPHA: f32 = 0.18;
/// Pressed (highlighted) background alpha for touch buttons.
const TOUCH_BTN_PRESSED_ALPHA: f32 = 0.45;

/// Spawns the touch controls overlay (hidden by default, auto-shows on idle).
/// Each button is tagged with `TouchButton(zone)` for highlight feedback.
pub fn spawn_touch_overlay(mut commands: Commands, font_handle: Res<UiFontHandle>) {
    let font = font_handle.0.clone();
    let btn_style = Node {
        width: Val::Px(60.0),
        height: Val::Px(60.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::all(Val::Px(4.0)),
        ..default()
    };
    let btn_bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, TOUCH_BTN_ALPHA));
    let btn_radius = BorderRadius::all(Val::Px(30.0)); // circular buttons

    commands
        .spawn((
            TouchControlsOverlay,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Auto,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::horizontal(Val::Px(16.0)),
                ..default()
            },
            Visibility::Hidden,
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            // ── Left side: D-pad ──
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|dpad| {
                    // Top row: Up
                    dpad.spawn((
                        TouchButton(TouchZone::DpadUp),
                        btn_style.clone(),
                        btn_bg,
                        btn_radius,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("\u{25B2}"), // ▲
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                    // Middle row: Left, Down, Right
                    dpad.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    })
                    .with_children(|row| {
                        // Left
                        row.spawn((
                            TouchButton(TouchZone::DpadLeft),
                            btn_style.clone(),
                            btn_bg,
                            btn_radius,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("\u{25C0}"), // ◀
                                TextFont {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                        // Down
                        row.spawn((
                            TouchButton(TouchZone::DpadDown),
                            btn_style.clone(),
                            btn_bg,
                            btn_radius,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("\u{25BC}"), // ▼
                                TextFont {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                        // Right
                        row.spawn((
                            TouchButton(TouchZone::DpadRight),
                            btn_style.clone(),
                            btn_bg,
                            btn_radius,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("\u{25B6}"), // ▶
                                TextFont {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
                });

            // ── Right side: Action buttons (diamond layout) ──
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|actions| {
                    // Top: Use / Tool (Space equivalent)
                    actions
                        .spawn((
                            TouchButton(TouchZone::ActionUse),
                            btn_style.clone(),
                            btn_bg,
                            btn_radius,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Use"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    // Middle row: Menu, Item, Talk
                    actions
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        })
                        .with_children(|row| {
                            // Left: Menu (Esc equivalent)
                            row.spawn((
                                TouchButton(TouchZone::ActionMenu),
                                btn_style.clone(),
                                btn_bg,
                                btn_radius,
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("Menu"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                            // Center: Item / secondary (R equivalent)
                            row.spawn((
                                TouchButton(TouchZone::ActionItem),
                                btn_style.clone(),
                                btn_bg,
                                btn_radius,
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("Item"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                            // Right: Talk / Interact (F equivalent)
                            row.spawn((
                                TouchButton(TouchZone::ActionTalk),
                                btn_style.clone(),
                                btn_bg,
                                btn_radius,
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("Talk"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                });
        });

    commands.insert_resource(TouchOverlayTimer::default());
}

/// Shows the touch overlay when no keyboard/gamepad input detected for a few
/// seconds, or immediately if any touch input is detected.  Hides it when
/// keyboard or gamepad input resumes.
///
/// Also updates the visual highlight on each TouchButton based on whether its
/// corresponding touch zone is currently held.
#[allow(clippy::too_many_arguments)]
pub fn update_touch_overlay(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<bevy::input::touch::Touches>,
    gamepads: Query<&Gamepad>,
    zone_state: Res<TouchZoneState>,
    mut timer: ResMut<TouchOverlayTimer>,
    mut overlay_query: Query<&mut Visibility, With<TouchControlsOverlay>>,
    mut btn_query: Query<(&TouchButton, &mut BackgroundColor)>,
) {
    let has_keyboard =
        keys.get_just_pressed().next().is_some() || keys.get_pressed().next().is_some();
    let has_gamepad = gamepads.iter().next().is_some();
    let has_touch = touches.iter().next().is_some() || touches.iter_just_pressed().next().is_some();

    if has_keyboard || has_gamepad {
        timer.idle_secs = 0.0;
        timer.visible = false;
    } else if has_touch {
        // Touch detected — show immediately and reset idle timer.
        timer.idle_secs = TOUCH_OVERLAY_IDLE_THRESHOLD;
        timer.visible = true;
    } else {
        timer.idle_secs += time.delta_secs();
        if timer.idle_secs >= TOUCH_OVERLAY_IDLE_THRESHOLD {
            timer.visible = true;
        }
    }

    for mut vis in &mut overlay_query {
        *vis = if timer.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    // Update button highlights based on zone state.
    for (btn, mut bg) in &mut btn_query {
        let alpha = if zone_state.is_held(btn.0) {
            TOUCH_BTN_PRESSED_ALPHA
        } else {
            TOUCH_BTN_ALPHA
        };
        *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, alpha));
    }
}

/// Despawns the touch controls overlay (called on HUD exit).
pub fn despawn_touch_overlay(
    mut commands: Commands,
    query: Query<Entity, With<TouchControlsOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<TouchOverlayTimer>();
}
