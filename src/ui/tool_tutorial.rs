//! Tool tutorial overlay — shown during Mayor Rex's intro dialogue.
//!
//! As the player advances through the tool-explanation dialogue lines
//! (lines 1-6 of the tutorial dialogue), this module spawns a
//! semi-transparent panel showing the relevant tool sprite enlarged.
//! The panel is despawned when dialogue ends.
//!
//! Design notes:
//! - Runs as an Update system during `GameState::Dialogue`.
//! - Detects the tutorial dialogue by `npc_id == "mayor_rex"` AND the
//!   `CutsceneFlags` flag `"tool_tutorial_active"` being set.
//! - Swaps the tool sprite whenever `DialogueUiState::current_line` advances.
//! - Completely self-contained: no shared-type modifications required.

use super::dialogue_box::DialogueUiState;
use super::UiFontHandle;
use crate::player::tool_anim::{tool_sprite_handle, ProceduralToolSprites};
use crate::shared::*;
use crate::ui::cutscene_runner::CutsceneFlags;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// MARKER COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Root entity of the tool tutorial overlay panel.
#[derive(Component)]
pub struct ToolTutorialOverlay;

/// The enlarged tool sprite inside the panel.
#[derive(Component)]
pub struct ToolTutorialSprite;

/// The tool-name label.
#[derive(Component)]
pub struct ToolTutorialName;

/// The tool description text.
#[derive(Component)]
pub struct ToolTutorialDescription;

/// The page indicator at the bottom ("Tool 1/6").
#[derive(Component)]
pub struct ToolTutorialPageIndicator;

// ═══════════════════════════════════════════════════════════════════════
// TUTORIAL TOOL TABLE
// Maps dialogue line index → (ToolKind, display name, description).
// Lines 0 and 7 are bookend lines (no tool shown).
// ═══════════════════════════════════════════════════════════════════════

struct ToolEntry {
    line_index: usize,
    tool: ToolKind,
    name: &'static str,
    description: &'static str,
    page: usize, // 1-based page number for "Tool N/6"
}

const TOOL_ENTRIES: &[ToolEntry] = &[
    ToolEntry {
        line_index: 1,
        tool: ToolKind::Hoe,
        name: "HOE",
        description: "Tills soil for planting crops.\nStand on dirt and press SPACE.\nUpgrade at the Blacksmith for wider area.",
        page: 1,
    },
    ToolEntry {
        line_index: 2,
        tool: ToolKind::WateringCan,
        name: "WATERING CAN",
        description: "Waters your crops to help them grow.\nAim at tilled soil and press SPACE.\nCrops need water daily to grow!",
        page: 2,
    },
    ToolEntry {
        line_index: 3,
        tool: ToolKind::Axe,
        name: "AXE",
        description: "Chops trees for wood.\nStand next to a tree and press SPACE.\nUpgrade for faster chopping.",
        page: 3,
    },
    ToolEntry {
        line_index: 4,
        tool: ToolKind::Pickaxe,
        name: "PICKAXE",
        description: "Breaks rocks in the mine.\nStand next to rocks and press SPACE.\nFind ores and gems deeper down.",
        page: 4,
    },
    ToolEntry {
        line_index: 5,
        tool: ToolKind::Scythe,
        name: "SCYTHE",
        description: "Cuts grass and harvests crops.\nSwing near plants and press SPACE.\nFast and costs little energy.",
        page: 5,
    },
    ToolEntry {
        line_index: 6,
        tool: ToolKind::FishingRod,
        name: "FISHING ROD",
        description: "Cast into water to catch fish.\nStand by water and press SPACE.\nReel in when a fish bites!",
        page: 6,
    },
];

const TOTAL_TOOLS: usize = 6;

// ═══════════════════════════════════════════════════════════════════════
// HELPER — find the tool entry for the current dialogue line
// ═══════════════════════════════════════════════════════════════════════

fn entry_for_line(line: usize) -> Option<&'static ToolEntry> {
    TOOL_ENTRIES.iter().find(|e| e.line_index == line)
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM — manage the overlay lifecycle during Dialogue state
// ═══════════════════════════════════════════════════════════════════════

