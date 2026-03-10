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


/// Helper: write an RGBA pixel into the data buffer at (x, y) for a 16-wide image.
fn put_pixel(data: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
    let i = (y * 16 + x) * 4;
    if i + 3 < data.len() {
        data[i] = r;
        data[i + 1] = g;
        data[i + 2] = b;
        data[i + 3] = a;
    }
}

/// Generate a 16x16 procedural emote image with multi-color pixel art.
///
/// Each emote uses a palette of 2-3 colors (fill, outline/detail, highlight)
/// for recognizable, expressive icons at small scale.
fn make_emote_image(kind: EmoteKind) -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];

    // Each pixel in the pattern maps to a color index:
    //   0 = transparent
    //   1 = primary fill color
    //   2 = secondary / outline color
    //   3 = highlight / accent color
    //
    // Each emote defines its own palette for these indices.

    type Pattern = [[u8; 16]; 16];

    let (pattern, palette): (Pattern, [(u8, u8, u8, u8); 4]) = match kind {
        // ── Heart: red fill, pink outline, white highlight ──
        EmoteKind::Heart => (
            [
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0],
                [0, 0, 2, 1, 1, 2, 0, 0, 0, 2, 1, 1, 2, 0, 0, 0],
                [0, 2, 3, 1, 1, 1, 2, 0, 2, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 3, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0],
                [0, 0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0],
                [0, 0, 0, 0, 2, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),         // 0: transparent
                (220, 40, 60, 255),   // 1: red fill
                (255, 140, 160, 255), // 2: pink outline
                (255, 220, 230, 255), // 3: white-pink highlight
            ],
        ),

        // ── Happy: yellow circle, dark eyes, curved smile ──
        EmoteKind::Happy => (
            [
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2],
                [0, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 2, 2, 2, 2, 2, 2, 1, 1, 1, 2, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),        // 0: transparent
                (255, 220, 50, 255), // 1: yellow fill
                (100, 70, 30, 255),  // 2: dark brown outlines/features
                (0, 0, 0, 0),        // 3: unused
            ],
        ),

        // ── Neutral: gray circle, flat mouth, dot eyes ──
        EmoteKind::Neutral => (
            [
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [0, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),         // 0: transparent
                (180, 180, 180, 255), // 1: gray fill
                (80, 80, 80, 255),    // 2: dark gray outlines/features
                (0, 0, 0, 0),         // 3: unused
            ],
        ),

        // ── Sad: blue circle, inverted smile, teardrop ──
        EmoteKind::Sad => (
            [
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 2],
                [0, 2, 1, 1, 1, 2, 2, 2, 2, 2, 2, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),         // 0: transparent
                (100, 140, 210, 255), // 1: blue fill
                (40, 60, 100, 255),   // 2: dark blue outline/features
                (140, 200, 255, 255), // 3: light blue teardrop
            ],
        ),

        // ── Angry: red circle, angular brows, gritted teeth ──
        EmoteKind::Angry => (
            [
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 2, 0],
                [0, 2, 1, 1, 2, 2, 1, 1, 1, 2, 2, 1, 1, 1, 2, 0],
                [2, 1, 1, 1, 1, 2, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [0, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),       // 0: transparent
                (210, 60, 40, 255), // 1: red fill
                (80, 20, 10, 255),  // 2: dark red-brown outlines/features
                (0, 0, 0, 0),       // 3: unused
            ],
        ),

        // ── Exclamation: bold "!" on orange/yellow circle ──
        EmoteKind::Exclamation => (
            [
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 0],
                [2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                [0, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),        // 0: transparent
                (255, 180, 40, 255), // 1: orange-yellow fill
                (120, 50, 10, 255),  // 2: dark brown outline and "!"
                (0, 0, 0, 0),        // 3: unused
            ],
        ),

        // ── Question: bold "?" on blue circle ──
        EmoteKind::Question => (
            [
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0, 0],
                [0, 2, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1, 1, 1, 2, 0],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 2],
                [2, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 2],
                [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 0],
                [0, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 0],
                [0, 0, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 0, 0],
                [0, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0],
                [0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                (0, 0, 0, 0),         // 0: transparent
                (110, 180, 255, 255), // 1: light blue fill
                (20, 50, 100, 255),   // 2: dark blue outline and "?"
                (0, 0, 0, 0),         // 3: unused
            ],
        ),
    };

    for (py, row) in pattern.iter().enumerate() {
        for (px, &idx) in row.iter().enumerate() {
            if idx > 0 && (idx as usize) < palette.len() {
                let (pr, pg, pb, pa) = palette[idx as usize];
                put_pixel(&mut data, px, py, pr, pg, pb, pa);
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
        sprite.custom_size = Some(Vec2::splat(16.0));

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
