//! Lifeline::domains::save — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Persister;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SaveSlot(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct SaveBuffer {
    pub entries: Vec<String>,
}

impl SaveBuffer {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SaveRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl SaveRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct SaveRegistry {
    pub defs: std::collections::HashMap<String, SaveRegistryDef>,
}

impl SaveRegistry {
    pub fn get(&self, id: &str) -> Option<&SaveRegistryDef> {
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
pub struct SaveQueuedEvent {
    pub save_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct SaveCompleteEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct SaveToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_save_marker(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads SaveQueuedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn queue_save(
    mut commands: Commands,
    mut pickup_events: EventReader<SaveQueuedEvent>,
    mut inventory: ResMut<SaveBuffer>,
    item_registry: Res<SaveRegistry>,
    mut sfx_events: EventWriter<SaveCompleteEvent>,
    mut toast_events: EventWriter<SaveToastEvent>,
    player_query: Query<&SaveSlot, With<Persister>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.save_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.save_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(SaveCompleteEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_save_marker(&mut commands, world_pos);
            }
            info!("[Persister] Picked up {} × '{}'", ev.quantity, ev.save_id);
        } else {
            let name = item_registry
                .get(&ev.save_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.save_id);
            toast_events.send(SaveToastEvent {
                message: format!("SaveBuffer full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Persister] SaveBuffer full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.save_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveBuffer>();
        app.init_resource::<SaveRegistry>();
        app.add_event::<SaveQueuedEvent>();
        app.add_event::<SaveCompleteEvent>();
        app.add_event::<SaveToastEvent>();
        app.add_systems(Update, queue_save);
    }
}
