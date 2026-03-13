use super::UiFontHandle;
use crate::shared::*;
use crate::world::map_data::{MapData, MapRegistry};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

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
    map_registry: Res<MapRegistry>,
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
                            spawn_map_layout(map_area, &font, current_map, &map_registry);
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
fn spawn_map_layout(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    current_map: MapId,
    map_registry: &MapRegistry,
) {
    let reachable_maps = reachable_world_maps(current_map, map_registry);

    for row in map_layout_rows() {
        spawn_map_row(parent, font, current_map, &reachable_maps, row);
    }

    let laid_out_maps = map_layout_rows()
        .iter()
        .flat_map(|row| row.iter())
        .filter_map(|node| match node {
            MapNode::Location(map_id) => Some(*map_id),
            MapNode::Empty => None,
        })
        .collect::<HashSet<_>>();

    let mut overflow_maps = reachable_maps
        .iter()
        .copied()
        .filter(|map_id| !laid_out_maps.contains(map_id))
        .collect::<Vec<_>>();
    overflow_maps.sort_unstable_by_key(|map_id| map_id_display_name(*map_id));

    if !overflow_maps.is_empty() {
        let names = overflow_maps
            .iter()
            .map(|map_id| map_id_display_name(*map_id))
            .collect::<Vec<_>>()
            .join(", ");
        parent.spawn((
            Text::new(format!("Other connected areas: {}", names)),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.7, 0.6)),
        ));
    }
}

const MAP_CELL_WIDTH: f32 = 100.0;

const MAP_LAYOUT_ROWS: &[&[MapNode]] = &[
    &[
        MapNode::Empty,
        MapNode::Empty,
        MapNode::Location(MapId::SnowMountain),
        MapNode::Empty,
        MapNode::Empty,
    ],
    &[
        MapNode::Location(MapId::MineEntrance),
        MapNode::Empty,
        MapNode::Location(MapId::Farm),
        MapNode::Location(MapId::Forest),
        MapNode::Location(MapId::DeepForest),
    ],
    &[
        MapNode::Location(MapId::Mine),
        MapNode::Empty,
        MapNode::Location(MapId::PlayerHouse),
        MapNode::Empty,
        MapNode::Empty,
    ],
    &[
        MapNode::Location(MapId::TownWest),
        MapNode::Empty,
        MapNode::Location(MapId::Town),
        MapNode::Location(MapId::Beach),
        MapNode::Location(MapId::CoralIsland),
    ],
    &[
        MapNode::Location(MapId::TownHouseWest),
        MapNode::Location(MapId::TownHouseEast),
        MapNode::Location(MapId::Library),
        MapNode::Location(MapId::Tavern),
        MapNode::Location(MapId::GeneralStore),
    ],
    &[
        MapNode::Empty,
        MapNode::Empty,
        MapNode::Location(MapId::AnimalShop),
        MapNode::Location(MapId::Blacksmith),
        MapNode::Empty,
    ],
];

fn map_layout_rows() -> &'static [&'static [MapNode]] {
    MAP_LAYOUT_ROWS
}

#[derive(Clone, Copy)]
enum MapNode {
    Location(MapId),
    Empty,
}

fn spawn_map_row(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    current_map: MapId,
    reachable_maps: &HashSet<MapId>,
    nodes: &[MapNode],
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },))
        .with_children(|row| {
            for node in nodes {
                row.spawn(Node {
                    width: Val::Px(MAP_CELL_WIDTH),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|cell| {
                    if let MapNode::Location(map_id) = node {
                        if !reachable_maps.contains(map_id) {
                            return;
                        }

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
                        cell.spawn((
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
                });
            }
        });
}

fn reachable_world_maps(start: MapId, map_registry: &MapRegistry) -> HashSet<MapId> {
    let adjacency = build_map_adjacency(map_registry);
    let mut visited = HashSet::new();
    let mut pending = VecDeque::from([start]);

    while let Some(map_id) = pending.pop_front() {
        if !visited.insert(map_id) {
            continue;
        }

        if let Some(neighbors) = adjacency.get(&map_id) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    pending.push_back(neighbor);
                }
            }
        }
    }

    visited
}

fn build_map_adjacency(map_registry: &MapRegistry) -> HashMap<MapId, Vec<MapId>> {
    let mut adjacency = HashMap::new();

    for (&map_id, map_data) in &map_registry.maps {
        adjacency.entry(map_id).or_default();

        for linked_map in linked_maps(map_data) {
            push_link(&mut adjacency, map_id, linked_map);
            push_link(&mut adjacency, linked_map, map_id);
        }
    }

    adjacency
}

fn linked_maps(map_data: &MapData) -> Vec<MapId> {
    let mut links = Vec::new();

    for transition in &map_data.transitions {
        push_unique_map(&mut links, transition.to_map);
    }
    for door in &map_data.doors {
        push_unique_map(&mut links, door.to_map);
    }
    for (map_id, _) in [
        map_data.edges.north.as_ref(),
        map_data.edges.south.as_ref(),
        map_data.edges.east.as_ref(),
        map_data.edges.west.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        push_unique_map(&mut links, *map_id);
    }

    links
}

fn push_link(adjacency: &mut HashMap<MapId, Vec<MapId>>, from: MapId, to: MapId) {
    let neighbors = adjacency.entry(from).or_default();
    push_unique_map(neighbors, to);
}

fn push_unique_map(maps: &mut Vec<MapId>, map_id: MapId) {
    if !maps.contains(&map_id) {
        maps.push(map_id);
    }
}

fn map_id_display_name(map_id: MapId) -> &'static str {
    match map_id {
        MapId::Farm => "Farm",
        MapId::Town => "Town",
        MapId::TownWest => "West Willowbrook",
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
        MapId::SnowMountain => "Snowy Mountain",
    }
}

pub fn despawn_map_screen(mut commands: Commands, query: Query<Entity, With<MapScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
