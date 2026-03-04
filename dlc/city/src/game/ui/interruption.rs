use bevy::prelude::*;

use crate::game::events::{PanicResponseEvent, ResolveCalmlyEvent};
use crate::game::resources::{ActiveInterruptionContext, InterruptionKind, PlayerMindState};

use super::InterruptionPopupRoot;

#[derive(Component)]
pub(crate) struct CalmButton;

#[derive(Component)]
pub(crate) struct PanicButton;

#[derive(Component)]
pub(crate) struct InterruptionTitle;

#[derive(Component)]
pub(crate) struct InterruptionBody;

pub fn spawn_interruption_popup(mut commands: Commands) {
    commands
        .spawn((
            InterruptionPopupRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            GlobalZIndex(5),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(340.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.08, 0.08)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        InterruptionTitle,
                        Text::new("INTERRUPTION!"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.25, 0.25)),
                    ));

                    popup.spawn((
                        InterruptionBody,
                        Text::new("A coworker needs your help urgently.\nHow do you respond?"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    popup.spawn(Node {
                        height: Val::Px(8.0),
                        ..default()
                    });

                    // Calm button
                    popup
                        .spawn((
                            CalmButton,
                            Button,
                            Node {
                                width: Val::Px(240.0),
                                height: Val::Px(44.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.45, 0.3)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Stay Calm (1)"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Panic button
                    popup
                        .spawn((
                            PanicButton,
                            Button,
                            Node {
                                width: Val::Px(240.0),
                                height: Val::Px(44.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.55, 0.15, 0.15)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Panic! (2)"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

pub fn despawn_interruption_popup(
    mut commands: Commands,
    query: Query<Entity, With<InterruptionPopupRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn interruption_kind_label(kind: InterruptionKind) -> &'static str {
    match kind {
        InterruptionKind::ManagerRequest => "MANAGER REQUEST",
        InterruptionKind::EmergencyMeeting => "EMERGENCY MEETING",
        InterruptionKind::SystemOutage => "SYSTEM OUTAGE",
        InterruptionKind::CoworkerHelp => "COWORKER HELP",
        InterruptionKind::PrinterJam => "PRINTER JAM",
        InterruptionKind::FireDrill => "FIRE DRILL",
        InterruptionKind::BossVisit => "BOSS VISIT",
        InterruptionKind::FreeLunch => "FREE LUNCH",
    }
}

fn interruption_kind_color(kind: InterruptionKind) -> Color {
    match kind {
        InterruptionKind::ManagerRequest => Color::srgb(1.0, 0.6, 0.15),
        InterruptionKind::EmergencyMeeting => Color::srgb(1.0, 0.85, 0.2),
        InterruptionKind::SystemOutage => Color::srgb(1.0, 0.2, 0.2),
        InterruptionKind::CoworkerHelp => Color::srgb(0.4, 0.8, 1.0),
        InterruptionKind::PrinterJam => Color::srgb(1.0, 0.45, 0.2),
        InterruptionKind::FireDrill => Color::srgb(1.0, 0.15, 0.15),
        InterruptionKind::BossVisit => Color::srgb(0.95, 0.75, 0.2),
        InterruptionKind::FreeLunch => Color::srgb(0.35, 0.9, 0.35),
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_interruption_visibility(
    mind: Res<PlayerMindState>,
    context: Res<ActiveInterruptionContext>,
    mut query: Query<&mut Visibility, With<InterruptionPopupRoot>>,
    mut title_query: Query<(&mut Text, &mut TextColor), (With<InterruptionTitle>, Without<InterruptionBody>)>,
    mut body_query: Query<(&mut Text, &mut TextColor), (With<InterruptionBody>, Without<InterruptionTitle>)>,
    calm_query: Query<&Interaction, (Changed<Interaction>, With<CalmButton>)>,
    panic_query: Query<&Interaction, (Changed<Interaction>, With<PanicButton>)>,
    mut calm_writer: EventWriter<ResolveCalmlyEvent>,
    mut panic_writer: EventWriter<PanicResponseEvent>,
) {
    // Show/hide based on pending interruptions
    for mut vis in &mut query {
        *vis = if mind.pending_interruptions > 0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    // Update title and body from ActiveInterruptionContext
    if mind.pending_interruptions > 0 {
        if let Some(kind) = context.kind {
            if let Ok((mut title_text, mut title_color)) = title_query.get_single_mut() {
                **title_text = interruption_kind_label(kind).to_string();
                title_color.0 = interruption_kind_color(kind);
            }
            if let Ok((mut body_text, _body_color)) = body_query.get_single_mut() {
                let mut display = context.description.clone();
                if let Some(ref name) = context.coworker_name {
                    display.push_str(&format!("\n\nInvolved: {name}"));
                }
                display.push_str("\n\nHow do you respond?");
                **body_text = display;
            }
        }
    }

    // Handle button clicks
    for interaction in &calm_query {
        if *interaction == Interaction::Pressed {
            calm_writer.send(ResolveCalmlyEvent);
        }
    }

    for interaction in &panic_query {
        if *interaction == Interaction::Pressed {
            panic_writer.send(PanicResponseEvent);
        }
    }
}
