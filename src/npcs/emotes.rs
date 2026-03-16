//! Floating emote bubbles above NPCs — atlas-based hand-drawn sprites.
//!
//! When an NPC reacts (gift, dialogue, etc.), a small emote sprite
//! appears above their head, floats upward, and fades out.
//!
//! Sprites are sourced from `assets/ui/emoji_spritesheet.png`
//! (160×608, 10 columns × 38 rows, each tile 16×16).
//! The procedural `make_emote_image()` is retained as a compile-time fallback.

use crate::shared::*;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// ═══════════════════════════════════════════════════════════════════════
// EMOTE SPRITE CACHE
// ═══════════════════════════════════════════════════════════════════════

/// Path to the hand-drawn emote spritesheet (10 cols × 38 rows, 16×16 tiles).
const EMOTE_SHEET_PATH: &str = "ui/emoji_spritesheet.png";
/// Number of columns in the emote spritesheet.
const EMOTE_COLS: u32 = 10;
/// Number of rows in the emote spritesheet.
const EMOTE_ROWS: u32 = 38;

/// Atlas-backed emote sprite cache (loaded once, reused per emote event).
#[derive(Resource, Default)]
pub struct EmoteSprites {
    /// Handle to the emoji spritesheet image.
    pub image: Handle<Image>,
    /// Handle to the shared TextureAtlasLayout for the spritesheet.
    pub layout: Handle<TextureAtlasLayout>,
    /// Whether the atlas handles have been registered.
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
    /// Returns the atlas index into `emoji_spritesheet.png` for this emote kind.
    ///
    /// The sheet is 10 columns wide, so index = row * 10 + col.
    /// These are initial guesses pending runtime visual verification (Harden phase).
    /// [Assumed] — actual visual content at each index has not been verified at runtime.
    pub fn atlas_index(self) -> usize {
        match self {
            // row 0, col 0 — hearts are typically first in emoji conventions
            EmoteKind::Heart => 0,
            // row 1, col 0 — smiley faces
            EmoteKind::Happy => 10,
            // row 1, col 2 — neutral face
            EmoteKind::Neutral => 12,
            // row 2, col 0 — sad face
            EmoteKind::Sad => 20,
            // row 2, col 2 — angry face
            EmoteKind::Angry => 22,
            // row 3, col 0 — surprised/exclamation
            EmoteKind::Exclamation => 30,
            // row 3, col 2 — question/sweat
            EmoteKind::Question => 32,
        }
    }
}

/// Helper: write an RGBA pixel into the data buffer at (x, y) for a 16-wide image.
#[allow(dead_code)]
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
///
/// Retained as a fallback — used only if the atlas fails to load.
#[allow(dead_code)]
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
///
/// Loads `emoji_spritesheet.png` as a TextureAtlas on first call.
/// Each EmoteKind maps to an atlas index (see `EmoteKind::atlas_index()`).
pub fn spawn_emote_bubbles(
    mut commands: Commands,
    mut events: EventReader<NpcEmoteEvent>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut emote_sprites: ResMut<EmoteSprites>,
    npc_query: Query<(&Npc, &Transform)>,
) {
    // Register atlas handles once on first call.
    if !emote_sprites.loaded {
        emote_sprites.image = asset_server.load(EMOTE_SHEET_PATH);
        emote_sprites.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            EMOTE_COLS,
            EMOTE_ROWS,
            None,
            None,
        ));
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

        let atlas_index = event.emote.atlas_index();

        let mut sprite = Sprite::from_atlas_image(
            emote_sprites.image.clone(),
            TextureAtlas {
                layout: emote_sprites.layout.clone(),
                index: atlas_index,
            },
        );
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
