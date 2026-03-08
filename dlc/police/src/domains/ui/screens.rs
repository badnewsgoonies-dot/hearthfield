use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::domains::cases::{case_definition, CaseCloseRequestedEvent};
use crate::domains::skills::perk_definition;
use crate::shared::{
    CaseBoard, CaseId, CaseStatus, DialogueEndEvent, DialogueStartEvent, Economy, EvidenceLocker,
    EvidenceProcessingState, GameState, InputContext, InterrogationEndEvent,
    InterrogationStartEvent, NpcId, NpcRegistry, NpcTrustChangeEvent, Rank, ShiftClock,
    SkillPointSpentEvent, SkillTree, Skills, ToastEvent, UpdatePhase, MAX_PRESSURE, MAX_TRUST,
    MIN_TRUST,
};

const METER_WIDTH: f32 = 240.0;
const METER_HEIGHT: f32 = 20.0;

#[derive(Resource, Debug, Clone, Default)]
struct DialogueUiState {
    npc_id: Option<NpcId>,
    context: Option<String>,
}

#[derive(Resource, Debug, Clone, Default)]
struct InterrogationUiState {
    npc_id: Option<NpcId>,
    case_id: Option<CaseId>,
}

#[derive(Resource, Debug, Clone, Default)]
struct EvidenceExamStatus {
    message: String,
}

#[derive(Component)]
struct DialogueScreenRoot;

#[derive(Component)]
struct DialogueBodyText;

#[derive(Component)]
struct DialogueButton(DialogueAction);

#[derive(Clone, Copy, PartialEq, Eq)]
enum DialogueAction {
    Back,
}

#[derive(Component)]
struct CaseFileRoot;

#[derive(Component)]
struct CaseFileBodyText;

#[derive(Component)]
struct CaseFileButton(CaseFileAction);

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseFileAction {
    CloseCase,
    Back,
}

#[derive(Component)]
struct SkillTreeRoot;

#[derive(Component)]
struct SkillTreePointsText;

#[derive(Component)]
struct SkillTreeBackButton;

#[derive(Component, Clone, Copy)]
struct SkillPerkButton {
    tree: SkillTree,
    level: u8,
}

#[derive(Component, Clone, Copy)]
struct SkillPerkLabel {
    tree: SkillTree,
    level: u8,
}

#[derive(Component)]
struct CareerViewRoot;

#[derive(Component)]
struct CareerBodyText;

#[derive(Component)]
struct CareerBackButton;

#[derive(Component)]
struct InterrogationRoot;

#[derive(Component)]
struct InterrogationNpcNameText;

#[derive(Component)]
struct InterrogationSummaryText;

#[derive(Component)]
struct InterrogationTrustFill;

#[derive(Component)]
struct InterrogationPressureFill;

#[derive(Component)]
struct InterrogationButton(InterrogationAction);

#[derive(Clone, Copy, PartialEq, Eq)]
enum InterrogationAction {
    BuildTrust,
    ApplyPressure,
    AskAboutEvidence,
    End,
}

#[derive(Component)]
struct EvidenceExamRoot;

#[derive(Component)]
struct EvidenceExamBodyText;

#[derive(Component)]
struct EvidenceExamStatusText;

#[derive(Component)]
struct EvidenceExamButton(EvidenceExamAction);

#[derive(Clone, Copy, PartialEq, Eq)]
enum EvidenceExamAction {
    ProcessAllRaw,
    Back,
}

type SkillTreeBackButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (
        Changed<Interaction>,
        With<SkillTreeBackButton>,
        Without<SkillPerkButton>,
    ),
>;

type SkillPerkInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static SkillPerkButton,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, Without<SkillTreeBackButton>),
>;

type CareerBackButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<CareerBackButton>),
>;

type InterrogationNameTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<InterrogationNpcNameText>,
        Without<InterrogationSummaryText>,
    ),
>;

type InterrogationSummaryTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<InterrogationSummaryText>,
        Without<InterrogationNpcNameText>,
    ),
>;

type InterrogationTrustFillQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<InterrogationTrustFill>,
        Without<InterrogationPressureFill>,
    ),
>;

type InterrogationPressureFillQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<InterrogationPressureFill>,
        Without<InterrogationTrustFill>,
    ),
>;

type EvidenceExamBodyTextQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<EvidenceExamBodyText>, Without<EvidenceExamStatusText>)>;

type EvidenceExamStatusTextQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<EvidenceExamStatusText>, Without<EvidenceExamBodyText>)>;

