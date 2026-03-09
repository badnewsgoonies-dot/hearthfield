//! Floating emote bubbles above NPCs — procedural colored sprites.
//!
//! When an NPC reacts (gift, dialogue, etc.), a small emote sprite
//! appears above their head, floats upward, and fades out.

use crate::shared::*;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// ═══════════════════════════════════════════════════════════════════════
// EMOTE SPRITE CACHE
// ═══════════════════════════════════════════════════════════════════════

/// Cached procedural emote sprite handles (generated once, reused).
#[derive(Resource, Default)]
pub struct EmoteSprites {
    pub sprites: Vec<(EmoteKind, Handle<Image>)>,
    pub loaded: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// EMOTE KINDS
// ═══════════════════════════════════════════════════════════════════════

/// The kind of emote to display above an NPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Color for each emote type (recognizable at a glance).
    pub fn color(self) -> Color {
        match self {
            EmoteKind::Heart => Color::srgb(0.9, 0.15, 0.25), // red
            EmoteKind::Happy => Color::srgb(1.0, 0.85, 0.2),  // yellow
            EmoteKind::Neutral => Color::srgb(0.7, 0.7, 0.7), // gray
            EmoteKind::Sad => Color::srgb(0.3, 0.5, 0.85),    // blue
            EmoteKind::Angry => Color::srgb(0.85, 0.2, 0.1),  // dark red
            EmoteKind::Exclamation => Color::srgb(1.0, 0.65, 0.0), // orange
            EmoteKind::Question => Color::srgb(0.4, 0.75, 1.0), // light blue
        }
    }
}

/// Generate an 8x8 procedural emote image.
fn make_emote_image(kind: EmoteKind) -> Image {
    let w = 8usize;
    let h = 8usize;
    let mut data = vec![0u8; w * h * 4];
    let c = kind.color().to_srgba();
    let r = (c.red * 255.0) as u8;
    let g = (c.green * 255.0) as u8;
    let b = (c.blue * 255.0) as u8;

    // Draw a simple shape based on emote type
    let pattern: [[u8; 8]; 8] = match kind {
        EmoteKind::Heart => [
            [0, 1, 1, 0, 0, 1, 1, 0],
            [1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 0, 1, 1, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        EmoteKind::Happy => [
            [0, 0, 1, 1, 1, 1, 0, 0],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [1, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 0, 1, 1, 0, 0, 1],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [0, 0, 1, 1, 1, 1, 0, 0],
        ],
        EmoteKind::Sad => [
            [0, 0, 1, 1, 1, 1, 0, 0],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [1, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 1, 1, 0, 0, 1],
            [1, 0, 1, 0, 0, 1, 0, 1],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [0, 0, 1, 1, 1, 1, 0, 0],
        ],
        EmoteKind::Angry => [
            [1, 0, 0, 0, 0, 0, 0, 1],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [0, 0, 1, 0, 0, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 1, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
        ],
        EmoteKind::Exclamation => [
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
        ],
        EmoteKind::Question => [
            [0, 0, 1, 1, 1, 0, 0, 0],
            [0, 1, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0, 0],
        ],
        EmoteKind::Neutral => [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
    };

    for (py, row) in pattern.iter().enumerate() {
        for (px, &pixel) in row.iter().enumerate() {
            let i = (py * w + px) * 4;
            if pixel == 1 {
                data[i] = r;
                data[i + 1] = g;
                data[i + 2] = b;
                data[i + 3] = 255;
            }
        }
    }

    let mut img = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img
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
/// Uses procedural pixel-art sprites — no atlas dependency.
pub fn spawn_emote_bubbles(
    mut commands: Commands,
    mut events: EventReader<NpcEmoteEvent>,
    mut images: ResMut<Assets<Image>>,
    mut emote_sprites: ResMut<EmoteSprites>,
    npc_query: Query<(&Npc, &Transform)>,
) {
    // Generate emote images once
    if !emote_sprites.loaded {
        let kinds = [
            EmoteKind::Heart,
            EmoteKind::Happy,
            EmoteKind::Neutral,
            EmoteKind::Sad,
            EmoteKind::Angry,
            EmoteKind::Exclamation,
            EmoteKind::Question,
        ];
        for kind in kinds {
            let handle = images.add(make_emote_image(kind));
            emote_sprites.sprites.push((kind, handle));
        }
        emote_sprites.loaded = true;
    }

    for event in events.read() {
        // Find the NPC's current position
        let Some((_npc, transform)) = npc_query.iter().find(|(npc, _)| npc.id == event.npc_id)
        else {
            continue;
        };

        let npc_pos = transform.translation;
        let emote_y = npc_pos.y + 20.0; // above head

        let image_handle = emote_sprites
            .sprites
            .iter()
            .find(|(k, _)| *k == event.emote)
            .map(|(_, h)| h.clone())
            .unwrap_or_default();

        let mut sprite = Sprite::from_image(image_handle);
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
