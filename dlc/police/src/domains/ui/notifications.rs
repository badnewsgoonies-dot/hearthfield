use bevy::prelude::*;
use bevy::time::TimerMode;

use crate::domains::cases::case_definition;
use crate::shared::{
    CaseFailedEvent, CaseSolvedEvent, DispatchCallEvent, EvidenceCollectedEvent, EvidenceLocker,
    EvidenceProcessedEvent, PatrolState, PlayMusicEvent, PlaySfxEvent, PromotionEvent,
    SkillPointSpentEvent, SkillTree, ToastEvent, UpdatePhase, XpGainedEvent,
};

const TOAST_MAX_WIDTH: f32 = 460.0;
const TOAST_MIN_DURATION_SECS: f32 = 0.25;
const DISPATCH_DURATION_SECS: f32 = 6.0;
const DISPATCH_FLASH_SECS: f32 = 0.35;

#[derive(Resource)]
struct DispatchBannerState {
    message: Option<String>,
    display_timer: Timer,
    flash_timer: Timer,
    flash_on: bool,
}

impl Default for DispatchBannerState {
    fn default() -> Self {
        Self {
            message: None,
            display_timer: Timer::from_seconds(DISPATCH_DURATION_SECS, TimerMode::Once),
            flash_timer: Timer::from_seconds(DISPATCH_FLASH_SECS, TimerMode::Repeating),
            flash_on: true,
        }
    }
}

#[derive(Component)]
struct FeedbackOverlayRoot;

#[derive(Component)]
struct ToastStack;

#[derive(Component)]
struct ToastLifetime(Timer);

#[derive(Component)]
struct ToastCard;

#[derive(Component)]
struct DispatchBanner;

#[derive(Component)]
struct DispatchHeaderText;

#[derive(Component)]
struct DispatchBodyText;

type DispatchHeaderQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut TextColor),
    (With<DispatchHeaderText>, Without<DispatchBodyText>),
>;

type DispatchBodyQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<DispatchBodyText>, Without<DispatchHeaderText>)>;

pub(super) fn build_notifications(app: &mut App) {
    app.init_resource::<DispatchBannerState>()
        .init_resource::<PatrolState>()
        .add_event::<ToastEvent>()
        .add_event::<DispatchCallEvent>()
        .add_event::<EvidenceProcessedEvent>()
        .add_event::<PlaySfxEvent>()
        .add_event::<PlayMusicEvent>()
        .add_event::<CaseSolvedEvent>()
        .add_event::<CaseFailedEvent>()
        .add_event::<EvidenceCollectedEvent>()
        .add_event::<PromotionEvent>()
        .add_event::<SkillPointSpentEvent>()
        .add_event::<XpGainedEvent>()
        .add_systems(Startup, spawn_feedback_overlay)
        .add_systems(
            Update,
            (
                emit_processed_evidence_toasts,
                emit_case_solved_toasts,
                emit_case_failed_toasts,
                emit_evidence_collected_toasts,
                emit_xp_toasts,
                emit_promotion_toasts,
                emit_skill_spent_toasts,
                capture_dispatch_events,
            )
                .in_set(UpdatePhase::Reactions),
        )
        .add_systems(
            Update,
            (
                spawn_toasts_from_events,
                tick_toast_lifetimes,
                update_dispatch_banner,
            )
                .in_set(UpdatePhase::Presentation),
        )
        .add_systems(Update, log_audio_requests.in_set(UpdatePhase::Reactions));
}

