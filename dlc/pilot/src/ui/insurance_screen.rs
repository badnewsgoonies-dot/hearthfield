//! Insurance Office screen — view policy, buy/upgrade coverage, claims history.

use crate::economy::insurance::{CoverageType, InsuranceState};
use crate::shared::*;
use bevy::prelude::*;

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct InsuranceScreenRoot;

#[derive(Component)]
pub struct BuyCoverageButton {
    pub coverage: CoverageType,
}

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_insurance_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    insurance: Res<InsuranceState>,
    fleet: Res<Fleet>,
) {
    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    let root = commands
        .spawn((
            InsuranceScreenRoot,
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
            Text::new("INSURANCE OFFICE"),
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

    // Current policy for active aircraft
    let policy_text = if let Some(aircraft) = fleet.active() {
        if let Some(policy) = insurance.policy_for(&aircraft.aircraft_id) {
            format!(
                "Current policy for {}: {} | Premium: {}g/mo | Deductible: {}g | Limit: {}g",
                aircraft.nickname,
                policy.coverage_type.display_name(),
                policy.premium,
                policy.deductible,
                policy.coverage_limit,
            )
        } else {
            format!("{} is UNINSURED!", aircraft.nickname)
        }
    } else {
        "No active aircraft.".to_string()
    };

    let policy_label = commands
        .spawn((
            Text::new(policy_text),
            text_style.clone(),
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(policy_label);

    // Monthly total
    let total_label = commands
        .spawn((
            Text::new(format!(
                "Total monthly premium: {}g (multiplier: {:.2}x)",
                insurance.total_monthly_premium(),
                insurance.premium_multiplier.max(1.0),
            )),
            text_style.clone(),
            TextColor(Color::srgb(0.7, 0.85, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(total_label);

    // Buy/upgrade buttons
    let section_label = commands
        .spawn((
            Text::new("── Buy / Upgrade Coverage ──"),
            TextFont {
                font: font.0.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(section_label);

    for coverage in [
        CoverageType::Basic,
        CoverageType::Standard,
        CoverageType::Premium,
    ] {
        let label = format!(
            "{} — {}g/mo base | Deductible: {}g | Limit: {}g",
            coverage.display_name(),
            coverage.base_premium(),
            coverage.deductible(),
            coverage.coverage_limit(),
        );

        let btn = commands
            .spawn((
                BuyCoverageButton { coverage },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.2, 0.3)),
            ))
            .id();
        let btn_text = commands
            .spawn((
                Text::new(label),
                text_style.clone(),
                TextColor(Color::WHITE),
            ))
            .id();
        commands.entity(btn).add_child(btn_text);
        commands.entity(root).add_child(btn);
    }

    // Claims history
    let claims_header = commands
        .spawn((
            Text::new(format!(
                "── Claims History ({} total) ──",
                insurance.total_claims_filed
            )),
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
    commands.entity(root).add_child(claims_header);

    for claim in insurance.claims.iter().rev().take(5) {
        let line = format!(
            "Day {} | {} | Damage: {}g | Payout: {}g",
            claim.day_filed, claim.reason, claim.damage_amount, claim.payout,
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

    if insurance.claims.is_empty() {
        let empty = commands
            .spawn((
                Text::new("No claims filed. Keep flying safe!"),
                text_style.clone(),
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
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

pub fn despawn_insurance_screen(
    mut commands: Commands,
    query: Query<Entity, With<InsuranceScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Input ───────────────────────────────────────────────────────────────

pub fn handle_insurance_input(
    input: Res<PlayerInput>,
    interaction_q: Query<(&Interaction, &BuyCoverageButton), Changed<Interaction>>,
    mut purchase_events: EventWriter<PurchaseEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.cancel || input.menu_cancel {
        next_state.set(GameState::Playing);
        return;
    }

    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            let item_id = match btn.coverage {
                CoverageType::Basic => "insurance_basic",
                CoverageType::Standard => "insurance_standard",
                CoverageType::Premium => "insurance_premium",
            };
            purchase_events.send(PurchaseEvent {
                item_id: item_id.to_string(),
                price: btn.coverage.base_premium(),
            });
        }
    }
}
