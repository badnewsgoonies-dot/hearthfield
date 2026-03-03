//! Title screen — SKYWARDEN with New Game / Load / Quit buttons.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MenuButton(MenuAction);

#[derive(Clone, Copy)]
enum MenuAction {
    NewGame,
    Load,
    Quit,
}

pub fn spawn_main_menu(mut commands: Commands, font: Res<UiFontHandle>) {
    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 48.0,
        ..default()
    };
    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 20.0,
        ..default()
    };

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.12)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SKYWARDEN"),
                title_style,
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            for (label, action) in [
                ("New Game", MenuAction::NewGame),
                ("Load Game", MenuAction::Load),
                ("Quit", MenuAction::Quit),
            ] {
                parent
                    .spawn((
                        MenuButton(action),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(32.0), Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
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

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_main_menu_input(
    input: Res<PlayerInput>,
    interaction_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            match btn.0 {
                MenuAction::NewGame => next_state.set(GameState::Playing),
                MenuAction::Load => { /* TODO: open load screen */ }
                MenuAction::Quit => {
                    exit.send(AppExit::Success);
                }
            }
        }
    }

    if input.menu_confirm {
        next_state.set(GameState::Playing);
    }
}
