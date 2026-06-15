//! Seed-addressed waves — the ADD/addressable channel.
//!
//! A wave is a *pure function* of one `u64`: `addressed_wave(seed)` always grows the
//! identical set of enemies. This is the wave analog of `MapId::Procedural(u64)` —
//! generation IS retrieval. The seed is the address; nothing is stored.
//!
//! (Lab finding: the generative/ADD side of the tower-defense loop is perfectly
//! addressable — determinism 1000/1000, ~9x addressing ratio. The post-combat
//! survivor set is NOT seed-addressable; it is path-dependent on the player's action
//! trace, and lives on the trace/replay side. Build by addition, gate removal.)

use bevy::prelude::*;
use crate::game::components::Enemy;

/// The wave coordinate. Same seed ⇒ identical wave (reproducible, can't drift).
#[derive(Resource, Debug, Clone, Copy)]
pub struct WaveSeed(pub u64);
impl Default for WaveSeed {
    fn default() -> Self { WaveSeed(0xA17C_3D5E_9F2B_8146) }
}

struct Rng(u64);
impl Rng {
    fn new(s: u64) -> Self { Rng(s ^ 0xD1B5_4A32_D192_ED03) }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 { self.next() % n }
}

/// Pure function: `seed` → the full enemy set (x, y, hp, kind). No external input.
pub fn addressed_wave(seed: u64) -> Vec<(f32, f32, u8, u8)> {
    let mut r = Rng::new(seed);
    let n = 8 + r.below(9) as usize; // 8..16 enemies, deterministic in the seed
    (0..n)
        .map(|_| {
            let x = (r.below(880) as f32) - 440.0; // centered field coords
            let y = (r.below(480) as f32) - 240.0;
            let hp = 2 + r.below(5) as u8;
            let kind = r.below(4) as u8;
            (x, y, hp, kind)
        })
        .collect()
}

/// Press **W** to spawn the seed-addressed wave. Same `WaveSeed` ⇒ the identical wave
/// every time (the "can't lie" property); the seed is bumped so successive presses
/// address successive waves.
pub fn spawn_addressed_wave_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut seed: ResMut<WaveSeed>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyW) {
        for (x, y, _hp, kind) in addressed_wave(seed.0) {
            let color = match kind {
                0 => Color::srgb(0.85, 0.20, 0.20),
                1 => Color::srgb(0.90, 0.55, 0.20),
                2 => Color::srgb(0.80, 0.25, 0.60),
                _ => Color::srgb(0.55, 0.20, 0.20),
            };
            commands.spawn((
                Sprite { color, custom_size: Some(Vec2::splat(20.0)), ..default() },
                Transform::from_xyz(x, y, 1.0),
                Enemy,
            ));
        }
        seed.0 = seed.0.wrapping_add(1); // next press addresses the next wave
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wave_is_a_pure_function_of_the_seed() {
        for s in [0u64, 1, 42, 1000, u64::MAX] {
            assert_eq!(addressed_wave(s), addressed_wave(s), "same seed must give the same wave");
        }
        // distinct seeds generally differ
        assert_ne!(addressed_wave(1), addressed_wave(2));
    }
}