pub(super) fn install_screen_systems(app: &mut App) {
    app.init_resource::<InputContext>()
        .init_resource::<DialogueUiState>()
        .init_resource::<InterrogationUiState>()
        .init_resource::<EvidenceExamStatus>()
        .add_event::<DialogueEndEvent>()
        .add_event::<CaseCloseRequestedEvent>()
        .add_event::<SkillPointSpentEvent>()
        .add_event::<NpcTrustChangeEvent>()
        .add_event::<InterrogationEndEvent>()
        .add_event::<ToastEvent>()
        .add_systems(Update, sync_input_context)
        .add_systems(
            Update,
            (track_dialogue_session, track_interrogation_session).in_set(UpdatePhase::Reactions),
        )
        .add_systems(OnEnter(GameState::Dialogue), spawn_dialogue_screen)
        .add_systems(
            Update,
            (handle_dialogue_buttons, update_dialogue_screen)
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::Dialogue)),
        )
        .add_systems(OnExit(GameState::Dialogue), cleanup_dialogue_screen)
        .add_systems(OnEnter(GameState::CaseFile), spawn_case_file_screen)
        .add_systems(
            Update,
            handle_case_file_buttons
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::CaseFile)),
        )
        .add_systems(OnExit(GameState::CaseFile), cleanup_case_file_screen)
        .add_systems(OnEnter(GameState::SkillTree), spawn_skill_tree_screen)
        .add_systems(
            Update,
            (handle_skill_tree_buttons, update_skill_tree_screen)
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::SkillTree)),
        )
        .add_systems(OnExit(GameState::SkillTree), cleanup_skill_tree_screen)
        .add_systems(OnEnter(GameState::CareerView), spawn_career_view_screen)
        .add_systems(
            Update,
            (handle_career_view_buttons, update_career_view_screen)
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::CareerView)),
        )
        .add_systems(OnExit(GameState::CareerView), cleanup_career_view_screen)
        .add_systems(
            OnEnter(GameState::Interrogation),
            spawn_interrogation_screen,
        )
        .add_systems(
            Update,
            (handle_interrogation_buttons, update_interrogation_screen)
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::Interrogation)),
        )
        .add_systems(
            OnExit(GameState::Interrogation),
            cleanup_interrogation_screen,
        )
        .add_systems(OnEnter(GameState::EvidenceExam), spawn_evidence_exam_screen)
        .add_systems(
            Update,
            (handle_evidence_exam_buttons, update_evidence_exam_screen)
                .in_set(UpdatePhase::Presentation)
                .run_if(in_state(GameState::EvidenceExam)),
        )
        .add_systems(
            OnExit(GameState::EvidenceExam),
            cleanup_evidence_exam_screen,
        );
}

fn sync_input_context(state: Res<State<GameState>>, mut input_context: ResMut<InputContext>) {
    input_context.in_dialogue = *state.get() == GameState::Dialogue;
    input_context.in_interrogation = *state.get() == GameState::Interrogation;
    input_context.in_menu = matches!(
        state.get(),
        GameState::MainMenu
            | GameState::Paused
            | GameState::CaseFile
            | GameState::SkillTree
            | GameState::CareerView
            | GameState::EvidenceExam
            | GameState::MapView
            | GameState::ShiftSummary
            | GameState::Precinct
    );
}

fn track_dialogue_session(
    mut start_events: EventReader<DialogueStartEvent>,
    mut end_events: EventReader<DialogueEndEvent>,
    mut dialogue_ui_state: ResMut<DialogueUiState>,
) {
    for event in start_events.read() {
        dialogue_ui_state.npc_id = Some(event.npc_id.clone());
        dialogue_ui_state.context = Some(event.context.clone());
    }

    if end_events.read().next().is_some() {
        dialogue_ui_state.npc_id = None;
        dialogue_ui_state.context = None;
    }
}

fn track_interrogation_session(
    mut start_events: EventReader<InterrogationStartEvent>,
    mut end_events: EventReader<InterrogationEndEvent>,
    mut interrogation_ui_state: ResMut<InterrogationUiState>,
) {
    for event in start_events.read() {
        interrogation_ui_state.npc_id = Some(event.npc_id.clone());
        interrogation_ui_state.case_id = Some(event.case_id.clone());
    }

    if end_events.read().next().is_some() {
        interrogation_ui_state.npc_id = None;
        interrogation_ui_state.case_id = None;
    }
}

fn spawn_dialogue_screen(mut commands: Commands) {
    commands
        .spawn(screen_root(DialogueScreenRoot))
        .with_children(|parent| {
            spawn_screen_title(parent, "Conversation");
            parent.spawn((
                DialogueBodyText,
                Text::new(""),
                body_font(),
                TextColor(Color::WHITE),
            ));
            spawn_text_button(parent, "Back", button_color(DialogueAction::Back), || {
                DialogueButton(DialogueAction::Back)
            });
        });
}

fn update_dialogue_screen(
    dialogue_ui_state: Res<DialogueUiState>,
    npc_registry: Res<NpcRegistry>,
    mut body_query: Query<&mut Text, With<DialogueBodyText>>,
) {
    let Ok(mut body_text) = body_query.get_single_mut() else {
        return;
    };

    let npc_name = dialogue_ui_state
        .npc_id
        .as_ref()
        .and_then(|npc_id| npc_registry.definitions.get(npc_id))
        .map(|npc| npc.name.as_str())
        .unwrap_or("Unknown contact");
    let context = dialogue_ui_state
        .context
        .as_deref()
        .unwrap_or("npc_interaction")
        .replace('_', " ");

    **body_text = format!(
        "{npc_name}\n\nConversation context: {context}\n\nPress Back or Escape to return to patrol."
    );
}

fn handle_dialogue_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &DialogueButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut dialogue_end_events: EventWriter<DialogueEndEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }

    for (interaction, button, mut background) in &mut interaction_query {
        background.0 = button_background(*interaction, button.0);
        if *interaction == Interaction::Pressed && button.0 == DialogueAction::Back {
            dialogue_end_events.send(DialogueEndEvent);
            next_state.set(GameState::Playing);
        }
    }
}