fn spawn_feedback_overlay(mut commands: Commands) {
    commands
        .spawn((
            FeedbackOverlayRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    DispatchBanner,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(72.0),
                        right: Val::Px(24.0),
                        width: Val::Px(340.0),
                        min_height: Val::Px(92.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.08, 0.13, 0.94)),
                    BorderColor(Color::srgb(0.86, 0.30, 0.30)),
                    Visibility::Hidden,
                ))
                .with_children(|banner| {
                    banner.spawn((
                        DispatchHeaderText,
                        Text::new("DISPATCH"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.82, 0.36)),
                    ));
                    banner.spawn((
                        DispatchBodyText,
                        Text::new(""),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent.spawn((
                ToastStack,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    right: Val::Percent(0.0),
                    bottom: Val::Px(28.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::horizontal(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
        });
}

fn emit_processed_evidence_toasts(
    mut processed_events: EventReader<EvidenceProcessedEvent>,
    locker: Res<EvidenceLocker>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in processed_events.read() {
        let evidence_name = locker
            .pieces
            .iter()
            .rev()
            .find(|piece| piece.id == event.evidence_id)
            .map(|piece| piece.name.clone())
            .unwrap_or_else(|| event.evidence_id.replace('_', " "));

        toast_events.send(ToastEvent {
            message: format!("Evidence analyzed: {evidence_name}"),
            duration_secs: 2.6,
        });
    }
}

fn emit_case_solved_toasts(
    mut case_solved_events: EventReader<CaseSolvedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in case_solved_events.read() {
        let case_name = case_definition(&event.case_id)
            .map(|case_def| case_def.name)
            .unwrap_or_else(|| event.case_id.clone());

        toast_events.send(ToastEvent {
            message: format!("Case closed: {case_name}"),
            duration_secs: 3.0,
        });
    }
}

fn emit_case_failed_toasts(
    mut case_failed_events: EventReader<CaseFailedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in case_failed_events.read() {
        toast_events.send(ToastEvent {
            message: format!("Case failed: {}", event.reason),
            duration_secs: 3.0,
        });
    }
}

fn emit_evidence_collected_toasts(
    mut evidence_events: EventReader<EvidenceCollectedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in evidence_events.read() {
        toast_events.send(ToastEvent {
            message: format!(
                "Evidence collected: {}",
                event.evidence_id.replace('_', " ")
            ),
            duration_secs: 2.2,
        });
    }
}

fn emit_xp_toasts(
    mut xp_events: EventReader<XpGainedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in xp_events.read() {
        toast_events.send(ToastEvent {
            message: format!(
                "XP earned: +{} ({})",
                event.amount,
                humanize_source(&event.source)
            ),
            duration_secs: 2.2,
        });
    }
}

fn emit_promotion_toasts(
    mut promotion_events: EventReader<PromotionEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in promotion_events.read() {
        toast_events.send(ToastEvent {
            message: format!("Promoted to {}", event.new_rank.display_name()),
            duration_secs: 3.2,
        });
    }
}

fn emit_skill_spent_toasts(
    mut skill_events: EventReader<SkillPointSpentEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in skill_events.read() {
        toast_events.send(ToastEvent {
            message: format!(
                "{} upgraded to tier {}",
                skill_tree_name(event.tree),
                event.new_level
            ),
            duration_secs: 2.4,
        });
    }
}

fn capture_dispatch_events(
    mut dispatch_events: EventReader<DispatchCallEvent>,
    mut dispatch_state: ResMut<DispatchBannerState>,
) {
    let Some(event) = dispatch_events.read().last() else {
        return;
    };

    dispatch_state.message = Some(event.call.description.clone());
    dispatch_state.display_timer = Timer::from_seconds(DISPATCH_DURATION_SECS, TimerMode::Once);
    dispatch_state.flash_timer = Timer::from_seconds(DISPATCH_FLASH_SECS, TimerMode::Repeating);
    dispatch_state.flash_on = true;
}

fn spawn_toasts_from_events(
    mut commands: Commands,
    toast_stack_query: Query<Entity, With<ToastStack>>,
    mut toast_events: EventReader<ToastEvent>,
) {
    let Ok(toast_stack) = toast_stack_query.get_single() else {
        return;
    };

    for event in toast_events.read() {
        commands.entity(toast_stack).with_children(|parent| {
            parent
                .spawn((
                    ToastCard,
                    ToastLifetime(Timer::from_seconds(
                        event.duration_secs.max(TOAST_MIN_DURATION_SECS),
                        TimerMode::Once,
                    )),
                    Node {
                        max_width: Val::Px(TOAST_MAX_WIDTH),
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.92)),
                    BorderColor(Color::srgb(0.86, 0.78, 0.42)),
                ))
                .with_children(|toast| {
                    toast.spawn((
                        Text::new(event.message.clone()),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
    }
}

fn tick_toast_lifetimes(
    time: Res<Time>,
    mut commands: Commands,
    mut toast_query: Query<(Entity, &mut ToastLifetime), With<ToastCard>>,
) {
    for (entity, mut lifetime) in &mut toast_query {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_dispatch_banner(
    time: Res<Time>,
    patrol_state: Res<PatrolState>,
    mut dispatch_state: ResMut<DispatchBannerState>,
    mut banner_query: Query<
        (&mut Visibility, &mut BackgroundColor, &mut BorderColor),
        With<DispatchBanner>,
    >,
    mut header_query: DispatchHeaderQuery,
    mut body_query: DispatchBodyQuery,
) {
    let Ok((mut visibility, mut background, mut border)) = banner_query.get_single_mut() else {
        return;
    };
    let Ok((mut header_text, mut header_color)) = header_query.get_single_mut() else {
        return;
    };
    let Ok(mut body_text) = body_query.get_single_mut() else {
        return;
    };

    let message = if let Some(call) = patrol_state.current_dispatch.as_ref() {
        let message = call.description.clone();
        dispatch_state.message = Some(message.clone());
        dispatch_state.display_timer.reset();
        Some(message)
    } else {
        dispatch_state.display_timer.tick(time.delta());
        if dispatch_state.display_timer.finished() {
            dispatch_state.message = None;
        }
        dispatch_state.message.clone()
    };

    let Some(message) = message else {
        *visibility = Visibility::Hidden;
        return;
    };

    dispatch_state.flash_timer.tick(time.delta());

    if dispatch_state.flash_timer.just_finished() {
        dispatch_state.flash_on = !dispatch_state.flash_on;
    }

    *visibility = Visibility::Visible;
    **header_text = "DISPATCH".to_string();
    **body_text = message;

    if dispatch_state.flash_on {
        header_color.0 = Color::srgb(0.98, 0.84, 0.28);
        border.0 = Color::srgb(0.92, 0.30, 0.30);
        background.0 = Color::srgba(0.13, 0.06, 0.08, 0.96);
    } else {
        header_color.0 = Color::srgb(1.0, 0.42, 0.42);
        border.0 = Color::srgb(0.98, 0.84, 0.28);
        background.0 = Color::srgba(0.05, 0.08, 0.13, 0.96);
    }
}

fn log_audio_requests(
    mut sfx_events: EventReader<PlaySfxEvent>,
    mut music_events: EventReader<PlayMusicEvent>,
) {
    for event in sfx_events.read() {
        info!("PlaySfxEvent stub received for '{}'", event.name);
    }

    for event in music_events.read() {
        info!(
            "PlayMusicEvent stub received for '{}' looping={}",
            event.name, event.looping
        );
    }
}

fn skill_tree_name(tree: SkillTree) -> &'static str {
    match tree {
        SkillTree::Investigation => "Investigation",
        SkillTree::Interrogation => "Interrogation",
        SkillTree::Patrol => "Patrol",
        SkillTree::Leadership => "Leadership",
    }
}

fn humanize_source(source: &str) -> String {
    source.split(':').next().unwrap_or(source).replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    use crate::shared::{
        DispatchCall, DispatchEventKind, EvidenceCategory, EvidencePiece, EvidenceProcessingState,
        GameState, MapId, Rank,
    };

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.2,
        )));
        app.init_state::<GameState>();
        app.configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        );
        app.init_resource::<EvidenceLocker>();
        app.add_event::<ToastEvent>();
        app.add_event::<DispatchCallEvent>();
        app.add_event::<EvidenceProcessedEvent>();
        app.add_event::<PlaySfxEvent>();
        app.add_event::<PlayMusicEvent>();
        app.add_event::<CaseSolvedEvent>();
        app.add_event::<PromotionEvent>();
        app.add_event::<SkillPointSpentEvent>();
        build_notifications(&mut app);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    #[test]
    fn toast_events_spawn_cards_and_expire() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().send_event(ToastEvent {
            message: "Evidence bagged".to_string(),
            duration_secs: 0.3,
        });
        app.update();

        let mut toast_query = app.world_mut().query_filtered::<Entity, With<ToastCard>>();
        assert_eq!(toast_query.iter(app.world()).count(), 1);

        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.5,
        )));
        app.update();
        app.update();

        let mut toast_query = app.world_mut().query_filtered::<Entity, With<ToastCard>>();
        assert_eq!(toast_query.iter(app.world()).count(), 0);
    }

    #[test]
    fn dispatch_event_updates_visible_banner_text() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().send_event(DispatchCallEvent {
            call: DispatchCall {
                kind: DispatchEventKind::OfficerNeedsBackup,
                map_id: MapId::Downtown,
                description: "Officer needs backup in Downtown".to_string(),
                fatigue_cost: 12.0,
                stress_cost: 10.0,
                xp_reward: 20,
                may_generate_evidence: false,
            },
        });
        app.update();

        let mut banner_query = app
            .world_mut()
            .query_filtered::<&Visibility, With<DispatchBanner>>();
        assert_eq!(*banner_query.single(app.world()), Visibility::Visible);

        let mut body_query = app
            .world_mut()
            .query_filtered::<&Text, With<DispatchBodyText>>();
        assert_eq!(
            body_query.single(app.world()).as_str(),
            "Officer needs backup in Downtown"
        );
    }

    #[test]
    fn evidence_processed_event_emits_named_toast() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<EvidenceLocker>()
            .pieces
            .push(EvidencePiece {
                id: "dna_match".to_string(),
                name: "DNA Match".to_string(),
                category: EvidenceCategory::Forensic,
                description: "Forensic hit".to_string(),
                quality: 0.9,
                linked_case: Some("case_a".to_string()),
                processing_state: EvidenceProcessingState::Analyzed,
                collected_shift: 1,
                collected_map: MapId::Hospital,
            });

        app.world_mut().send_event(EvidenceProcessedEvent {
            evidence_id: "dna_match".to_string(),
        });
        app.update();

        let events = app.world().resource::<Events<ToastEvent>>();
        let mut reader = events.get_cursor();
        let messages: Vec<_> = reader
            .read(events)
            .map(|event| event.message.clone())
            .collect();
        assert!(messages.contains(&"Evidence analyzed: DNA Match".to_string()));
    }

    #[test]
    fn promotion_events_emit_player_visible_toasts() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().send_event(PromotionEvent {
            new_rank: Rank::Detective,
        });
        app.update();

        let events = app.world().resource::<Events<ToastEvent>>();
        let mut reader = events.get_cursor();
        let messages: Vec<_> = reader
            .read(events)
            .map(|event| event.message.clone())
            .collect();
        assert!(messages.contains(&"Promoted to Detective".to_string()));
    }
}
