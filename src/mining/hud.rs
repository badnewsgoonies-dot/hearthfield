//! Minimal mine HUD overlay showing floor number and elevator info.
//!
//! The main HUD is managed by the UI domain. This module only spawns
//! a small "Floor N" text label that exists while in the mine, plus
//! the elevator selection prompt when ElevatorUiOpen is true.

use bevy::prelude::*;

use crate::shared::*;
use super::components::*;

/// Marker for mine HUD entities so we can despawn them.
#[derive(Component, Debug)]
pub struct MineHudEntity;

/// Marker for the floor label text.
#[derive(Component, Debug)]
pub struct FloorLabel;

/// Marker for the elevator prompt text.
#[derive(Component, Debug)]
pub struct ElevatorPrompt;

/// System: spawn mine HUD when entering the mine.
pub fn spawn_mine_hud(
    mut commands: Commands,
    in_mine: Res<InMine>,
    existing: Query<Entity, With<MineHudEntity>>,
) {
    // Only spawn if we're in the mine and no HUD exists yet
    if !in_mine.0 || !existing.is_empty() {
        return;
    }

    // Floor label (top-left corner)
    commands.spawn((
        Text::new("Floor 1"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        MineHudEntity,
        FloorLabel,
    ));
}

/// System: update the floor label text.
pub fn update_floor_label(
    active_floor: Res<ActiveFloor>,
    mut labels: Query<&mut Text, With<FloorLabel>>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    for mut text in labels.iter_mut() {
        **text = format!("Floor {}", active_floor.floor);
    }
}

/// System: show elevator prompt when ElevatorUiOpen is true.
pub fn show_elevator_prompt(
    mut commands: Commands,
    elevator_ui: Res<ElevatorUiOpen>,
    mine_state: Res<MineState>,
    existing_prompts: Query<Entity, With<ElevatorPrompt>>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    if elevator_ui.0 && existing_prompts.is_empty() {
        // Build elevator text
        let mut lines = String::from("Choose floor:\n[1] Floor 1\n");
        for (i, floor) in mine_state.elevator_floors.iter().enumerate() {
            lines.push_str(&format!("[{}] Floor {}\n", i + 2, floor));
        }
        lines.push_str("[Esc] Cancel");

        commands.spawn((
            Text::new(lines),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.85, 0.6)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(50.0),
                ..default()
            },
            MineHudEntity,
            ElevatorPrompt,
        ));
    } else if !elevator_ui.0 {
        // Remove elevator prompt
        for entity in existing_prompts.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// System: despawn mine HUD when leaving the mine.
pub fn despawn_mine_hud(
    mut commands: Commands,
    in_mine: Res<InMine>,
    entities: Query<Entity, With<MineHudEntity>>,
) {
    if !in_mine.0 {
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
    }
}