fn cleanup_dialogue_screen(
    mut commands: Commands,
    root_query: Query<Entity, With<DialogueScreenRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_case_file_screen(
    mut commands: Commands,
    case_board: Res<CaseBoard>,
    evidence_locker: Res<EvidenceLocker>,
    npc_registry: Res<NpcRegistry>,
) {
    let selected_case = selected_case(case_board.as_ref());
    let case_text = build_case_file_text(selected_case, &evidence_locker, &npc_registry);
    let can_close_case = selected_case
        .map(|active_case| active_case.status == CaseStatus::EvidenceComplete)
        .unwrap_or(false);

    commands
        .spawn(screen_root(CaseFileRoot))
        .with_children(|parent| {
            spawn_screen_title(parent, "Case File");
            parent.spawn((
                CaseFileBodyText,
                Text::new(case_text),
                body_font(),
                TextColor(Color::WHITE),
            ));

            if can_close_case {
                spawn_text_button(
                    parent,
                    "Close Case",
                    button_color(CaseFileAction::CloseCase),
                    || CaseFileButton(CaseFileAction::CloseCase),
                );
            }

            spawn_text_button(parent, "Back", button_color(CaseFileAction::Back), || {
                CaseFileButton(CaseFileAction::Back)
            });
        });
}

fn handle_case_file_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    case_board: Res<CaseBoard>,
    mut interaction_query: Query<
        (&Interaction, &CaseFileButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut close_requests: EventWriter<CaseCloseRequestedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }

    for (interaction, button, mut background) in &mut interaction_query {
        background.0 = button_background(*interaction, button.0);

        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.0 {
            CaseFileAction::CloseCase => {
                if let Some(active_case) = selected_case(case_board.as_ref())
                    .filter(|active_case| active_case.status == CaseStatus::EvidenceComplete)
                {
                    close_requests.send(CaseCloseRequestedEvent {
                        case_id: active_case.case_id.clone(),
                    });
                }
                next_state.set(GameState::Playing);
            }
            CaseFileAction::Back => next_state.set(GameState::Playing),
        }
    }
}

