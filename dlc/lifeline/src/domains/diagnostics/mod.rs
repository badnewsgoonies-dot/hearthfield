//! Lifeline::domains::diagnostics — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Technician;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TestStation(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct DiagnosticPanel {
    pub entries: Vec<String>,
}

impl DiagnosticPanel {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl TestRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct TestRegistry {
    pub defs: std::collections::HashMap<String, TestRegistryDef>,
}

impl TestRegistry {
    pub fn get(&self, id: &str) -> Option<&TestRegistryDef> {
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
pub struct TestOrderedEvent {
    pub test_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct TestCompleteEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct DiagnosticsToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_test_indicator(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads TestOrderedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn order_test(
    mut commands: Commands,
    mut pickup_events: EventReader<TestOrderedEvent>,
    mut inventory: ResMut<DiagnosticPanel>,
    item_registry: Res<TestRegistry>,
    mut sfx_events: EventWriter<TestCompleteEvent>,
    mut toast_events: EventWriter<DiagnosticsToastEvent>,
    player_query: Query<&TestStation, With<Technician>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.test_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.test_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(TestCompleteEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_test_indicator(&mut commands, world_pos);
            }
            info!("[Technician] Picked up {} × '{}'", ev.quantity, ev.test_id);
        } else {
            let name = item_registry
                .get(&ev.test_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.test_id);
            toast_events.send(DiagnosticsToastEvent {
                message: format!("DiagnosticPanel full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Technician] DiagnosticPanel full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.test_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticPanel>();
        app.init_resource::<TestRegistry>();
        app.add_event::<TestOrderedEvent>();
        app.add_event::<TestCompleteEvent>();
        app.add_event::<DiagnosticsToastEvent>();
        app.add_systems(Update, order_test);
    }
}
