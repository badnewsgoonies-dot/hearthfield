//! Lifeline::domains::world — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct WardManager;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct WardCoord(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct WardObjectStore {
    pub entries: Vec<String>,
}

impl WardObjectStore {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WardObjectRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl WardObjectRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct WardObjectRegistry {
    pub defs: std::collections::HashMap<String, WardObjectRegistryDef>,
}

impl WardObjectRegistry {
    pub fn get(&self, id: &str) -> Option<&WardObjectRegistryDef> {
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
pub struct WardObjectAddedEvent {
    pub object_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct WardChimeEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct WorldToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_world_marker(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads WardObjectAddedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn add_ward_object(
    mut commands: Commands,
    mut pickup_events: EventReader<WardObjectAddedEvent>,
    mut inventory: ResMut<WardObjectStore>,
    item_registry: Res<WardObjectRegistry>,
    mut sfx_events: EventWriter<WardChimeEvent>,
    mut toast_events: EventWriter<WorldToastEvent>,
    player_query: Query<&WardCoord, With<WardManager>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.object_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.object_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(WardChimeEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_world_marker(&mut commands, world_pos);
            }
            info!("[WardManager] Picked up {} × '{}'", ev.quantity, ev.object_id);
        } else {
            let name = item_registry
                .get(&ev.object_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.object_id);
            toast_events.send(WorldToastEvent {
                message: format!("WardObjectStore full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[WardManager] WardObjectStore full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.object_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WardObjectStore>();
        app.init_resource::<WardObjectRegistry>();
        app.add_event::<WardObjectAddedEvent>();
        app.add_event::<WardChimeEvent>();
        app.add_event::<WorldToastEvent>();
        app.add_systems(Update, add_ward_object);
    }
}