/// Called every frame while in `GameState::Dialogue`.
///
/// Logic:
/// 1. Check that the tutorial flag is active and we are in the tutorial NPC dialogue.
/// 2. If there is a tool entry for the current line, ensure the overlay exists and
///    shows the correct tool. If the line changed, update text/sprite in place.
/// 3. If there is no tool entry for the current line (intro/outro lines), despawn the overlay.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_tool_tutorial_overlay(
    mut commands: Commands,
    dialogue_state: Option<Res<DialogueUiState>>,
    cutscene_flags: Res<CutsceneFlags>,
    tool_sprites: Res<ProceduralToolSprites>,
    font_handle: Res<UiFontHandle>,
    overlay_query: Query<Entity, With<ToolTutorialOverlay>>,
    mut sprite_query: Query<&mut ImageNode, With<ToolTutorialSprite>>,
    mut name_query: Query<
        &mut Text,
        (
            With<ToolTutorialName>,
            Without<ToolTutorialDescription>,
            Without<ToolTutorialPageIndicator>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<ToolTutorialDescription>,
            Without<ToolTutorialName>,
            Without<ToolTutorialPageIndicator>,
        ),
    >,
    mut page_query: Query<
        &mut Text,
        (
            With<ToolTutorialPageIndicator>,
            Without<ToolTutorialName>,
            Without<ToolTutorialDescription>,
        ),
    >,
) {
    // Only show the overlay when the tutorial flag is active.
    let tutorial_active = cutscene_flags
        .flags
        .get("tool_tutorial_active")
        .copied()
        .unwrap_or(false);

    if !tutorial_active {
        // Tutorial not active — despawn overlay if it exists and return.
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    let Some(dlg) = dialogue_state else {
        // Dialogue ended — despawn overlay.
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    };

    // Only show during the Mayor Rex tutorial dialogue.
    if dlg.npc_id != "mayor_rex" {
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    let current_line = dlg.current_line;

    // Check if the current line maps to a tool.
    let Some(entry) = entry_for_line(current_line) else {
        // No tool for this line (intro/outro) — despawn overlay if present.
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    };

    // Get the tool image handle.
    let Some(tool_handle) = tool_sprite_handle(&tool_sprites, entry.tool) else {
        return;
    };

    if overlay_query.is_empty() {
        // Spawn the overlay fresh.
        spawn_tutorial_overlay(
            &mut commands,
            &font_handle,
            tool_handle,
            entry.name,
            entry.description,
            entry.page,
        );
    } else {
        // Update existing overlay in-place.
        for mut image_node in sprite_query.iter_mut() {
            image_node.image = tool_handle.clone();
        }
        for mut text in name_query.iter_mut() {
            **text = entry.name.to_string();
        }
        for mut text in desc_query.iter_mut() {
            **text = entry.description.to_string();
        }
        for mut text in page_query.iter_mut() {
            **text = format!("Tool {}/{}", entry.page, TOTAL_TOOLS);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWN helper
// ═══════════════════════════════════════════════════════════════════════

fn spawn_tutorial_overlay(
    commands: &mut Commands,
    font_handle: &UiFontHandle,
    tool_image: Handle<Image>,
    tool_name: &'static str,
    description: &'static str,
    page: usize,
) {
    let font = font_handle.0.clone();

    // Backdrop — full-screen, semi-transparent dark, positioned to the LEFT
    // of the dialogue box so it doesn't overlap the NPC portrait area.
    // Use GlobalZIndex(90) so it renders above world but below dialogue (z=100).
    commands
        .spawn((
            ToolTutorialOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                // Align toward the top-center, above the dialogue box.
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                // Push the panel up so it sits above the dialogue bar.
                bottom: Val::Px(185.0),
                ..default()
            },
            // Don't intercept pointer events so dialogue input still works.
            PickingBehavior::IGNORE,
            GlobalZIndex(90),
        ))
        .with_children(|root| {
            // Center panel — white/cream card.
            root.spawn((
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(180.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(12.0)),
                    column_gap: Val::Px(14.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.98, 0.97, 0.93, 0.95)),
                BorderColor(Color::srgb(0.55, 0.45, 0.25)),
                BorderRadius::all(Val::Px(6.0)),
                PickingBehavior::IGNORE,
            ))
            .with_children(|panel| {
                // LEFT — enlarged tool sprite (64x64 logical pixels displayed).
                panel.spawn((
                    ToolTutorialSprite,
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode {
                        image: tool_image,
                        ..default()
                    },
                    PickingBehavior::IGNORE,
                ));

                // RIGHT — text column.
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            justify_content: JustifyContent::SpaceBetween,
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        PickingBehavior::IGNORE,
                    ))
                    .with_children(|text_col| {
                        // Tool name (bold via larger font size).
                        text_col.spawn((
                            ToolTutorialName,
                            Text::new(tool_name),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.25, 0.15, 0.05)),
                            PickingBehavior::IGNORE,
                        ));

                        // Description (multi-line, smaller).
                        text_col.spawn((
                            ToolTutorialDescription,
                            Text::new(description),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.2, 0.2, 0.2)),
                            PickingBehavior::IGNORE,
                        ));

                        // Page indicator.
                        text_col.spawn((
                            ToolTutorialPageIndicator,
                            Text::new(format!("Tool {}/{}", page, TOTAL_TOOLS)),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            PickingBehavior::IGNORE,
                        ));
                    });
            });
        });
}

// ═══════════════════════════════════════════════════════════════════════
// CLEANUP — despawn on exit from Dialogue state
// ═══════════════════════════════════════════════════════════════════════

pub fn despawn_tool_tutorial_overlay(
    mut commands: Commands,
    overlay_query: Query<Entity, With<ToolTutorialOverlay>>,
) {
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_entries_cover_all_six_tools() {
        assert_eq!(TOOL_ENTRIES.len(), 6);
    }

    #[test]
    fn tool_entry_line_indices_are_1_through_6() {
        let lines: Vec<usize> = TOOL_ENTRIES.iter().map(|e| e.line_index).collect();
        assert_eq!(lines, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn entry_for_line_returns_correct_tool() {
        let e = entry_for_line(1).expect("line 1 should map to hoe");
        assert_eq!(e.name, "HOE");
        let e2 = entry_for_line(6).expect("line 6 should map to fishing rod");
        assert_eq!(e2.name, "FISHING ROD");
    }

    #[test]
    fn entry_for_line_returns_none_for_bookend_lines() {
        assert!(entry_for_line(0).is_none());
        assert!(entry_for_line(7).is_none());
        assert!(entry_for_line(99).is_none());
    }

    #[test]
    fn all_tool_entries_have_non_empty_descriptions() {
        for e in TOOL_ENTRIES {
            assert!(
                !e.description.is_empty(),
                "{} has empty description",
                e.name
            );
        }
    }

    #[test]
    fn page_numbers_are_sequential_1_to_6() {
        for (i, e) in TOOL_ENTRIES.iter().enumerate() {
            assert_eq!(e.page, i + 1, "{} has wrong page number", e.name);
        }
    }
}