fn cleanup_case_file_screen(mut commands: Commands, root_query: Query<Entity, With<CaseFileRoot>>) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_skill_tree_screen(mut commands: Commands) {
    let button_size = Vec2::new(190.0, 72.0);

    commands
        .spawn(screen_root(SkillTreeRoot))
        .with_children(|parent| {
            spawn_screen_title(parent, "Skill Tree");
            parent.spawn((
                SkillTreePointsText,
                Text::new("Available points: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.88, 0.52)),
            ));

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|row| {
                    for tree in [
                        SkillTree::Investigation,
                        SkillTree::Interrogation,
                        SkillTree::Patrol,
                        SkillTree::Leadership,
                    ] {
                        row.spawn((
                            Node {
                                width: Val::Px(button_size.x),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|column| {
                            column.spawn((
                                Text::new(skill_tree_name(tree)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            for level in 1..=5 {
                                column
                                    .spawn((
                                        SkillPerkButton { tree, level },
                                        Button,
                                        Node {
                                            width: Val::Px(button_size.x),
                                            min_height: Val::Px(button_size.y),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            padding: UiRect::all(Val::Px(10.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.16, 0.18, 0.24)),
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            SkillPerkLabel { tree, level },
                                            Text::new(""),
                                            TextFont {
                                                font_size: 17.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            }
                        });
                    }
                });

            spawn_text_button(parent, "Back", button_color(()), || SkillTreeBackButton);
        });
}

fn update_skill_tree_screen(
    skills: Res<Skills>,
    mut points_query: Query<&mut Text, (With<SkillTreePointsText>, Without<SkillPerkLabel>)>,
    mut button_query: Query<(&SkillPerkButton, &mut BackgroundColor)>,
    mut label_query: Query<
        (&SkillPerkLabel, &mut Text, &mut TextColor),
        Without<SkillTreePointsText>,
    >,
) {
    if let Ok(mut points_text) = points_query.get_single_mut() {
        **points_text = format!("Available points: {}", skills.available_points);
    }

    for (button, mut background) in &mut button_query {
        let current_level = skill_level(&skills, button.tree);
        let next_available = button.level == current_level.saturating_add(1)
            && skills.available_points > 0
            && current_level < 5;
        let unlocked = button.level <= current_level;

        background.0 = if unlocked {
            Color::srgb(0.26, 0.48, 0.30)
        } else if next_available {
            Color::srgb(0.68, 0.48, 0.18)
        } else {
            Color::srgb(0.16, 0.18, 0.24)
        };
    }

    for (label, mut text, mut text_color) in &mut label_query {
        let current_level = skill_level(&skills, label.tree);
        let next_available = label.level == current_level.saturating_add(1)
            && skills.available_points > 0
            && current_level < 5;
        let perk = perk_definition(label.tree, label.level)
            .map(|perk| perk.name)
            .unwrap_or("Unknown");

        **text = format!("{}. {}", label.level, perk);
        text_color.0 = if label.level <= current_level {
            Color::srgb(0.96, 0.96, 0.96)
        } else if next_available {
            Color::srgb(0.14, 0.08, 0.02)
        } else {
            Color::srgb(0.60, 0.64, 0.72)
        };
    }
}

fn handle_skill_tree_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    skills: Res<Skills>,
    mut interaction_query: SkillPerkInteractionQuery,
    mut back_query: SkillTreeBackButtonQuery,
    mut spend_events: EventWriter<SkillPointSpentEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }

    for (interaction, button, mut background) in &mut interaction_query {
        let action_style = if skill_level(&skills, button.tree) >= button.level {
            ButtonStyleKey::Completed
        } else if button.level == skill_level(&skills, button.tree).saturating_add(1)
            && skills.available_points > 0
        {
            ButtonStyleKey::Primary
        } else {
            ButtonStyleKey::Disabled
        };
        background.0 = button_style_color(*interaction, action_style);

        if *interaction == Interaction::Pressed
            && button.level == skill_level(&skills, button.tree).saturating_add(1)
            && skills.available_points > 0
        {
            spend_events.send(SkillPointSpentEvent {
                tree: button.tree,
                new_level: button.level,
            });
        }
    }

    for (interaction, mut background) in &mut back_query {
        background.0 = button_style_color(*interaction, ButtonStyleKey::Secondary);
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }
}

fn cleanup_skill_tree_screen(
    mut commands: Commands,
    root_query: Query<Entity, With<SkillTreeRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_career_view_screen(mut commands: Commands) {
    commands
        .spawn(screen_root(CareerViewRoot))
        .with_children(|parent| {
            spawn_screen_title(parent, "Career");
            parent.spawn((
                CareerBodyText,
                Text::new(""),
                body_font(),
                TextColor(Color::WHITE),
            ));
            spawn_text_button(parent, "Back", button_color(()), || CareerBackButton);
        });
}

fn update_career_view_screen(
    clock: Res<ShiftClock>,
    skills: Res<Skills>,
    case_board: Res<CaseBoard>,
    economy: Res<Economy>,
    mut body_query: Query<&mut Text, With<CareerBodyText>>,
) {
    let Ok(mut body_text) = body_query.get_single_mut() else {
        return;
    };

    let requirements = next_rank_requirements(clock.rank);
    let body = if let Some(requirements) = requirements {
        format!(
            "Current rank: {}\nTotal XP: {}\nCases solved: {}\nReputation: {}\nDepartment budget: ${}\n\nNext rank: {}\nXP requirement: {} ({})\nCases requirement: {} ({})\nReputation requirement: {} ({})",
            clock.rank.display_name(),
            skills.total_xp,
            case_board.total_cases_solved,
            economy.reputation,
            economy.department_budget,
            requirements.next_rank.display_name(),
            requirements.required_xp,
            requirement_status(skills.total_xp >= requirements.required_xp),
            requirements.required_cases_solved,
            requirement_status(case_board.total_cases_solved >= requirements.required_cases_solved),
            requirements.required_reputation,
            requirement_status(economy.reputation >= requirements.required_reputation),
        )
    } else {
        format!(
            "Current rank: {}\nTotal XP: {}\nCases solved: {}\nReputation: {}\nDepartment budget: ${}\n\nYou have reached the top of the department ladder.",
            clock.rank.display_name(),
            skills.total_xp,
            case_board.total_cases_solved,
            economy.reputation,
            economy.department_budget,
        )
    };

    **body_text = body;
}

fn handle_career_view_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_query: CareerBackButtonQuery,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }

    for (interaction, mut background) in &mut interaction_query {
        background.0 = button_style_color(*interaction, ButtonStyleKey::Secondary);
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }
}

fn cleanup_career_view_screen(
    mut commands: Commands,
    root_query: Query<Entity, With<CareerViewRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_interrogation_screen(mut commands: Commands) {
    commands
        .spawn(screen_root(InterrogationRoot))
        .with_children(|parent| {
            spawn_screen_title(parent, "Interrogation");
            parent.spawn((
                InterrogationNpcNameText,
                Text::new("Unknown suspect"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(24.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(220.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.14, 0.16, 0.22)),
                        BorderColor(Color::srgb(0.56, 0.62, 0.78)),
                    ))
                    .with_children(|portrait| {
                        portrait.spawn((
                            Text::new("PORTRAIT"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.74, 0.78, 0.86)),
                        ));
                    });

                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|panel| {
                        spawn_meter(
                            panel,
                            "Trust",
                            InterrogationTrustFill,
                            Color::srgb(0.28, 0.74, 0.48),
                        );
                        spawn_meter(
                            panel,
                            "Pressure",
                            InterrogationPressureFill,
                            Color::srgb(0.88, 0.32, 0.30),
                        );
                        panel.spawn((
                            InterrogationSummaryText,
                            Text::new(""),
                            body_font(),
                            TextColor(Color::WHITE),
                        ));
                    });
                });

            for (label, action) in [
                ("Build Trust", InterrogationAction::BuildTrust),
                ("Apply Pressure", InterrogationAction::ApplyPressure),
                ("Ask About Evidence", InterrogationAction::AskAboutEvidence),
                ("End Interrogation", InterrogationAction::End),
            ] {
                spawn_text_button(parent, label, button_color(action), || {
                    InterrogationButton(action)
                });
            }
        });
}

fn update_interrogation_screen(
    interrogation_ui_state: Res<InterrogationUiState>,
    npc_registry: Res<NpcRegistry>,
    mut name_query: InterrogationNameTextQuery,
    mut summary_query: InterrogationSummaryTextQuery,
    mut trust_query: InterrogationTrustFillQuery,
    mut pressure_query: InterrogationPressureFillQuery,
) {
    let Some(npc_id) = interrogation_ui_state.npc_id.as_ref() else {
        return;
    };

    let npc_name = npc_registry
        .definitions
        .get(npc_id)
        .map(|npc| npc.name.as_str())
        .unwrap_or("Unknown suspect");
    let relationship = npc_registry.relationships.get(npc_id);
    let trust = relationship.map(|rel| rel.trust).unwrap_or_default();
    let pressure = relationship.map(|rel| rel.pressure).unwrap_or_default();

    if let Ok(mut name_text) = name_query.get_single_mut() {
        **name_text = npc_name.to_string();
    }

    if let Ok(mut summary_text) = summary_query.get_single_mut() {
        **summary_text = if pressure > 80 {
            format!(
                "Trust: {trust} | Pressure: {pressure}\nPressure is high enough that ending the interview may crack them."
            )
        } else {
            format!(
                "Trust: {trust} | Pressure: {pressure}\nPush harder or build rapport before ending the interrogation."
            )
        };
    }

    if let Ok(mut trust_fill) = trust_query.get_single_mut() {
        trust_fill.width = Val::Px(trust_meter_width(trust));
    }

    if let Ok(mut pressure_fill) = pressure_query.get_single_mut() {
        pressure_fill.width = Val::Px(pressure_meter_width(pressure));
    }
}

fn handle_interrogation_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    interrogation_ui_state: Res<InterrogationUiState>,
    npc_registry: Res<NpcRegistry>,
    mut interaction_query: Query<
        (&Interaction, &InterrogationButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut output: InterrogationButtonOutput,
) {
    let Some(npc_id) = interrogation_ui_state.npc_id.as_ref() else {
        return;
    };
    let Some(case_id) = interrogation_ui_state.case_id.as_ref() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        output.interrogation_end_events.send(InterrogationEndEvent {
            npc_id: npc_id.clone(),
            case_id: case_id.clone(),
            confession: false,
        });
        output.next_state.set(GameState::Playing);
    }

    for (interaction, button, mut background) in &mut interaction_query {
        background.0 = button_background(*interaction, button.0);

        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.0 {
            InterrogationAction::BuildTrust => {
                output.trust_events.send(NpcTrustChangeEvent {
                    npc_id: npc_id.clone(),
                    trust_delta: 12,
                    pressure_delta: -4,
                });
            }
            InterrogationAction::ApplyPressure => {
                output.trust_events.send(NpcTrustChangeEvent {
                    npc_id: npc_id.clone(),
                    trust_delta: -8,
                    pressure_delta: 15,
                });
            }
            InterrogationAction::AskAboutEvidence => {
                output.trust_events.send(NpcTrustChangeEvent {
                    npc_id: npc_id.clone(),
                    trust_delta: 0,
                    pressure_delta: 0,
                });
            }
            InterrogationAction::End => {
                let relationship = npc_registry.relationships.get(npc_id);
                let pressure = relationship.map(|rel| rel.pressure).unwrap_or_default();
                let trust = relationship.map(|rel| rel.trust).unwrap_or_default();
                let confession =
                    pressure > 80 && confession_roll_succeeds(npc_id, case_id, trust, pressure);

                output.interrogation_end_events.send(InterrogationEndEvent {
                    npc_id: npc_id.clone(),
                    case_id: case_id.clone(),
                    confession,
                });
                output.toast_events.send(ToastEvent {
                    message: if confession {
                        "Confession secured.".to_string()
                    } else {
                        "Interrogation concluded without a confession.".to_string()
                    },
                    duration_secs: 2.4,
                });
                output.next_state.set(GameState::Playing);
            }
        }
    }
}

#[derive(SystemParam)]
struct InterrogationButtonOutput<'w, 's> {
    trust_events: EventWriter<'w, NpcTrustChangeEvent>,
    interrogation_end_events: EventWriter<'w, InterrogationEndEvent>,
    toast_events: EventWriter<'w, ToastEvent>,
    next_state: ResMut<'w, NextState<GameState>>,
    marker: std::marker::PhantomData<&'s ()>,
}

fn cleanup_interrogation_screen(
    mut commands: Commands,
    root_query: Query<Entity, With<InterrogationRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_evidence_exam_screen(
    mut commands: Commands,
    mut evidence_exam_status: ResMut<EvidenceExamStatus>,
) {
    evidence_exam_status.message.clear();

    commands
        .spawn(screen_root(EvidenceExamRoot))
        .with_children(|parent| {
            spawn_screen_title(parent, "Evidence Examination");
            parent.spawn((
                EvidenceExamStatusText,
                Text::new(""),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.86, 0.62)),
            ));
            parent.spawn((
                EvidenceExamBodyText,
                Text::new(""),
                body_font(),
                TextColor(Color::WHITE),
            ));

            spawn_text_button(
                parent,
                "Process All Raw",
                button_color(EvidenceExamAction::ProcessAllRaw),
                || EvidenceExamButton(EvidenceExamAction::ProcessAllRaw),
            );
            spawn_text_button(
                parent,
                "Back",
                button_color(EvidenceExamAction::Back),
                || EvidenceExamButton(EvidenceExamAction::Back),
            );
        });
}

fn update_evidence_exam_screen(
    locker: Res<EvidenceLocker>,
    evidence_exam_status: Res<EvidenceExamStatus>,
    mut body_query: EvidenceExamBodyTextQuery,
    mut status_query: EvidenceExamStatusTextQuery,
) {
    if let Ok(mut status_text) = status_query.get_single_mut() {
        **status_text = evidence_exam_status.message.clone();
    }

    if let Ok(mut body_text) = body_query.get_single_mut() {
        **body_text = build_evidence_exam_text(&locker);
    }
}

fn handle_evidence_exam_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut locker: ResMut<EvidenceLocker>,
    mut evidence_exam_status: ResMut<EvidenceExamStatus>,
    mut interaction_query: Query<
        (&Interaction, &EvidenceExamButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }

    for (interaction, button, mut background) in &mut interaction_query {
        background.0 = button_background(*interaction, button.0);

        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.0 {
            EvidenceExamAction::ProcessAllRaw => {
                let started = start_processing_all_raw(&mut locker);
                evidence_exam_status.message = if started > 0 {
                    format!("Queued {started} raw evidence item(s) for analysis.")
                } else {
                    "No raw evidence is waiting for analysis.".to_string()
                };
            }
            EvidenceExamAction::Back => next_state.set(GameState::Playing),
        }
    }
}

fn cleanup_evidence_exam_screen(
    mut commands: Commands,
    root_query: Query<Entity, With<EvidenceExamRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn screen_root<T: Component>(marker: T) -> impl Bundle {
    (
        marker,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(18.0),
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.94)),
        GlobalZIndex(20),
    )
}

