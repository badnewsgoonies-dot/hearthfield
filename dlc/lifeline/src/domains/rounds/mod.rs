//! Lifeline::domains::rounds — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Rounder;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct BedPosition(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct RoundsQueue {
    pub entries: Vec<String>,
}

impl RoundsQueue {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PatientChartRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl PatientChartRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct PatientChartRegistry {
    pub defs: std::collections::HashMap<String, PatientChartRegistryDef>,
}

impl PatientChartRegistry {
    pub fn get(&self, id: &str) -> Option<&PatientChartRegistryDef> {
        self.defs.get(id)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StackSize(pub u8);
impl StackSize {
    pub fn get(self) -> u8 { self.0 }
}

// ── events ────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone, Default)]
pub struct PatientVisitedEvent {
    pub patient_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct RoundsBellEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct RoundsToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_visit_indicator(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads PatientVisitedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn visit_patient(
    mut commands: Commands,
    mut pickup_events: EventReader<PatientVisitedEvent>,
    mut inventory: ResMut<RoundsQueue>,
    item_registry: Res<PatientChartRegistry>,
    mut sfx_events: EventWriter<RoundsBellEvent>,
    mut toast_events: EventWriter<RoundsToastEvent>,
    player_query: Query<&BedPosition, With<Rounder>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.patient_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.patient_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(RoundsBellEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_visit_indicator(&mut commands, world_pos);
            }
            info!("[Rounder] Picked up {} × '{}'", ev.quantity, ev.patient_id);
        } else {
            let name = item_registry
                .get(&ev.patient_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.patient_id);
            toast_events.send(RoundsToastEvent {
                message: format!("RoundsQueue full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Rounder] RoundsQueue full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.patient_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoundsQueue>();
        app.init_resource::<PatientChartRegistry>();
        app.add_event::<PatientVisitedEvent>();
        app.add_event::<RoundsBellEvent>();
        app.add_event::<RoundsToastEvent>();
        app.add_systems(Update, visit_patient);
    }
}
