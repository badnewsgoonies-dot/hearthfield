use bevy::prelude::*;
use crate::shared::{GameState, FishRegistry, FishDef, TILE_SIZE};
use crate::fishing::FishingAtlas;
use crate::data::fish::fish_description;

#[derive(Component)]
pub struct FishEncyclopediaRoot;

pub fn spawn_fish_encyclopedia_screen(
    mut commands: Commands,
    fish_registry: Res<FishRegistry>,
    fishing_atlas: Res<FishingAtlas>,
) {
    let sprite_size = TILE_SIZE as f32;

    commands
        .spawn((
            FishEncyclopediaRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.95, 0.90, 0.80)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Fishing Encyclopedia"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.28, 0.20, 0.12)),
            ));

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::FlexStart,
                        column_gap: Val::Px(16.0),
                        row_gap: Val::Px(16.0),
                        padding: UiRect::right(Val::Px(8.0)),
                        overflow: Overflow {
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Scroll,
                        },
                        ..default()
                    },
                ))
                .with_children(|grid| {
                    for fish in fish_registry.fish.values() {
                        spawn_fish_card(grid, fish, &fishing_atlas, sprite_size);
                    }
                });
        });
}

pub fn despawn_fish_encyclopedia_screen(
    mut commands: Commands,
    roots: Query<Entity, With<FishEncyclopediaRoot>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_fish_encyclopedia_visuals() {}

pub fn fish_encyclopedia_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let escape_pressed = keyboard.just_pressed(KeyCode::Escape);
    let gamepad_back_pressed = gamepads.iter().any(|gp| {
        gp.just_pressed(GamepadButton::East)
    });

    if escape_pressed || gamepad_back_pressed {
        next_state.set(GameState::MainMenu);
    }
}

fn spawn_fish_card(
    parent: &mut ChildBuilder,
    fish: &FishDef,
    fishing_atlas: &FishingAtlas,
    sprite_size: f32,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(48.0),
                min_width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(14.0)),
                column_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                ..Node::DEFAULT
            },
            BackgroundColor(Color::srgb(0.98, 0.95, 0.88)),
            BorderColor(Color::srgb(0.74, 0.66, 0.52)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|card| {
            card.spawn((
                ImageNode::from_atlas_image(
                    fishing_atlas.image.clone(),
                    TextureAtlas {
                        layout: fishing_atlas.layout.clone(),
                        index: fish.sprite_index as usize,
                    },
                ),
                Node {
                    width: Val::Px(sprite_size),
                    height: Val::Px(sprite_size),
                    ..default()
                },
            ));

            card.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    flex_grow: 1.0,
                    ..default()
                },
            ))
            .with_children(|text_column| {
                text_column.spawn((
                    Text::new(fish.name.clone()),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.22, 0.15, 0.10)),
                ));

                text_column.spawn((
                    Text::new(fish_description(&fish.id)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.42, 0.40, 0.38)),
                ));
            });
        });
}
