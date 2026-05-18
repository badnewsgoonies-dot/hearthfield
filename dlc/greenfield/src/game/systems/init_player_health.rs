use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Loads the fishing sprite atlas on first entry into Playing state.
pub fn init_player_health(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas: ResMut<FishingAtlas>,
) {
    if atlas.loaded {
        return;
    }
    // fishing_atlas.png: 128x96, 8 cols x 6 rows of 16x16 tiles (48 sprites)
    // Row 0: rod, old rod, bobber, hook, worm bait, spinner lure, tackle box, bucket
    // Row 1: net, crab trap, cooler, treasure chest, wooden crate, splash, ripple, fish shadow
    // Row 2: carp(16), bluegill(17), perch(18), trout(19), catfish(20), bass(21), salmon(22), sardine(23)
    // Row 3: red snapper(24), tuna(25), swordfish(26), eel(27), anglerfish(28), pufferfish(29), koi(30), goldfish(31)
    // Row 4: seaweed(32), coral(33), shell(34), pearl(35), starfish(36), driftwood(37), message bottle(38), old boot(39)
    // Row 5: ancient coin(40), sunken key(41), fish bone(42), crab(43), lobster(44), octopus(45), squid(46), jellyfish(47)
    atlas.image = asset_server.load("sprites/fishing_atlas.png");
    atlas.layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        8,
        6,
        None,
        None,
    ));
    atlas.loaded = true;
}


