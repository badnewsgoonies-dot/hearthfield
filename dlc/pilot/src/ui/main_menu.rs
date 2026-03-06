//! Title screen — SKYWARDEN with New Game / Load / Quit buttons.

use crate::shared::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MenuButton(MenuAction);

#[derive(Component)]
pub struct MenuButtonLabel;

#[derive(Clone, Copy)]
enum MenuAction {
    NewGame,
    Load,
    Quit,
}

fn button_idle_color(action: MenuAction) -> Color {
    match action {
        MenuAction::NewGame => Color::srgb(0.16, 0.28, 0.24),
        MenuAction::Load => Color::srgb(0.16, 0.20, 0.30),
        MenuAction::Quit => Color::srgb(0.30, 0.15, 0.18),
    }
}

fn button_hover_color(action: MenuAction) -> Color {
    match action {
        MenuAction::NewGame => Color::srgb(0.28, 0.46, 0.38),
        MenuAction::Load => Color::srgb(0.28, 0.34, 0.50),
        MenuAction::Quit => Color::srgb(0.46, 0.24, 0.28),
    }
}

fn button_pressed_color(action: MenuAction) -> Color {
    match action {
        MenuAction::NewGame => Color::srgb(0.82, 0.72, 0.30),
        MenuAction::Load => Color::srgb(0.78, 0.67, 0.28),
        MenuAction::Quit => Color::srgb(0.74, 0.53, 0.26),
    }
}

pub fn spawn_main_menu(mut commands: Commands, font: Res<UiFontHandle>) {
    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 52.0,
        ..default()
    };
    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 22.0,
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
            BackgroundColor(Color::srgb(0.04, 0.05, 0.10)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SKYWARDEN"),
                title_style,
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            parent.spawn((
                Text::new("A Hearthfield DLC"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.70, 0.92)),
            ));

            parent.spawn((
                Text::new("Bush flying, cargo runs, crew, and weather."),
                TextFont {
                    font: font.0.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.74, 0.78)),
            ));

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

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
                            width: Val::Px(240.0),
                            height: Val::Px(52.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(32.0), Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(button_idle_color(action)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            MenuButtonLabel,
                            Text::new(label),
                            btn_style.clone(),
                            TextColor(Color::WHITE),
                        ));
                    });
            }

            parent.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });

            parent.spawn((
                Text::new("Click a button or press Enter to launch a new flight career."),
                TextFont {
                    font: font.0.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.68, 0.70)),
            ));
        });
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_main_menu_input(
    input: Res<PlayerInput>,
    mut interaction_q: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor, &Children),
        Changed<Interaction>,
    >,
    mut text_q: Query<&mut TextColor, With<MenuButtonLabel>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, btn, mut bg, children) in &mut interaction_q {
        let (bg_color, text_color) = match *interaction {
            Interaction::Pressed => (button_pressed_color(btn.0), Color::srgb(0.10, 0.10, 0.14)),
            Interaction::Hovered => (button_hover_color(btn.0), Color::WHITE),
            Interaction::None => (button_idle_color(btn.0), Color::WHITE),
        };
        bg.0 = bg_color;
        for child in children.iter() {
            if let Ok(mut text_color_comp) = text_q.get_mut(*child) {
                text_color_comp.0 = text_color;
            }
        }

        if *interaction == Interaction::Pressed {
            match btn.0 {
                MenuAction::NewGame => next_state.set(GameState::Playing),
                MenuAction::Load => {
                    next_state.set(GameState::LoadGame);
                }
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
