//! Building Upgrade menu UI — lets the player upgrade Coop, Barn, House, and Silo.
//!
//! Activated by entering `GameState::BuildingUpgrade`. Uses the same
//! `MenuAction` resource that all other overlay menus consume.

use bevy::prelude::*;
use crate::economy::buildings::BuildingLevels;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// LOCAL TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Marker component for the root UI node so we can despawn everything on exit.
#[derive(Component)]
pub struct BuildingUpgradeMenuRoot;

/// Marker for individual row nodes so we can update highlight colours.
#[derive(Component)]
pub struct BuildingRow {
    pub index: usize,
}

/// Marker for the name/status text of each row.
#[derive(Component)]
pub struct BuildingRowText {
    pub index: usize,
}

/// Marker for the cost text of each row.
#[derive(Component)]
pub struct BuildingRowCost {
    pub index: usize,
}

/// Marker for the status feedback text at the bottom.
#[derive(Component)]
pub struct BuildingUpgradeStatusText;

/// A single entry in the upgrade menu.
#[derive(Clone, Debug)]
struct UpgradeEntry {
    building: BuildingKind,
    label: &'static str,
    from_tier: BuildingTier,
    to_tier: Option<BuildingTier>,
    cost_gold: u32,
    cost_materials: Vec<(&'static str, u8)>,
    /// false if already at max tier, or an upgrade is currently in progress
    available: bool,
    status_line: String,
}

/// UI-local resource tracking cursor and computed entries.
#[derive(Resource, Default)]
pub struct BuildingUpgradeMenuState {
    cursor: usize,
    entries: Vec<UpgradeEntry>,
    status_message: String,
    status_timer: f32,
}

use crate::economy::buildings::upgrade_cost;

fn tier_label(tier: BuildingTier) -> &'static str {
    match tier {
        BuildingTier::None => "None",
        BuildingTier::Basic => "Basic",
        BuildingTier::Big => "Big",
        BuildingTier::Deluxe => "Deluxe",
    }
}

fn building_label(kind: BuildingKind) -> &'static str {
    match kind {
        BuildingKind::House => "House",
        BuildingKind::Coop => "Coop",
        BuildingKind::Barn => "Barn",
        BuildingKind::Silo => "Silo",
    }
}

/// Determine the current tier of a building from the world state.
fn current_tier(
    kind: BuildingKind,
    building_levels: &BuildingLevels,
    house_state: &HouseState,
) -> BuildingTier {
    match kind {
        BuildingKind::House => match house_state.tier {
            HouseTier::Basic => BuildingTier::Basic,
            HouseTier::Big => BuildingTier::Big,
            HouseTier::Deluxe => BuildingTier::Deluxe,
        },
        BuildingKind::Coop => building_levels.coop_tier,
        BuildingKind::Barn => building_levels.barn_tier,
        BuildingKind::Silo => {
            if building_levels.silo_built {
                BuildingTier::Basic
            } else {
                BuildingTier::None
            }
        }
    }
}

/// Build the list of upgrade entries.
fn build_entries(
    building_levels: &BuildingLevels,
    house_state: &HouseState,
) -> Vec<UpgradeEntry> {
    let buildings = [
        BuildingKind::Coop,
        BuildingKind::Barn,
        BuildingKind::House,
        BuildingKind::Silo,
    ];

    let upgrade_in_progress = building_levels.upgrade_in_progress.is_some();

    buildings
        .iter()
        .map(|&kind| {
            let from = current_tier(kind, building_levels, house_state);
            let next = from.next();

            let (cost_gold, cost_materials, available, status_line) = match next {
                Some(to) => {
                    let (gold, mats) = upgrade_cost(kind, to);
                    if gold == 0 && mats.is_empty() {
                        // Invalid upgrade combo (shouldn't happen, but guard)
                        (0, vec![], false, "MAX".to_string())
                    } else if upgrade_in_progress {
                        // Check if THIS building is the one being upgraded
                        let is_this = building_levels
                            .upgrade_in_progress
                            .as_ref()
                            .map(|(b, _, _)| *b == kind)
                            .unwrap_or(false);
                        if is_this {
                            let days = building_levels
                                .upgrade_in_progress
                                .as_ref()
                                .map(|(_, _, d)| *d)
                                .unwrap_or(0);
                            (gold, mats, false, format!("BUILDING ({} days left)", days))
                        } else {
                            (gold, mats, false, "Another upgrade in progress".to_string())
                        }
                    } else {
                        let cost_str = format_cost(gold, &mats);
                        (gold, mats, true, cost_str)
                    }
                }
                None => (0, vec![], false, "MAX".to_string()),
            };

            UpgradeEntry {
                building: kind,
                label: building_label(kind),
                from_tier: from,
                to_tier: next,
                cost_gold,
                cost_materials,
                available,
                status_line,
            }
        })
        .collect()
}

