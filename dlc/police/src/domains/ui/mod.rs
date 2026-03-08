use bevy::{app::AppExit, prelude::*};

use crate::shared::{
    DayOfWeek, GameState, PlayerInput, PlayerState, ShiftClock, UpdatePhase, Weather, MAX_FATIGUE,
    MAX_STRESS, SCREEN_HEIGHT, SCREEN_WIDTH,
};

const BAR_MAX_WIDTH: f32 = 200.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), boot_to_main_menu)
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(
                Update,
                handle_main_menu_buttons.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            .add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(Update, toggle_pause.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                update_hud
                    .in_set(UpdatePhase::Presentation)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_hud)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(
                Update,
                handle_pause_buttons.run_if(in_state(GameState::Paused)),
            )
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu);
    }
}

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component, Clone, Copy)]
struct MainMenuButton(MenuAction);

#[derive(Component, Clone, Copy)]
struct PauseMenuButton(PauseAction);

#[derive(Component)]
struct HudClockText;

#[derive(Component)]
struct HudDayText;

#[derive(Component)]
struct HudWeatherText;

#[derive(Component)]
struct HudRankText;

#[derive(Component)]
struct HudDutyText;

#[derive(Component)]
struct HudGoldText;

#[derive(Component)]
struct HudFatigueFill;

#[derive(Component)]
struct HudStressFill;

type HudQuerySet<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<HudClockText>>,
    Query<'w, 's, &'static mut Text, With<HudDayText>>,
    Query<'w, 's, &'static mut Text, With<HudWeatherText>>,
    Query<'w, 's, &'static mut Text, With<HudRankText>>,
    Query<'w, 's, (&'static mut Text, &'static mut TextColor), With<HudDutyText>>,
    Query<'w, 's, &'static mut Text, With<HudGoldText>>,
    Query<'w, 's, (&'static mut Node, &'static mut BackgroundColor), With<HudFatigueFill>>,
    Query<'w, 's, (&'static mut Node, &'static mut BackgroundColor), With<HudStressFill>>,
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    NewGame,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PauseAction {
    Resume,
    QuitToMenu,
}

fn boot_to_main_menu(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
}

fn menu_button_idle_color(action: MenuAction) -> Color {
    match action {
        MenuAction::NewGame => Color::srgb(0.18, 0.34, 0.24),
        MenuAction::Quit => Color::srgb(0.36, 0.18, 0.20),
    }
}

fn menu_button_hover_color(action: MenuAction) -> Color {
    match action {
        MenuAction::NewGame => Color::srgb(0.28, 0.50, 0.36),
        MenuAction::Quit => Color::srgb(0.50, 0.28, 0.30),
    }
}

fn menu_button_pressed_color(action: MenuAction) -> Color {
    match action {
        MenuAction::NewGame => Color::srgb(0.86, 0.78, 0.40),
        MenuAction::Quit => Color::srgb(0.74, 0.58, 0.32),
    }
}

fn pause_button_idle_color(action: PauseAction) -> Color {
    match action {
        PauseAction::Resume => Color::srgb(0.20, 0.42, 0.30),
        PauseAction::QuitToMenu => Color::srgb(0.38, 0.20, 0.22),
    }
}

fn pause_button_hover_color(action: PauseAction) -> Color {
    match action {
        PauseAction::Resume => Color::srgb(0.30, 0.56, 0.40),
        PauseAction::QuitToMenu => Color::srgb(0.52, 0.30, 0.34),
    }
}

fn pause_button_pressed_color(action: PauseAction) -> Color {
    match action {
        PauseAction::Resume => Color::srgb(0.86, 0.78, 0.40),
        PauseAction::QuitToMenu => Color::srgb(0.74, 0.58, 0.32),
    }
}

fn clock_display(clock: &ShiftClock) -> String {
    format!("{:02}:{:02}", clock.hour, clock.minute)
}

fn day_of_week_display(day: DayOfWeek) -> &'static str {
    match day {
        DayOfWeek::Monday => "Monday",
        DayOfWeek::Tuesday => "Tuesday",
        DayOfWeek::Wednesday => "Wednesday",
        DayOfWeek::Thursday => "Thursday",
        DayOfWeek::Friday => "Friday",
        DayOfWeek::Saturday => "Saturday",
        DayOfWeek::Sunday => "Sunday",
    }
}

fn weather_display(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "Clear",
        Weather::Rainy => "Rainy",
        Weather::Foggy => "Foggy",
        Weather::Snowy => "Snowy",
    }
}

