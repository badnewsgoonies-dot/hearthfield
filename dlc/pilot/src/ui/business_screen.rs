//! Business HQ screen — airline overview, routes, fleet, employees, revenue.

use crate::economy::business::AirlineBusiness;
use crate::shared::*;
use bevy::prelude::*;

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct BusinessScreenRoot;

#[derive(Component)]
pub struct HireButton;

#[derive(Component)]
pub struct AddRouteButton;

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_business_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    business: Res<AirlineBusiness>,
) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    let root = commands
        .spawn((
            BusinessScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new("BUSINESS HQ"),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.3)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    if !business.established {
        let msg = commands
            .spawn((
                Text::new("Airline not established yet. Reach Captain rank to start your airline!"),
                text_style.clone(),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(msg);
    } else {
        // Airline name & hub
        let name_label = commands
            .spawn((
                Text::new(format!(
                    "{} — Hub: {}",
                    business.name,
                    business.hub_airport.display_name()
                )),
                TextFont {
                    font: font.0.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(name_label);

        // Overview stats
        let overview_lines = vec![
            format!("Reputation: {:.0}%", business.reputation),
            format!("Fleet size: {}", business.fleet_size),
            format!("Employees: {}", business.employee_count),
            format!(
                "Monthly revenue: {}g | Expenses: {}g | Profit: {}g",
                business.monthly_revenue,
                business.monthly_expenses,
                business.monthly_profit(),
            ),
        ];

        for line in &overview_lines {
            let child = commands
                .spawn((
                    Text::new(line.clone()),
                    text_style.clone(),
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(3.0)),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(child);
        }

        // ── Routes ────────────────────────────────────────────────
        let routes_header = commands
            .spawn((
                Text::new(format!("── Routes ({}) ──", business.active_routes.len())),
                TextFont {
                    font: font.0.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(6.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(routes_header);

        for route in &business.active_routes {
            let line = format!(
                "{} -> {} | Ticket: {}g | Pax/day: {} | Profit/day: {}g",
                route.origin.display_name(),
                route.destination.display_name(),
                route.ticket_price,
                route.daily_passengers,
                route.daily_profit(),
            );
            let entry = commands
                .spawn((
                    Text::new(line),
                    text_style.clone(),
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                    Node {
                        margin: UiRect::bottom(Val::Px(3.0)),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(entry);
        }

        if business.active_routes.is_empty() {
            let empty = commands
                .spawn((
                    Text::new("No active routes yet."),
                    text_style.clone(),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ))
                .id();
            commands.entity(root).add_child(empty);
        }

        // ── Employees ─────────────────────────────────────────────
        let emp_header = commands
            .spawn((
                Text::new(format!("── Employees ({}) ──", business.hired_pilots.len())),
                TextFont {
                    font: font.0.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(6.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(emp_header);

        for pilot in &business.hired_pilots {
            let route_str = pilot.assigned_route.as_deref().unwrap_or("Unassigned");
            let line = format!(
                "{} | Skill: {:.0}% | Salary: {}g | Morale: {:.0}% | {}",
                pilot.name,
                pilot.skill * 100.0,
                pilot.salary,
                pilot.morale,
                route_str,
            );
            let entry = commands
                .spawn((
                    Text::new(line),
                    text_style.clone(),
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                    Node {
                        margin: UiRect::bottom(Val::Px(3.0)),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(entry);
        }

        if business.hired_pilots.is_empty() {
            let empty = commands
                .spawn((
                    Text::new("No employees yet."),
                    text_style.clone(),
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ))
                .id();
            commands.entity(root).add_child(empty);
        }

        // Placeholder buttons
        let btn_row = commands
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(16.0)),
                ..default()
            })
            .id();
        commands.entity(root).add_child(btn_row);

        let hire_btn = commands
            .spawn((
                HireButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.25, 0.15)),
            ))
            .id();
        let hire_text = commands
            .spawn((
                Text::new("Hire Pilot"),
                text_style.clone(),
                TextColor(Color::srgb(0.6, 1.0, 0.6)),
            ))
            .id();
        commands.entity(hire_btn).add_child(hire_text);
        commands.entity(btn_row).add_child(hire_btn);

        let route_btn = commands
            .spawn((
                AddRouteButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.2, 0.3)),
            ))
            .id();
        let route_text = commands
            .spawn((
                Text::new("Add Route"),
                text_style.clone(),
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
            ))
            .id();
        commands.entity(route_btn).add_child(route_text);
        commands.entity(btn_row).add_child(route_btn);

        // Milestones
        if !business.milestones_reached.is_empty() {
            let mile_header = commands
                .spawn((
                    Text::new("── Milestones ──"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.8, 1.0)),
                    Node {
                        margin: UiRect::new(
                            Val::Px(0.0),
                            Val::Px(0.0),
                            Val::Px(12.0),
                            Val::Px(6.0),
                        ),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(mile_header);

            for milestone in &business.milestones_reached {
                let entry = commands
                    .spawn((
                        Text::new(format!("* {}", milestone.display_name())),
                        text_style.clone(),
                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(root).add_child(entry);
            }
        }
    }

    // Back hint
    let hint = commands
        .spawn((
            Text::new("[Esc] Back"),
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

pub fn despawn_business_screen(
    mut commands: Commands,
    query: Query<Entity, With<BusinessScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Input ───────────────────────────────────────────────────────────────

pub fn handle_business_input(
    input: Res<PlayerInput>,
    hire_q: Query<&Interaction, (Changed<Interaction>, With<HireButton>)>,
    route_q: Query<&Interaction, (Changed<Interaction>, With<AddRouteButton>)>,
    mut toast_events: EventWriter<ToastEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.cancel || input.menu_cancel {
        next_state.set(GameState::Playing);
        return;
    }

    for interaction in &hire_q {
        if *interaction == Interaction::Pressed {
            toast_events.send(ToastEvent {
                message: "Hiring coming soon!".to_string(),
                duration_secs: 3.0,
            });
        }
    }

    for interaction in &route_q {
        if *interaction == Interaction::Pressed {
            toast_events.send(ToastEvent {
                message: "Route planning coming soon!".to_string(),
                duration_secs: 3.0,
            });
        }
    }
}
