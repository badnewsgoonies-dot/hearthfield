use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct MapScreenRoot;

#[derive(Component)]
pub struct MapLocationLabel {
    #[allow(dead_code)]
    pub map_id: MapId,
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_map_screen(
    mut commands: Commands,
    font_handle: Res<UiFontHandle>,
    player_state: Res<PlayerState>,
) {
    let font = font_handle.0.clone();
    let current_map = player_state.current_map;

    commands
        .spawn((
            MapScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // Main panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(560.0),
                        height: Val::Px(480.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.1, 0.07, 0.97)),
                    BorderColor(Color::srgb(0.3, 0.55, 0.3)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("HEARTHFIELD MAP"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 1.0, 0.7)),
                    ));

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.55, 0.3)),
                    ));

                    // Map layout area
                    panel
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },))
                        .with_children(|map_area| {
                            spawn_map_layout(map_area, &font, current_map);
                        });

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.55, 0.3)),
                    ));

                    // You are here indicator
                    let location_name = map_id_display_name(current_map);
                    panel.spawn((
                        Text::new(format!("★  You are here: {}", location_name)),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));

                    // Close hint
                    panel.spawn((
                        Text::new("Press M or Esc to close"),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.7, 0.6)),
                    ));
                });
        });
}

/// Spawns the ASCII-art style map layout showing all locations and connections.
fn spawn_map_layout(parent: &mut ChildBuilder, font: &Handle<Font>, current_map: MapId) {
    // Row 1:  [Forest]          [Mine]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[
            MapNode::Location(MapId::Forest),
            MapNode::Spacer("          "),
            MapNode::Location(MapId::MineEntrance),
        ],
    );

    // Row 2:      \               |
    spawn_map_text_row(parent, font, "    \\                    |");

    // Row 3:  [Farm] ------  [Town]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[
            MapNode::Location(MapId::Farm),
            MapNode::Spacer(" ----- "),
            MapNode::Location(MapId::Town),
        ],
    );

    // Row 4:     |               |
    spawn_map_text_row(parent, font, "   |                     |");

    // Row 5:  [Beach]       [General Store]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[
            MapNode::Location(MapId::Beach),
            MapNode::Spacer("       "),
            MapNode::Location(MapId::GeneralStore),
        ],
    );

    // Row 6:                [Animal Shop]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[
            MapNode::Spacer("                    "),
            MapNode::Location(MapId::AnimalShop),
        ],
    );

    // Row 7:                [Blacksmith]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[
            MapNode::Spacer("                    "),
            MapNode::Location(MapId::Blacksmith),
        ],
    );

    // Row 8:  [Player House]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[MapNode::Location(MapId::PlayerHouse)],
    );

    // Row 9:  [Town House W]  [Town House E]
    spawn_map_row(
        parent,
        font,
        current_map,
        &[
            MapNode::Location(MapId::TownHouseWest),
            MapNode::Spacer("   "),
            MapNode::Location(MapId::TownHouseEast),
        ],
    );
}

enum MapNode {
    Location(MapId),
    Spacer(&'static str),
}

fn spawn_map_row(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    current_map: MapId,
    nodes: &[MapNode],
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(0.0),
            ..default()
        },))
        .with_children(|row| {
            for node in nodes {
                match node {
                    MapNode::Location(map_id) => {
                        let is_current = *map_id == current_map;
                        let name = map_id_display_name(*map_id);
                        let label = if is_current {
                            format!("[★ {}]", name)
                        } else {
                            format!("[{}]", name)
                        };
                        let color = if is_current {
                            Color::srgb(1.0, 0.9, 0.3)
                        } else {
                            Color::srgb(0.75, 0.95, 0.7)
                        };
                        row.spawn((
                            MapLocationLabel { map_id: *map_id },
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                    MapNode::Spacer(text) => {
                        row.spawn((
                            Text::new(*text),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.65, 0.5)),
                        ));
                    }
                }
            }
        });
}

fn spawn_map_text_row(parent: &mut ChildBuilder, font: &Handle<Font>, text: &'static str) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font: font.clone(),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.65, 0.5)),
    ));
}

fn map_id_display_name(map_id: MapId) -> &'static str {
    match map_id {
        MapId::Farm => "Farm",
        MapId::Town => "Town",
        MapId::Beach => "Beach",
        MapId::Forest => "Forest",
        MapId::DeepForest => "Deep Forest",
        MapId::MineEntrance => "Mine",
        MapId::Mine => "Mine (Deep)",
        MapId::PlayerHouse => "Player House",
        MapId::TownHouseWest => "Town House West",
        MapId::TownHouseEast => "Town House East",
        MapId::GeneralStore => "General Store",
        MapId::AnimalShop => "Animal Shop",
        MapId::Blacksmith => "Blacksmith",
        MapId::Library => "Library",
        MapId::Tavern => "Tavern",
        MapId::CoralIsland => "Coral Island",
    }
}

pub fn despawn_map_screen(mut commands: Commands, query: Query<Entity, With<MapScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
