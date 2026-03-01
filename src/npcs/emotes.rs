//! Floating emote bubbles above NPCs — uses Sprout Lands emotes.png.
//!
//! When an NPC reacts (gift, dialogue, etc.), a small emote sprite
//! appears above their head, floats upward, and fades out.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// EMOTE ATLAS
// ═══════════════════════════════════════════════════════════════════════

/// Lazily-loaded atlas for NPC emote sprites.
/// emotes.png: 160×480px → 10 cols × 30 rows of 16×16 tiles (300 frames).
#[derive(Resource, Default)]
pub struct EmoteAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// EMOTE KINDS & INDEX MAPPING
// ═══════════════════════════════════════════════════════════════════════

/// The kind of emote to display above an NPC.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum EmoteKind {
    Heart,       // loved gift
    Happy,       // liked gift
    Neutral,     // neutral gift
    Sad,         // disliked gift
    Angry,       // hated gift
    Exclamation, // quest complete, surprise
    Question,    // confused
}

impl EmoteKind {
    /// Map to an atlas index in emotes.png.
    /// These are educated guesses for a standard Sprout Lands emote sheet.
    pub fn atlas_index(self) -> usize {
        match self {
            EmoteKind::Heart => 0,        // row 0, col 0 — heart
            EmoteKind::Happy => 10,       // row 1, col 0 — smile
            EmoteKind::Neutral => 30,     // row 3, col 0 — dots/neutral
            EmoteKind::Sad => 40,         // row 4, col 0 — sad
            EmoteKind::Angry => 50,       // row 5, col 0 — angry
            EmoteKind::Exclamation => 20, // row 2, col 0 — exclamation
            EmoteKind::Question => 60,    // row 6, col 0 — question
        }
    }
}

impl From<GiftPreference> for EmoteKind {
    fn from(pref: GiftPreference) -> Self {
        match pref {
            GiftPreference::Loved => EmoteKind::Heart,
            GiftPreference::Liked => EmoteKind::Happy,
            GiftPreference::Neutral => EmoteKind::Neutral,
            GiftPreference::Disliked => EmoteKind::Sad,
            GiftPreference::Hated => EmoteKind::Angry,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// EMOTE EVENT & COMPONENT
// ═══════════════════════════════════════════════════════════════════════

/// Fire this to show an emote bubble over an NPC.
#[derive(Event, Debug)]
pub struct NpcEmoteEvent {
    pub npc_id: String,
    pub emote: EmoteKind,
}

/// Component on the floating emote sprite entity.
#[derive(Component)]
pub struct EmoteBubble {
    pub timer: Timer,
    pub start_y: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn emote bubble sprites in response to NpcEmoteEvent.
pub fn spawn_emote_bubbles(
    mut commands: Commands,
    mut events: EventReader<NpcEmoteEvent>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas: ResMut<EmoteAtlas>,
    npc_query: Query<(&Npc, &Transform)>,
) {
    // Lazy-load the emotes atlas
    if !atlas.loaded {
        atlas.image = asset_server.load("ui/emotes.png");
        atlas.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            10,
            30,
            None,
            None,
        ));
        atlas.loaded = true;
    }

    for event in events.read() {
        // Find the NPC's current position
        let Some((_npc, transform)) = npc_query
            .iter()
            .find(|(npc, _)| npc.id == event.npc_id)
        else {
            continue;
        };

        let npc_pos = transform.translation;
        let emote_y = npc_pos.y + 20.0; // above head

        let mut sprite = Sprite::from_atlas_image(
            atlas.image.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: event.emote.atlas_index(),
            },
        );
        sprite.custom_size = Some(Vec2::splat(12.0));

        commands.spawn((
            EmoteBubble {
                timer: Timer::from_seconds(1.5, TimerMode::Once),
                start_y: emote_y,
            },
            sprite,
            Transform::from_xyz(npc_pos.x, emote_y, Z_ENTITY_BASE + 50.0),
            Visibility::default(),
        ));
    }
}

/// Animate emote bubbles: float upward and fade out, then despawn.
pub fn animate_emote_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EmoteBubble, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut bubble, mut transform, mut sprite) in &mut query {
        bubble.timer.tick(time.delta());

        let progress = bubble.timer.fraction(); // 0.0 → 1.0

        // Float upward
        transform.translation.y = bubble.start_y + progress * 12.0;

        // Fade out in last 30%
        let alpha = if progress > 0.7 {
            1.0 - (progress - 0.7) / 0.3
        } else {
            1.0
        };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);

        if bubble.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
