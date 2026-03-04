//! Loan Office screen — view active loans, take new loan, early payoff.

use bevy::prelude::*;
use crate::shared::*;
use crate::economy::loans::{LoanPortfolio, interest_rate_for_rank};

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct LoanScreenRoot;

#[derive(Component)]
pub struct TakeLoanButton;

#[derive(Component)]
pub struct PayOffButton {
    pub loan_index: usize,
}

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_loan_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    portfolio: Res<LoanPortfolio>,
    pilot: Res<PilotState>,
) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    let root = commands
        .spawn((
            LoanScreenRoot,
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
            Text::new("LOAN OFFICE"),
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

    // Interest rate info
    let rate = interest_rate_for_rank(pilot.rank);
    let rate_label = commands
        .spawn((
            Text::new(format!(
                "Your rate ({}): {:.0}% APR",
                pilot.rank.display_name(),
                rate * 100.0
            )),
            text_style.clone(),
            TextColor(Color::srgb(0.7, 0.85, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(rate_label);

    // Summary
    let summary = commands
        .spawn((
            Text::new(format!(
                "Active loans: {} | Total debt: {}g | Monthly payment: {}g",
                portfolio.active_loan_count(),
                portfolio.total_debt(),
                portfolio.total_monthly_payment(),
            )),
            text_style.clone(),
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(summary);

    // Active loans list
    for (i, loan) in portfolio.loans.iter().enumerate() {
        if !loan.active {
            continue;
        }
        let line = format!(
            "{} — Balance: {}g | Payment: {}g/mo | Rate: {:.0}% | {}/{}mo",
            loan.aircraft_id,
            loan.remaining_balance,
            loan.monthly_payment,
            loan.interest_rate * 100.0,
            loan.months_paid,
            loan.term_months,
        );
        let entry = commands
            .spawn((
                Text::new(line),
                text_style.clone(),
                TextColor(Color::srgb(0.9, 0.9, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(entry);

        // Pay Off button
        let payoff_text = format!("Pay Off ({}g)", loan.early_payoff_amount());
        let btn = commands
            .spawn((
                PayOffButton { loan_index: i },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.3, 0.15)),
            ))
            .id();
        let btn_text = commands
            .spawn((
                Text::new(payoff_text),
                TextFont {
                    font: font.0.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 1.0, 0.6)),
            ))
            .id();
        commands.entity(btn).add_child(btn_text);
        commands.entity(root).add_child(btn);
    }

    if portfolio.active_loan_count() == 0 {
        let empty = commands
            .spawn((
                Text::new("No active loans."),
                text_style.clone(),
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(empty);
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

pub fn despawn_loan_screen(
    mut commands: Commands,
    query: Query<Entity, With<LoanScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Input ───────────────────────────────────────────────────────────────

pub fn handle_loan_input(
    input: Res<PlayerInput>,
    interaction_q: Query<(&Interaction, &PayOffButton), Changed<Interaction>>,
    portfolio: Res<LoanPortfolio>,
    mut purchase_events: EventWriter<PurchaseEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.cancel || input.menu_cancel {
        next_state.set(GameState::Playing);
        return;
    }

    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            if let Some(loan) = portfolio.loans.get(btn.loan_index) {
                if loan.active {
                    purchase_events.send(PurchaseEvent {
                        item_id: format!("payoff_{}", loan.aircraft_id),
                        price: loan.early_payoff_amount(),
                    });
                }
            }
        }
    }
}
