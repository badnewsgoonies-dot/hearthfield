//! Load Game screen — select a save slot and load it.

use crate::shared::*;
use bevy::prelude::*;

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct LoadScreenRoot;

#[derive(Component)]
pub struct LoadSlotButton {
    pub slot: usize,
}

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_load_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    save_slots: Res<SaveSlots>,
) {
    let root = commands
        .spawn((
            LoadScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.03, 0.08, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new("LOAD GAME"),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.9, 0.5)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 16.0,
        ..default()
    };

    // Save slot buttons
    for (i, slot_opt) in save_slots.slots.iter().enumerate() {
        let label = if let Some(info) = slot_opt {
            format!(
                "Slot {} — {} (Rank: {}, Day {}, Year {})",
                i + 1,
                info.pilot_name,
                info.rank.display_name(),
                info.day,
                info.year,
            )
        } else {
            format!("Slot {} — Empty —", i + 1)
        };

        let is_occupied = slot_opt.is_some();

        let btn = commands
            .spawn((
                LoadSlotButton { slot: i },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(32.0), Val::Px(10.0)),
                    min_width: Val::Px(400.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(if is_occupied {
                    Color::srgb(0.15, 0.2, 0.3)
                } else {
                    Color::srgb(0.1, 0.1, 0.12)
                }),
            ))
            .id();

        let text = commands
            .spawn((
                Text::new(label),
                btn_style.clone(),
                TextColor(if is_occupied {
                    Color::WHITE
                } else {
                    Color::srgb(0.4, 0.4, 0.4)
                }),
            ))
            .id();

        commands.entity(btn).add_child(text);
        commands.entity(root).add_child(btn);
    }

    // Back hint
    let hint = commands
        .spawn((
            Text::new("[Esc] Back to Main Menu"),
            TextFont {
                font: font.0.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect::top(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(hint);
}

pub fn despawn_load_screen(mut commands: Commands, query: Query<Entity, With<LoadScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Input ───────────────────────────────────────────────────────────────

pub fn handle_load_input(
    input: Res<PlayerInput>,
    interaction_q: Query<(&Interaction, &LoadSlotButton), Changed<Interaction>>,
    save_slots: Res<SaveSlots>,
    mut load_events: EventWriter<LoadRequestEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Esc → back to main menu
    if input.cancel || input.menu_cancel {
        next_state.set(GameState::MainMenu);
        return;
    }

    // Click a slot button
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed && save_slots.slots[btn.slot].is_some() {
            load_events.send(LoadRequestEvent { slot: btn.slot });
            next_state.set(GameState::Playing);
        }
    }
}
