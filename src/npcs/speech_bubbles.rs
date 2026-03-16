//! Proximity-based speech bubble indicators above NPCs.
//!
//! When the player enters interaction range (~3 tiles / 48px) of an NPC, a small
//! speech bubble sprite appears above that NPC's head as a visual "come talk to me"
//! cue.  The bubble is despawned as soon as the player moves away.
//!
//! Sprites are sourced from `assets/ui/speech_bubble.png`
//! (448×192, 28 columns × 12 rows, each tile 16×16).
//!
//! Atlas index 0 is used as the default bubble style.
//! [Assumed] — visual content at index 0 has not been verified at runtime.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Path to the speech bubble spritesheet.
const SPEECH_BUBBLE_SHEET_PATH: &str = "ui/speech_bubble.png";

/// Columns in speech_bubble.png (28 tiles wide).
const SPEECH_BUBBLE_COLS: u32 = 28;

/// Rows in speech_bubble.png (12 tiles tall).
const SPEECH_BUBBLE_ROWS: u32 = 12;

/// Atlas index for the default small bubble sprite.
/// [Assumed] — visual verification needed at runtime.
const DEFAULT_BUBBLE_INDEX: usize = 0;

/// Distance in world units at which speech bubbles appear.
/// 3 tiles × 16px = 48px.  Larger than the 24px dialogue trigger so the
/// bubble appears before the player can actually start a conversation.
const BUBBLE_APPEAR_RANGE: f32 = TILE_SIZE * 3.0;

/// Y offset above the NPC centre to place the bubble sprite.
/// NPC sprites are 24px tall with BottomCenter anchor, so adding 24 puts us
/// just above the top of the sprite.
const BUBBLE_Y_OFFSET: f32 = 24.0;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCE
// ═══════════════════════════════════════════════════════════════════════

/// Atlas handles for the speech bubble spritesheet (loaded on first use).
#[derive(Resource, Default)]
pub struct SpeechBubbleAtlas {
    /// Handle to the speech_bubble.png image.
    pub image: Handle<Image>,
    /// Handle to the shared TextureAtlasLayout.
    pub layout: Handle<TextureAtlasLayout>,
    /// Whether the handles have been registered with the asset server.
    pub loaded: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// COMPONENT
// ═══════════════════════════════════════════════════════════════════════

/// Marker component placed on a speech bubble sprite entity.
/// Stores the NPC entity it belongs to so we can match / despawn correctly.
#[derive(Component)]
pub struct SpeechBubble {
    /// The NPC entity above which this bubble is floating.
    pub npc_entity: Entity,
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM
// ═══════════════════════════════════════════════════════════════════════

/// Show/hide speech bubble sprites above NPCs based on player proximity.
///
/// - NPCs within `BUBBLE_APPEAR_RANGE` that have no bubble → spawn one.
/// - NPCs outside range that have a bubble → despawn it.
///
/// Runs only during `Playing` state; bubbles are therefore automatically
/// absent during `Dialogue`, `Shop`, etc.
pub fn show_npc_interaction_bubbles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas: ResMut<SpeechBubbleAtlas>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(Entity, &Transform), With<Npc>>,
    bubble_query: Query<(Entity, &SpeechBubble)>,
) {
    // Load atlas handles once on first system call.
    if !atlas.loaded {
        atlas.image = asset_server.load(SPEECH_BUBBLE_SHEET_PATH);
        atlas.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            SPEECH_BUBBLE_COLS,
            SPEECH_BUBBLE_ROWS,
            None,
            None,
        ));
        atlas.loaded = true;
    }

    // Obtain the player's world position.  If no player exists yet, bail out.
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate(); // Vec2

    // Build a quick lookup: npc_entity → bubble_entity
    let mut existing_bubbles: std::collections::HashMap<Entity, Entity> =
        std::collections::HashMap::new();
    for (bubble_entity, bubble) in &bubble_query {
        existing_bubbles.insert(bubble.npc_entity, bubble_entity);
    }

    for (npc_entity, npc_transform) in &npc_query {
        let npc_pos = npc_transform.translation.truncate();
        let dist = player_pos.distance(npc_pos);

        if dist <= BUBBLE_APPEAR_RANGE {
            // Within range — spawn a bubble if one does not already exist.
            if !existing_bubbles.contains_key(&npc_entity) {
                let bubble_x = npc_transform.translation.x;
                let bubble_y = npc_transform.translation.y + BUBBLE_Y_OFFSET;
                let bubble_z = Z_ENTITY_BASE + 50.0; // above NPC sprite

                let mut sprite = Sprite::from_atlas_image(
                    atlas.image.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: DEFAULT_BUBBLE_INDEX,
                    },
                );
                sprite.custom_size = Some(Vec2::new(32.0, 32.0));

                commands.spawn((
                    SpeechBubble { npc_entity },
                    sprite,
                    Transform::from_xyz(bubble_x, bubble_y, bubble_z),
                    Visibility::default(),
                ));
            }
        } else {
            // Out of range — despawn any existing bubble for this NPC.
            if let Some(&bubble_entity) = existing_bubbles.get(&npc_entity) {
                commands.entity(bubble_entity).despawn();
            }
        }
    }
}
