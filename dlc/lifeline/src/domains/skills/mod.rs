//! Lifeline::domains::skills — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Trainee;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TrainingPosition(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct SkillTree {
    pub entries: Vec<String>,
}

impl SkillTree {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SkillCatalogDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl SkillCatalogDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct SkillCatalog {
    pub defs: std::collections::HashMap<String, SkillCatalogDef>,
}

impl SkillCatalog {
    pub fn get(&self, id: &str) -> Option<&SkillCatalogDef> {
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
pub struct SkillUnlockedEvent {
    pub skill_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct LevelUpEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct SkillsToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_unlock_indicator(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads SkillUnlockedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn unlock_skill(
    mut commands: Commands,
    mut pickup_events: EventReader<SkillUnlockedEvent>,
    mut inventory: ResMut<SkillTree>,
    item_registry: Res<SkillCatalog>,
    mut sfx_events: EventWriter<LevelUpEvent>,
    mut toast_events: EventWriter<SkillsToastEvent>,
    player_query: Query<&TrainingPosition, With<Trainee>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.skill_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.skill_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(LevelUpEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_unlock_indicator(&mut commands, world_pos);
            }
            info!("[Trainee] Picked up {} × '{}'", ev.quantity, ev.skill_id);
        } else {
            let name = item_registry
                .get(&ev.skill_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.skill_id);
            toast_events.send(SkillsToastEvent {
                message: format!("SkillTree full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Trainee] SkillTree full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.skill_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillTree>();
        app.init_resource::<SkillCatalog>();
        app.add_event::<SkillUnlockedEvent>();
        app.add_event::<LevelUpEvent>();
        app.add_event::<SkillsToastEvent>();
        app.add_systems(Update, unlock_skill);
    }
}