fn duty_display(clock: &ShiftClock) -> &'static str {
    if clock.on_duty {
        "ON DUTY"
    } else {
        "OFF DUTY"
    }
}

fn duty_color(clock: &ShiftClock) -> Color {
    if clock.on_duty {
        Color::srgb(0.52, 0.90, 0.56)
    } else {
        Color::srgb(0.92, 0.56, 0.56)
    }
}

fn fatigue_fill_width(fatigue: f32) -> f32 {
    (fatigue.clamp(0.0, MAX_FATIGUE) / MAX_FATIGUE) * BAR_MAX_WIDTH
}

fn stress_fill_width(stress: f32) -> f32 {
    (stress.clamp(0.0, MAX_STRESS) / MAX_STRESS) * BAR_MAX_WIDTH
}

fn fatigue_fill_color(fatigue: f32) -> Color {
    let ratio = (fatigue / MAX_FATIGUE).clamp(0.0, 1.0);
    Color::srgb(1.0 - ratio, 0.25 + (ratio * 0.55), 0.18)
}

fn stress_fill_color(stress: f32) -> Color {
    let ratio = (stress / MAX_STRESS).clamp(0.0, 1.0);
    Color::srgb(
        0.20 + (ratio * 0.75),
        0.25 * (1.0 - ratio),
        0.92 - (ratio * 0.74),
    )
}

fn spawn_main_menu(mut commands: Commands) {
    let button_width = SCREEN_WIDTH * 0.24;
    let button_height = (SCREEN_HEIGHT * 0.10).clamp(48.0, 60.0);

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                padding: UiRect::axes(Val::Px(SCREEN_WIDTH * 0.05), Val::Px(SCREEN_HEIGHT * 0.06)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.05, 0.09)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PRECINCT"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.86, 0.64)),
            ));

            parent.spawn(Node {
                height: Val::Px(12.0),
                ..default()
            });

            for (label, action) in [
                ("New Game", MenuAction::NewGame),
                ("Quit", MenuAction::Quit),
            ] {
                parent
                    .spawn((
                        MainMenuButton(action),
                        Button,
                        Node {
                            width: Val::Px(button_width),
                            height: Val::Px(button_height),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(menu_button_idle_color(action)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn handle_main_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &MainMenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        background.0 = match *interaction {
            Interaction::Pressed => menu_button_pressed_color(button.0),
            Interaction::Hovered => menu_button_hover_color(button.0),
            Interaction::None => menu_button_idle_color(button.0),
        };

        if *interaction == Interaction::Pressed {
            match button.0 {
                MenuAction::NewGame => next_state.set(GameState::Playing),
                MenuAction::Quit => {
                    exit_events.send(AppExit::Success);
                }
            }
        }
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_hud(mut commands: Commands, clock: Res<ShiftClock>, player: Res<PlayerState>) {
    let panel_padding = UiRect::all(Val::Px(10.0));

    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(SCREEN_WIDTH * 0.02), Val::Px(SCREEN_HEIGHT * 0.02)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(5),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(SCREEN_HEIGHT * 0.08),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(18.0),
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.82)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HudClockText,
                        Text::new(clock_display(&clock)),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    bar.spawn((
                        HudDayText,
                        Text::new(day_of_week_display(clock.day_of_week)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.82, 0.84, 0.90)),
                    ));
                    bar.spawn((
                        HudWeatherText,
                        Text::new(weather_display(clock.weather)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.62, 0.80, 0.96)),
                    ));
                    bar.spawn((
                        HudRankText,
                        Text::new(clock.rank.display_name()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.96, 0.84, 0.50)),
                    ));
                    bar.spawn((
                        HudDutyText,
                        Text::new(duty_display(&clock)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(duty_color(&clock)),
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            padding: panel_padding,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.74)),
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new("Fatigue"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.82, 0.86, 0.76)),
                        ));
                        panel
                            .spawn((
                                Node {
                                    width: Val::Px(BAR_MAX_WIDTH),
                                    height: Val::Px(18.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    HudFatigueFill,
                                    Node {
                                        width: Val::Px(fatigue_fill_width(player.fatigue)),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(fatigue_fill_color(player.fatigue)),
                                ));
                            });

                        panel.spawn((
                            Text::new("Stress"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.82, 0.86, 0.76)),
                        ));
                        panel
                            .spawn((
                                Node {
                                    width: Val::Px(BAR_MAX_WIDTH),
                                    height: Val::Px(18.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    HudStressFill,
                                    Node {
                                        width: Val::Px(stress_fill_width(player.stress)),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(stress_fill_color(player.stress)),
                                ));
                            });
                    });

                    row.spawn((
                        Node {
                            padding: panel_padding,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.74)),
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            HudGoldText,
                            Text::new(format!("Gold: ${}", player.gold)),
                            TextFont {
                                font_size: 26.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.96, 0.84, 0.40)),
                        ));
                    });
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn update_hud(
    clock: Res<ShiftClock>,
    player: Res<PlayerState>,
    mut hud_queries: ParamSet<HudQuerySet<'_, '_>>,
) {
    if let Ok(mut text) = hud_queries.p0().get_single_mut() {
        **text = clock_display(&clock);
    }
    if let Ok(mut text) = hud_queries.p1().get_single_mut() {
        **text = day_of_week_display(clock.day_of_week).to_string();
    }
    if let Ok(mut text) = hud_queries.p2().get_single_mut() {
        **text = weather_display(clock.weather).to_string();
    }
    if let Ok(mut text) = hud_queries.p3().get_single_mut() {
        **text = clock.rank.display_name().to_string();
    }
    if let Ok((mut text, mut color)) = hud_queries.p4().get_single_mut() {
        **text = duty_display(&clock).to_string();
        color.0 = duty_color(&clock);
    }
    if let Ok(mut text) = hud_queries.p5().get_single_mut() {
        **text = format!("Gold: ${}", player.gold);
    }
    if let Ok((mut node, mut background)) = hud_queries.p6().get_single_mut() {
        node.width = Val::Px(fatigue_fill_width(player.fatigue));
        background.0 = fatigue_fill_color(player.fatigue);
    }
    if let Ok((mut node, mut background)) = hud_queries.p7().get_single_mut() {
        node.width = Val::Px(stress_fill_width(player.stress));
        background.0 = stress_fill_color(player.stress);
    }
}

fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    input: Res<PlayerInput>,
    mut clock: ResMut<ShiftClock>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || input.cancel || input.menu {
        clock.time_paused = true;
        next_state.set(GameState::Paused);
    }
}

fn spawn_pause_menu(mut commands: Commands) {
    let button_width = SCREEN_WIDTH * 0.24;

    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.74)),
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            for (label, action) in [
                ("Resume", PauseAction::Resume),
                ("Quit to Menu", PauseAction::QuitToMenu),
            ] {
                parent
                    .spawn((
                        PauseMenuButton(action),
                        Button,
                        Node {
                            width: Val::Px(button_width),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(pause_button_idle_color(action)),
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
        });
}

fn handle_pause_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    input: Res<PlayerInput>,
    mut interaction_query: Query<
        (&Interaction, &PauseMenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut clock: ResMut<ShiftClock>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || input.cancel || input.menu {
        clock.time_paused = false;
        next_state.set(GameState::Playing);
        return;
    }

    for (interaction, button, mut background) in &mut interaction_query {
        background.0 = match *interaction {
            Interaction::Pressed => pause_button_pressed_color(button.0),
            Interaction::Hovered => pause_button_hover_color(button.0),
            Interaction::None => pause_button_idle_color(button.0),
        };

        if *interaction == Interaction::Pressed {
            match button.0 {
                PauseAction::Resume => {
                    clock.time_paused = false;
                    next_state.set(GameState::Playing);
                }
                PauseAction::QuitToMenu => {
                    clock.time_paused = false;
                    next_state.set(GameState::MainMenu);
                }
            }
        }
    }
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

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
        app.init_resource::<ShiftClock>()
            .init_resource::<PlayerState>()
            .init_resource::<PlayerInput>()
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_event::<AppExit>()
            .add_plugins(UiPlugin);

        app
    }

    fn set_state(app: &mut App, state: GameState) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        app.update();
    }

    #[test]
    fn loading_boots_into_main_menu() {
        let mut app = build_test_app();

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::MainMenu
        );

        let mut menu_roots = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuRoot>>();
        assert_eq!(menu_roots.iter(app.world()).count(), 1);
    }

    #[test]
    fn main_menu_spawns_title_and_two_buttons() {
        let mut app = build_test_app();
        set_state(&mut app, GameState::MainMenu);

        let mut buttons = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 2);

        let mut title_query = app.world_mut().query::<&Text>();
        let has_title = title_query
            .iter(app.world())
            .any(|text| text.as_str() == "PRECINCT");
        assert!(has_title);
    }

    #[test]
    fn new_game_button_transitions_to_playing_and_cleans_main_menu() {
        let mut app = build_test_app();
        set_state(&mut app, GameState::MainMenu);

        let mut button_query = app.world_mut().query::<(Entity, &MainMenuButton)>();
        let new_game_button = button_query
            .iter(app.world())
            .find_map(|(entity, button)| (button.0 == MenuAction::NewGame).then_some(entity))
            .expect("new game button should exist");

        app.world_mut()
            .entity_mut(new_game_button)
            .insert(Interaction::Pressed);

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Playing
        );

        let mut menu_roots = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuRoot>>();
        let mut hud_roots = app.world_mut().query_filtered::<Entity, With<HudRoot>>();
        assert_eq!(menu_roots.iter(app.world()).count(), 0);
        assert_eq!(hud_roots.iter(app.world()).count(), 1);
    }

    #[test]
    fn quit_button_emits_app_exit() {
        let mut app = build_test_app();
        set_state(&mut app, GameState::MainMenu);

        let mut button_query = app.world_mut().query::<(Entity, &MainMenuButton)>();
        let quit_button = button_query
            .iter(app.world())
            .find_map(|(entity, button)| (button.0 == MenuAction::Quit).then_some(entity))
            .expect("quit button should exist");

        app.world_mut()
            .entity_mut(quit_button)
            .insert(Interaction::Pressed);

        app.update();

        let events = app.world().resource::<Events<AppExit>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);
    }

    #[test]
    fn hud_spawns_and_reflects_player_and_clock_values() {
        let mut app = build_test_app();
        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.hour = 6;
            clock.minute = 0;
            clock.day_of_week = DayOfWeek::Friday;
            clock.weather = Weather::Rainy;
            clock.on_duty = true;
        }
        {
            let mut player = app.world_mut().resource_mut::<PlayerState>();
            player.fatigue = 75.0;
            player.stress = 40.0;
            player.gold = 275;
        }

        set_state(&mut app, GameState::Playing);
        app.update();

        let mut fatigue_query = app
            .world_mut()
            .query_filtered::<&Node, With<HudFatigueFill>>();
        let fatigue_node = fatigue_query.single(app.world());
        match fatigue_node.width {
            Val::Px(width) => assert!((width - 150.0).abs() < 0.001),
            other => panic!("expected fatigue width in pixels, got {other:?}"),
        }

        let mut stress_query = app
            .world_mut()
            .query_filtered::<&Node, With<HudStressFill>>();
        let stress_node = stress_query.single(app.world());
        match stress_node.width {
            Val::Px(width) => assert!((width - 80.0).abs() < 0.001),
            other => panic!("expected stress width in pixels, got {other:?}"),
        }

        let mut clock_query = app
            .world_mut()
            .query_filtered::<&Text, With<HudClockText>>();
        let clock_text = clock_query.single(app.world());
        assert_eq!(clock_text.as_str(), "06:00");

        let mut gold_query = app.world_mut().query_filtered::<&Text, With<HudGoldText>>();
        let gold_text = gold_query.single(app.world());
        assert_eq!(gold_text.as_str(), "Gold: $275");
    }

    #[test]
    fn escape_pauses_game_sets_time_paused_and_swaps_screens() {
        let mut app = build_test_app();
        set_state(&mut app, GameState::Playing);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.update();

        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Paused
        );
        assert!(app.world().resource::<ShiftClock>().time_paused);

        let mut hud_roots = app.world_mut().query_filtered::<Entity, With<HudRoot>>();
        let mut pause_roots = app
            .world_mut()
            .query_filtered::<Entity, With<PauseMenuRoot>>();
        assert_eq!(hud_roots.iter(app.world()).count(), 0);
        assert_eq!(pause_roots.iter(app.world()).count(), 1);
    }

    #[test]
    fn resume_button_returns_to_playing_and_unpauses_clock() {
        let mut app = build_test_app();
        app.world_mut().resource_mut::<ShiftClock>().time_paused = true;
        set_state(&mut app, GameState::Paused);

        let mut button_query = app.world_mut().query::<(Entity, &PauseMenuButton)>();
        let resume_button = button_query
            .iter(app.world())
            .find_map(|(entity, button)| (button.0 == PauseAction::Resume).then_some(entity))
            .expect("resume button should exist");

        app.world_mut()
            .entity_mut(resume_button)
            .insert(Interaction::Pressed);

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Playing
        );
        assert!(!app.world().resource::<ShiftClock>().time_paused);

        let mut pause_roots = app
            .world_mut()
            .query_filtered::<Entity, With<PauseMenuRoot>>();
        let mut hud_roots = app.world_mut().query_filtered::<Entity, With<HudRoot>>();
        assert_eq!(pause_roots.iter(app.world()).count(), 0);
        assert_eq!(hud_roots.iter(app.world()).count(), 1);
    }
}
