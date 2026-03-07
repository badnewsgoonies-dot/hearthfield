//! Bottom-screen dialogue box — speaker name + text.

use crate::shared::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct DialogueBoxRoot;

#[derive(Component)]
pub struct SpeakerText;

#[derive(Component)]
pub struct BodyText;

pub fn spawn_dialogue_box(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    dialogue: Res<DialogueState>,
) {
    let speaker_style = TextFont {
        font: font.0.clone(),
        font_size: 16.0,
        ..default()
    };
    let body_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    let speaker_name = dialogue.speaker_name.clone();
    let body = dialogue
        .lines
        .get(dialogue.current_line)
        .map(|l| l.text.clone())
        .unwrap_or_default();

    commands
        .spawn((
            DialogueBoxRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                SpeakerText,
                Text::new(speaker_name),
                speaker_style,
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));
            parent.spawn((
                BodyText,
                Text::new(body),
                body_style,
                TextColor(Color::WHITE),
            ));
        });
}

pub fn despawn_dialogue_box(mut commands: Commands, query: Query<Entity, With<DialogueBoxRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_dialogue_box(
    dialogue: Res<DialogueState>,
    root_q: Query<Entity, With<DialogueBoxRoot>>,
    children_q: Query<&Children>,
    mut speaker_q: Query<&mut Text, (With<SpeakerText>, Without<BodyText>)>,
    mut body_q: Query<&mut Text, (With<BodyText>, Without<SpeakerText>)>,
) {
    if !dialogue.is_changed() {
        return;
    }
    let Ok(root) = root_q.get_single() else {
        return;
    };
    let Ok(children) = children_q.get(root) else {
        return;
    };

    for child in children.iter() {
        if let Ok(mut text) = speaker_q.get_mut(*child) {
            **text = dialogue.speaker_name.clone();
        }
        if let Ok(mut text) = body_q.get_mut(*child) {
            **text = dialogue
                .lines
                .get(dialogue.current_line)
                .map(|l| l.text.clone())
                .unwrap_or_default();
        }
    }
}
