//! Pause overlay — Resume / Save / Quit to Menu.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct PauseButton(PauseAction);

#[derive(Clone, Copy)]
enum PauseAction {
    Resume,
    Save,
    QuitToMenu,
}

pub fn spawn_pause_menu(mut commands: Commands, font: Res<UiFontHandle>) {
    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 28.0,
        ..default()
    };
    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 18.0,
        ..default()
    };

    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                title_style,
                TextColor(Color::WHITE),
            ));

            for (label, action) in [
                ("Resume", PauseAction::Resume),
                ("Save Game", PauseAction::Save),
                ("Quit to Menu", PauseAction::QuitToMenu),
            ] {
                parent
                    .spawn((
                        PauseButton(action),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(28.0), Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            btn_style.clone(),
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

pub fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_pause_input(
    input: Res<PlayerInput>,
    interaction_q: Query<(&Interaction, &PauseButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_evt: EventWriter<SaveRequestEvent>,
) {
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            match btn.0 {
                PauseAction::Resume => next_state.set(GameState::Playing),
                PauseAction::Save => {
                    save_evt.send(SaveRequestEvent { slot: 0 });
                }
                PauseAction::QuitToMenu => next_state.set(GameState::MainMenu),
            }
        }
    }

    if input.cancel || input.pause {
        next_state.set(GameState::Playing);
    }
}