fn format_cost(gold: u32, materials: &[(&str, u8)]) -> String {
    let mut parts = vec![format!("{}g", gold)];
    for &(mat, qty) in materials {
        parts.push(format!("{} {}", qty, mat));
    }
    parts.join(" + ")
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_building_upgrade_menu(
    mut commands: Commands,
    building_levels: Res<BuildingLevels>,
    house_state: Res<HouseState>,
) {
    let entries = build_entries(&building_levels, &house_state);

    commands.insert_resource(BuildingUpgradeMenuState {
        cursor: 0,
        entries: entries.clone(),
        status_message: String::new(),
        status_timer: 0.0,
    });

    commands
        .spawn((
            BuildingUpgradeMenuRoot,
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
                        width: Val::Px(520.0),
                        min_height: Val::Px(300.0),
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
                    // Title
                    panel.spawn((
                        Text::new("BUILDING UPGRADES"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.6)),
                    ));

                    // Status / feedback text
                    panel.spawn((
                        BuildingUpgradeStatusText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.9, 0.5)),
                    ));

                    // Building rows
                    for (i, entry) in entries.iter().enumerate() {
                        let row_text = if let Some(to) = entry.to_tier {
                            format!(
                                "{}: {} -> {}",
                                entry.label,
                                tier_label(entry.from_tier),
                                tier_label(to),
                            )
                        } else {
                            format!(
                                "{}: {} (MAX)",
                                entry.label,
                                tier_label(entry.from_tier),
                            )
                        };

                        let bg = if i == 0 {
                            Color::srgba(0.35, 0.3, 0.2, 0.9)
                        } else {
                            Color::srgba(0.2, 0.17, 0.14, 0.6)
                        };

                        panel
                            .spawn((
                                BuildingRow { index: i },
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(36.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                            ))
                            .with_children(|row| {
                                let name_color = if entry.available {
                                    Color::srgb(0.5, 1.0, 0.5)
                                } else {
                                    Color::srgb(0.6, 0.5, 0.4)
                                };

                                row.spawn((
                                    BuildingRowText { index: i },
                                    Text::new(row_text),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(name_color),
                                ));

                                let cost_color = if entry.available {
                                    Color::srgb(0.8, 0.8, 0.6)
                                } else {
                                    Color::srgb(0.5, 0.4, 0.4)
                                };

                                row.spawn((
                                    BuildingRowCost { index: i },
                                    Text::new(entry.status_line.clone()),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(cost_color),
                                ));
                            });
                    }

                    // Hint
                    panel.spawn((
                        Text::new("Up/Down: Select | Enter: Upgrade | Esc: Close"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

pub fn despawn_building_upgrade_menu(
    mut commands: Commands,
    query: Query<Entity, With<BuildingUpgradeMenuRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BuildingUpgradeMenuState>();
}

// ═══════════════════════════════════════════════════════════════════════
// NAVIGATION + INPUT
// ═══════════════════════════════════════════════════════════════════════

pub fn building_upgrade_navigation(
    action: Res<MenuAction>,
    mut ui_state: Option<ResMut<BuildingUpgradeMenuState>>,
    player_state: Res<PlayerState>,
    mut upgrade_writer: EventWriter<BuildingUpgradeEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut ui_state) = ui_state else { return };
    let max = ui_state.entries.len();
    if max == 0 {
        return;
    }

    // Cancel → back to Playing
    if action.cancel {
        next_state.set(GameState::Playing);
        return;
    }

    // Cursor movement
    if action.move_down && ui_state.cursor < max - 1 {
        ui_state.cursor += 1;
    }
    if action.move_up && ui_state.cursor > 0 {
        ui_state.cursor -= 1;
    }

    // Confirm → attempt upgrade
    if action.activate {
        let entry = &ui_state.entries[ui_state.cursor];
        if !entry.available {
            if entry.to_tier.is_none() {
                ui_state.status_message = "Already at max tier!".to_string();
            } else {
                ui_state.status_message = "Upgrade not available right now.".to_string();
            }
            ui_state.status_timer = 2.0;
            return;
        }

        let to_tier = entry.to_tier.unwrap();

        // Check gold
        if player_state.gold < entry.cost_gold {
            ui_state.status_message = format!(
                "Not enough gold! Need {}g, have {}g.",
                entry.cost_gold, player_state.gold
            );
            ui_state.status_timer = 2.5;
            return;
        }

        // Send the upgrade event — the economy/buildings handler does the
        // actual gold/material deduction and validation.
        let cost_materials_ids: Vec<(ItemId, u8)> = entry
            .cost_materials
            .iter()
            .map(|&(name, qty)| (name.to_string(), qty))
            .collect();

        upgrade_writer.send(BuildingUpgradeEvent {
            building: entry.building,
            from_tier: entry.from_tier,
            to_tier,
            cost_gold: entry.cost_gold,
            cost_materials: cost_materials_ids,
        });

        ui_state.status_message = format!("Upgrade requested for {}!", entry.label);
        ui_state.status_timer = 2.0;

        // Return to gameplay after sending the event
        next_state.set(GameState::Playing);
    }
}

/// Updates row highlight colours to track the cursor position.
pub fn update_building_upgrade_display(
    ui_state: Option<Res<BuildingUpgradeMenuState>>,
    mut row_query: Query<(&BuildingRow, &mut BackgroundColor)>,
    mut status_query: Query<
        &mut Text,
        (
            With<BuildingUpgradeStatusText>,
            Without<BuildingRowText>,
            Without<BuildingRowCost>,
        ),
    >,
) {
    let Some(ui_state) = ui_state else { return };

    // Cursor highlight
    for (row, mut bg) in &mut row_query {
        if row.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.9));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6));
        }
    }

    // Status text
    for mut text in &mut status_query {
        **text = ui_state.status_message.clone();
    }
}

/// Tick down the status message timer.
pub fn building_upgrade_status_timer(
    time: Res<Time>,
    mut ui_state: Option<ResMut<BuildingUpgradeMenuState>>,
) {
    let Some(ref mut ui_state) = ui_state else { return };
    if ui_state.status_timer > 0.0 {
        ui_state.status_timer -= time.delta_secs();
        if ui_state.status_timer <= 0.0 {
            ui_state.status_message.clear();
        }
    }
}