fn spawn_screen_title(parent: &mut ChildBuilder, title: &str) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font_size: 38.0,
            ..default()
        },
        TextColor(Color::srgb(0.94, 0.88, 0.62)),
    ));
}

fn spawn_text_button<T: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    color: Color,
    component: impl FnOnce() -> T,
) {
    parent
        .spawn((
            component(),
            Button,
            Node {
                width: Val::Px(260.0),
                min_height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(color),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_meter<T: Component>(parent: &mut ChildBuilder, label: &str, marker: T, fill_color: Color) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    parent
        .spawn((
            Node {
                width: Val::Px(METER_WIDTH),
                height: Val::Px(METER_HEIGHT),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.16, 0.22)),
        ))
        .with_children(|bar| {
            bar.spawn((
                marker,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
            ));
        });
}

fn body_font() -> TextFont {
    TextFont {
        font_size: 21.0,
        ..default()
    }
}

fn selected_case(case_board: &CaseBoard) -> Option<&crate::shared::ActiveCase> {
    case_board
        .active
        .iter()
        .find(|active_case| active_case.status == CaseStatus::EvidenceComplete)
        .or_else(|| case_board.active.first())
}

fn build_case_file_text(
    active_case: Option<&crate::shared::ActiveCase>,
    evidence_locker: &EvidenceLocker,
    npc_registry: &NpcRegistry,
) -> String {
    let Some(active_case) = active_case else {
        return "No active cases.\n\nTake a case from the board to open a live case file."
            .to_string();
    };

    let Some(case_def) = case_definition(&active_case.case_id) else {
        return format!("Missing authored data for case {}.", active_case.case_id);
    };

    let required_evidence = case_def
        .evidence_required
        .iter()
        .map(|evidence_id| {
            let piece = evidence_locker
                .pieces
                .iter()
                .filter(|piece| piece.id == *evidence_id)
                .filter(|piece| piece.linked_case.as_deref() == Some(active_case.case_id.as_str()))
                .max_by(|a, b| a.quality.total_cmp(&b.quality));

            if let Some(piece) = piece {
                format!(
                    "[Collected] {} ({:.0}% quality)",
                    piece.name,
                    piece.quality * 100.0
                )
            } else if active_case.evidence_collected.contains(evidence_id) {
                format!("[Collected] {}", evidence_id.replace('_', " "))
            } else {
                format!("[Missing] {}", evidence_id.replace('_', " "))
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let witnesses = npc_list_text(&case_def.witnesses, npc_registry);
    let suspects = npc_list_text(&case_def.suspects, npc_registry);

    format!(
        "{}\nStatus: {:?}\n\n{}\n\nRequired evidence:\n{}\n\nWitnesses:\n{}\n\nSuspects:\n{}\n",
        case_def.name,
        active_case.status,
        case_def.description,
        required_evidence,
        witnesses,
        suspects,
    )
}

fn npc_list_text(npc_ids: &[NpcId], npc_registry: &NpcRegistry) -> String {
    if npc_ids.is_empty() {
        return "None".to_string();
    }

    npc_ids
        .iter()
        .map(|npc_id| {
            npc_registry
                .definitions
                .get(npc_id)
                .map(|npc| npc.name.clone())
                .unwrap_or_else(|| npc_id.clone())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_evidence_exam_text(locker: &EvidenceLocker) -> String {
    if locker.pieces.is_empty() {
        return "No evidence is stored in the locker.".to_string();
    }

    locker
        .pieces
        .iter()
        .map(|piece| {
            format!(
                "{} | {:?} | {:.0}% | {:?} | {}",
                piece.name,
                piece.category,
                piece.quality * 100.0,
                piece.processing_state,
                piece.linked_case.as_deref().unwrap_or("Unlinked")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn start_processing_all_raw(locker: &mut EvidenceLocker) -> usize {
    let mut started = 0;

    for piece in &mut locker.pieces {
        if piece.processing_state == EvidenceProcessingState::Raw {
            piece.processing_state = EvidenceProcessingState::Processing;
            started += 1;
        }
    }

    started
}

#[derive(Clone, Copy)]
struct PromotionRequirements {
    next_rank: Rank,
    required_xp: u32,
    required_cases_solved: u32,
    required_reputation: i32,
}

fn next_rank_requirements(current_rank: Rank) -> Option<PromotionRequirements> {
    use crate::shared::{
        PROMOTION_DETECTIVE_CASES, PROMOTION_DETECTIVE_REP, PROMOTION_DETECTIVE_XP,
        PROMOTION_LIEUTENANT_CASES, PROMOTION_LIEUTENANT_REP, PROMOTION_LIEUTENANT_XP,
        PROMOTION_SERGEANT_CASES, PROMOTION_SERGEANT_REP, PROMOTION_SERGEANT_XP,
    };

    match current_rank {
        Rank::PatrolOfficer => Some(PromotionRequirements {
            next_rank: Rank::Detective,
            required_xp: PROMOTION_DETECTIVE_XP,
            required_cases_solved: PROMOTION_DETECTIVE_CASES,
            required_reputation: PROMOTION_DETECTIVE_REP,
        }),
        Rank::Detective => Some(PromotionRequirements {
            next_rank: Rank::Sergeant,
            required_xp: PROMOTION_SERGEANT_XP,
            required_cases_solved: PROMOTION_SERGEANT_CASES,
            required_reputation: PROMOTION_SERGEANT_REP,
        }),
        Rank::Sergeant => Some(PromotionRequirements {
            next_rank: Rank::Lieutenant,
            required_xp: PROMOTION_LIEUTENANT_XP,
            required_cases_solved: PROMOTION_LIEUTENANT_CASES,
            required_reputation: PROMOTION_LIEUTENANT_REP,
        }),
        Rank::Lieutenant => None,
    }
}

fn requirement_status(met: bool) -> &'static str {
    if met {
        "met"
    } else {
        "not met"
    }
}

fn skill_level(skills: &Skills, tree: SkillTree) -> u8 {
    match tree {
        SkillTree::Investigation => skills.investigation_level,
        SkillTree::Interrogation => skills.interrogation_level,
        SkillTree::Patrol => skills.patrol_level,
        SkillTree::Leadership => skills.leadership_level,
    }
}

fn trust_meter_width(trust: i32) -> f32 {
    let ratio = (trust - MIN_TRUST) as f32 / (MAX_TRUST - MIN_TRUST) as f32;
    ratio.clamp(0.0, 1.0) * METER_WIDTH
}

fn pressure_meter_width(pressure: i32) -> f32 {
    (pressure as f32 / MAX_PRESSURE as f32).clamp(0.0, 1.0) * METER_WIDTH
}

fn confession_roll_succeeds(npc_id: &str, case_id: &str, trust: i32, pressure: i32) -> bool {
    let chance = (pressure - 40 + (-trust).max(0) / 4).clamp(0, 100) as u64;
    let mut hasher = DefaultHasher::new();
    npc_id.hash(&mut hasher);
    case_id.hash(&mut hasher);
    trust.hash(&mut hasher);
    pressure.hash(&mut hasher);
    hasher.finish() % 100 < chance
}

fn skill_tree_name(tree: SkillTree) -> &'static str {
    match tree {
        SkillTree::Investigation => "Investigation",
        SkillTree::Interrogation => "Interrogation",
        SkillTree::Patrol => "Patrol",
        SkillTree::Leadership => "Leadership",
    }
}

#[derive(Clone, Copy)]
enum ButtonStyleKey {
    Primary,
    Secondary,
    Disabled,
    Completed,
}

fn button_style_color(interaction: Interaction, style: ButtonStyleKey) -> Color {
    match (style, interaction) {
        (ButtonStyleKey::Primary, Interaction::Pressed) => Color::srgb(0.92, 0.76, 0.32),
        (ButtonStyleKey::Primary, Interaction::Hovered) => Color::srgb(0.72, 0.54, 0.20),
        (ButtonStyleKey::Primary, Interaction::None) => Color::srgb(0.56, 0.40, 0.16),
        (ButtonStyleKey::Secondary, Interaction::Pressed) => Color::srgb(0.36, 0.54, 0.72),
        (ButtonStyleKey::Secondary, Interaction::Hovered) => Color::srgb(0.28, 0.42, 0.58),
        (ButtonStyleKey::Secondary, Interaction::None) => Color::srgb(0.22, 0.32, 0.46),
        (ButtonStyleKey::Disabled, Interaction::Pressed) => Color::srgb(0.20, 0.22, 0.28),
        (ButtonStyleKey::Disabled, Interaction::Hovered) => Color::srgb(0.18, 0.20, 0.26),
        (ButtonStyleKey::Disabled, Interaction::None) => Color::srgb(0.16, 0.18, 0.24),
        (ButtonStyleKey::Completed, Interaction::Pressed) => Color::srgb(0.34, 0.58, 0.40),
        (ButtonStyleKey::Completed, Interaction::Hovered) => Color::srgb(0.30, 0.52, 0.36),
        (ButtonStyleKey::Completed, Interaction::None) => Color::srgb(0.24, 0.46, 0.30),
    }
}

fn button_color<T>(_action: T) -> Color {
    button_style_color(Interaction::None, ButtonStyleKey::Secondary)
}

fn button_background<T>(interaction: Interaction, action: T) -> Color
where
    T: Copy + Into<ButtonStyleKey>,
{
    button_style_color(interaction, action.into())
}

impl From<DialogueAction> for ButtonStyleKey {
    fn from(_: DialogueAction) -> Self {
        ButtonStyleKey::Secondary
    }
}

impl From<CaseFileAction> for ButtonStyleKey {
    fn from(action: CaseFileAction) -> Self {
        match action {
            CaseFileAction::CloseCase => ButtonStyleKey::Primary,
            CaseFileAction::Back => ButtonStyleKey::Secondary,
        }
    }
}

impl From<InterrogationAction> for ButtonStyleKey {
    fn from(action: InterrogationAction) -> Self {
        match action {
            InterrogationAction::ApplyPressure => ButtonStyleKey::Primary,
            InterrogationAction::End => ButtonStyleKey::Secondary,
            InterrogationAction::BuildTrust | InterrogationAction::AskAboutEvidence => {
                ButtonStyleKey::Secondary
            }
        }
    }
}

impl From<EvidenceExamAction> for ButtonStyleKey {
    fn from(action: EvidenceExamAction) -> Self {
        match action {
            EvidenceExamAction::ProcessAllRaw => ButtonStyleKey::Primary,
            EvidenceExamAction::Back => ButtonStyleKey::Secondary,
        }
    }
}

impl From<()> for ButtonStyleKey {
    fn from(_: ()) -> Self {
        ButtonStyleKey::Secondary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;

    use crate::shared::{
        ActiveCase, EvidenceCategory, EvidencePiece, NpcDef, NpcRelationship, NpcRole, PlayerState,
    };
    use std::collections::HashSet;

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
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
        app.init_resource::<CaseBoard>();
        app.init_resource::<EvidenceLocker>();
        app.init_resource::<NpcRegistry>();
        app.init_resource::<Skills>();
        app.init_resource::<Economy>();
        app.init_resource::<ShiftClock>();
        app.init_resource::<PlayerState>();
        app.init_resource::<InputContext>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_event::<CaseCloseRequestedEvent>();
        app.add_event::<SkillPointSpentEvent>();
        app.add_event::<NpcTrustChangeEvent>();
        app.add_event::<InterrogationStartEvent>();
        app.add_event::<InterrogationEndEvent>();
        app.add_event::<DialogueStartEvent>();
        app.add_event::<DialogueEndEvent>();
        install_screen_systems(&mut app);
        seed_npcs(&mut app);
        app
    }

    fn seed_npcs(app: &mut App) {
        let registry = &mut app.world_mut().resource_mut::<NpcRegistry>();
        registry.definitions.insert(
            "marcus_cole".to_string(),
            NpcDef {
                id: "marcus_cole".to_string(),
                name: "Marcus Cole".to_string(),
                role: NpcRole::Suspect,
                default_map: crate::shared::MapId::Downtown,
                description: "Suspect".to_string(),
            },
        );
        registry.relationships.insert(
            "marcus_cole".to_string(),
            NpcRelationship {
                npc_id: "marcus_cole".to_string(),
                trust: -20,
                pressure: 85,
                favors_done: 0,
                dialogue_flags: HashSet::new(),
            },
        );
    }

    fn set_state(app: &mut App, state: GameState) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        app.update();
        app.update();
    }

    #[test]
    fn case_file_close_button_requests_case_close() {
        let mut app = build_test_app();
        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_001_petty_theft".to_string(),
                status: CaseStatus::EvidenceComplete,
                evidence_collected: vec!["fingerprint".to_string()],
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });

        set_state(&mut app, GameState::CaseFile);

        let mut button_query = app.world_mut().query::<(Entity, &CaseFileButton)>();
        let close_button = button_query
            .iter(app.world())
            .find_map(|(entity, button)| (button.0 == CaseFileAction::CloseCase).then_some(entity))
            .expect("close button should exist");

        app.world_mut()
            .entity_mut(close_button)
            .insert(Interaction::Pressed);
        app.update();

        let events = app.world().resource::<Events<CaseCloseRequestedEvent>>();
        let mut reader = events.get_cursor();
        let requested: Vec<_> = reader
            .read(events)
            .map(|event| event.case_id.clone())
            .collect();
        assert_eq!(requested, vec!["patrol_001_petty_theft".to_string()]);
    }

    #[test]
    fn skill_tree_next_perk_button_emits_spend_event() {
        let mut app = build_test_app();
        {
            let mut skills = app.world_mut().resource_mut::<Skills>();
            skills.total_xp = 100;
            skills.available_points = 1;
        }

        set_state(&mut app, GameState::SkillTree);

        let mut button_query = app.world_mut().query::<(Entity, &SkillPerkButton)>();
        let investigation_one = button_query
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.tree == SkillTree::Investigation && button.level == 1).then_some(entity)
            })
            .expect("first investigation perk should exist");

        app.world_mut()
            .entity_mut(investigation_one)
            .insert(Interaction::Pressed);
        app.update();

        let events = app.world().resource::<Events<SkillPointSpentEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader
            .read(events)
            .map(|event| (event.tree, event.new_level))
            .collect();
        assert_eq!(emitted, vec![(SkillTree::Investigation, 1)]);
    }

    #[test]
    fn interrogation_buttons_emit_trust_and_end_events() {
        let mut app = build_test_app();
        app.world_mut().send_event(InterrogationStartEvent {
            npc_id: "marcus_cole".to_string(),
            case_id: "patrol_001_petty_theft".to_string(),
        });
        app.update();
        set_state(&mut app, GameState::Interrogation);

        let mut button_query = app.world_mut().query::<(Entity, &InterrogationButton)>();
        let build_trust = button_query
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.0 == InterrogationAction::BuildTrust).then_some(entity)
            })
            .expect("build trust button should exist");
        let end_button = button_query
            .iter(app.world())
            .find_map(|(entity, button)| (button.0 == InterrogationAction::End).then_some(entity))
            .expect("end button should exist");

        app.world_mut()
            .entity_mut(build_trust)
            .insert(Interaction::Pressed);
        app.update();

        let trust_events = app.world().resource::<Events<NpcTrustChangeEvent>>();
        let mut trust_reader = trust_events.get_cursor();
        let emitted_trust: Vec<_> = trust_reader
            .read(trust_events)
            .map(|event| {
                (
                    event.npc_id.clone(),
                    event.trust_delta,
                    event.pressure_delta,
                )
            })
            .collect();
        assert_eq!(emitted_trust, vec![("marcus_cole".to_string(), 12, -4)]);

        app.world_mut()
            .entity_mut(end_button)
            .insert(Interaction::Pressed);
        app.update();

        let end_events = app.world().resource::<Events<InterrogationEndEvent>>();
        let mut end_reader = end_events.get_cursor();
        let emitted_end: Vec<_> = end_reader
            .read(end_events)
            .map(|event| (event.npc_id.clone(), event.case_id.clone()))
            .collect();
        assert_eq!(
            emitted_end,
            vec![(
                "marcus_cole".to_string(),
                "patrol_001_petty_theft".to_string()
            )]
        );
    }

    #[test]
    fn evidence_exam_process_all_updates_processing_state() {
        let mut app = build_test_app();
        app.world_mut().resource_mut::<EvidenceLocker>().pieces = vec![
            EvidencePiece {
                id: "fingerprint".to_string(),
                name: "Fingerprint".to_string(),
                category: EvidenceCategory::Physical,
                description: "Print".to_string(),
                quality: 0.8,
                linked_case: Some("patrol_001_petty_theft".to_string()),
                processing_state: EvidenceProcessingState::Raw,
                collected_shift: 1,
                collected_map: crate::shared::MapId::Downtown,
            },
            EvidencePiece {
                id: "dna_match".to_string(),
                name: "DNA Match".to_string(),
                category: EvidenceCategory::Forensic,
                description: "DNA".to_string(),
                quality: 0.9,
                linked_case: Some("patrol_001_petty_theft".to_string()),
                processing_state: EvidenceProcessingState::Analyzed,
                collected_shift: 1,
                collected_map: crate::shared::MapId::Hospital,
            },
        ];

        set_state(&mut app, GameState::EvidenceExam);

        let mut button_query = app.world_mut().query::<(Entity, &EvidenceExamButton)>();
        let process_button = button_query
            .iter(app.world())
            .find_map(|(entity, button)| {
                (button.0 == EvidenceExamAction::ProcessAllRaw).then_some(entity)
            })
            .expect("process button should exist");

        app.world_mut()
            .entity_mut(process_button)
            .insert(Interaction::Pressed);
        app.update();

        let states: Vec<_> = app
            .world()
            .resource::<EvidenceLocker>()
            .pieces
            .iter()
            .map(|piece| piece.processing_state)
            .collect();
        assert_eq!(
            states,
            vec![
                EvidenceProcessingState::Processing,
                EvidenceProcessingState::Analyzed
            ]
        );
    }
}
