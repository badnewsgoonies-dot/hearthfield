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

/// Append-only record of every wave coordinate spawned this session (the ADD-side history).
/// With `KillLog` (the REMOVE-side witness), `(WaveHistory, KillLog)` captures the entire session
/// as coordinates — enough to replay it exactly, with no per-tick state stored. (Trial 3: a whole
/// session reconstructs 100% from its coordinates and replays deterministically.)
#[derive(Resource, Default, Debug)]
pub struct WaveHistory(pub Vec<u64>);

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
    mut history: ResMut<WaveHistory>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyW) {
        history.0.push(seed.0); // record the coordinate (append-only) before spawning
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

// ─────────────────────────────────────────────────────────────────────────────
// A TRUE bijective wave coordinate (mixed-radix), superseding the splitmix key.
//
// `addressed_wave(seed)` above is a one-way KEY: unrank-only, avalanche, a sampler — you can
// dereference it but you can't find the index of a wave you want, the bits carry no structure,
// and sampling never proves the space. `coord::unrank(index)` here is a COORDINATE over an
// explicit enumerated grammar: it has a genuine inverse `coord::rank` (address a *specific* wave),
// adjacent indices give adjacent waves (positional), and enumeration covers the space exactly.
// (Measured standalone: bijection 100%, round-trip identity, ~5% adjacent-index change vs ~84%
// for the key, 100% coverage vs ~63% sampling.)
// ─────────────────────────────────────────────────────────────────────────────
pub mod coord {
    pub const GRID_W: u128 = 22;
    pub const GRID_H: u128 = 12;
    pub const CELLS: u128 = GRID_W * GRID_H; // 264 positions
    pub const HPS: u128 = 5;                 // hp ∈ {2..6}
    pub const KINDS: u128 = 4;
    pub const RADIX: u128 = CELLS * HPS * KINDS; // 5280 distinct enemies
    pub const N: usize = 8;                  // fixed wave size for this grammar
    /// The field's cardinality: RADIX^N. The domain law is [0, M); everything
    /// at or beyond M refuses by name inside the operator (never caller-side).
    /// wave_coordinate.v0 is ORDERED by operator ruling (2026-08-15); a
    /// multiset identity would be a second admitted family, never a silent
    /// re-meaning of this one.
    pub const M: u128 = 604_047_902_015_764_404_633_600_000_000;

    /// Named refusals of the coordinate field. `address_outside_field` is the
    /// ruled name for Q >= M (rediscovered from the live GCOS vocabulary);
    /// the two structural codes are its in-pattern kin. Pre-repair battery:
    /// gc-project status/wave_coordinate_v0/BATTERY_BEFORE.md.
    #[derive(Debug, PartialEq, Eq)]
    pub enum CoordRefusal {
        /// Q decodes nowhere: Q >= M. No aliasing, no modular reuse.
        AddressOutsideField { q: u128, m: u128 },
        /// rank input is not exactly N enemies.
        AddressLengthInvalid { got: usize, want: usize },
        /// a component digit is outside its declared range.
        AddressDigitOutsideField { index: usize, cell: u128, hp: u128, kind: u128 },
    }

    impl CoordRefusal {
        /// The stable refusal name, matching the GCOS vocabulary.
        pub fn name(&self) -> &'static str {
            match self {
                CoordRefusal::AddressOutsideField { .. } => "address_outside_field",
                CoordRefusal::AddressLengthInvalid { .. } => "address_length_invalid",
                CoordRefusal::AddressDigitOutsideField { .. } => "address_digit_outside_field",
            }
        }
    }

    /// unrank: index → wave (Vec of (x, y, hp, kind)). Pure, positional,
    /// defined on [0, M) and REFUSING outside it — the check lives here,
    /// inside the operator, never at a caller.
    pub fn unrank(mut k: u128) -> Result<Vec<(f32, f32, u8, u8)>, CoordRefusal> {
        if k >= M {
            return Err(CoordRefusal::AddressOutsideField { q: k, m: M });
        }
        let mut codes = vec![0u128; N];
        for i in (0..N).rev() { codes[i] = k % RADIX; k /= RADIX; } // base-RADIX, big-endian
        Ok(codes.into_iter().map(|e| {
            let kind = (e % KINDS) as u8; let e2 = e / KINDS;
            let hp = (e2 % HPS) as u8 + 2; let cell = e2 / HPS;
            let cx = (cell % GRID_W) as f32; let cy = (cell / GRID_W) as f32;
            (cx * 40.0 - 440.0, cy * 40.0 - 240.0, hp, kind)
        }).collect())
    }

    /// rank: the genuine inverse over checked digits. Exactly N enemies,
    /// each (cell < CELLS, hp < HPS, kind < KINDS); anything else refuses
    /// by name. On the checked domain, rank∘unrank == id and no malformed
    /// vector can collide with a valid coordinate.
    pub fn rank(enemies: &[(u128, u128, u128)]) -> Result<u128, CoordRefusal> {
        if enemies.len() != N {
            return Err(CoordRefusal::AddressLengthInvalid { got: enemies.len(), want: N });
        }
        let mut x = 0u128;
        for (i, &(cell, hp, kind)) in enemies.iter().enumerate() {
            if cell >= CELLS || hp >= HPS || kind >= KINDS {
                return Err(CoordRefusal::AddressDigitOutsideField { index: i, cell, hp, kind });
            }
            x = x * RADIX + (cell * HPS + hp) * KINDS + kind;
        }
        Ok(x)
    }

    #[cfg(test)]
    mod m_law {
        #[test]
        fn m_is_exactly_radix_to_the_n() {
            assert_eq!(super::M, super::RADIX.pow(super::N as u32));
        }
    }
}

/// The wave coordinate index (a true position in the wave-grammar, unlike the splitmix key).
#[derive(Resource, Default, Debug)]
pub struct WaveIndex(pub u128);

/// Press **I** to spawn the wave at the current *coordinate* (then advance by one). Sequential
/// indices give adjacent waves — the legible, positional counterpart to W's avalanche key.
pub fn spawn_indexed_wave_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut idx: ResMut<WaveIndex>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        // The caller consumes the checked Result; the domain law lives in the
        // operator. At the rim (idx == M) this refuses by name — it never
        // wraps and never manufactures a coordinate. A cyclic counter would
        // be a different admitted family.
        match coord::unrank(idx.0) {
            Ok(wave) => {
                for (x, y, _hp, kind) in wave {
                    let color = match kind {
                        0 => Color::srgb(0.20, 0.60, 0.90),
                        1 => Color::srgb(0.30, 0.80, 0.50),
                        2 => Color::srgb(0.90, 0.80, 0.20),
                        _ => Color::srgb(0.70, 0.40, 0.90),
                    };
                    commands.spawn((
                        Sprite { color, custom_size: Some(Vec2::splat(20.0)), ..default() },
                        Transform::from_xyz(x, y, 1.0),
                        Enemy,
                    ));
                }
                // Advance only within the field; the counter halts at M-1.
                if idx.0 + 1 < coord::M {
                    idx.0 += 1;
                } else {
                    info!("wave coordinate rim: next index would leave the field \
                           (address_outside_field); counter holds at M-1");
                }
            }
            Err(refusal) => {
                info!("wave spawn refused: {} ({:?})", refusal.name(), refusal);
            }
        }
    }
}

#[cfg(test)]
mod coord_tests {
    use super::coord;
    use super::coord::CoordRefusal;

    #[test]
    fn rank_is_the_genuine_inverse_of_unrank_on_the_domain() {
        for k in [0u128, 1, 5279, 5280, 123_456_789, u64::MAX as u128, coord::M - 1] {
            let w = coord::unrank(k).expect("in-domain unrank");
            let enemies: Vec<(u128, u128, u128)> = w.iter().map(|&(x, y, hp, kind)| {
                let cx = ((x + 440.0) / 40.0) as u128;
                let cy = ((y + 240.0) / 40.0) as u128;
                (cy * coord::GRID_W + cx, hp as u128 - 2, kind as u128)
            }).collect();
            assert_eq!(coord::rank(&enemies), Ok(k), "rank∘unrank must be identity");
        }
    }

    #[test]
    fn outside_the_field_refuses_by_name() {
        // The BATTERY_BEFORE roster: M, M+1, u128::MAX all refused; M-1 green.
        for q in [coord::M, coord::M + 1, u128::MAX] {
            let got = coord::unrank(q);
            assert_eq!(
                got,
                Err(CoordRefusal::AddressOutsideField { q, m: coord::M }),
                "Q >= M must refuse address_outside_field, never alias"
            );
        }
        assert!(coord::unrank(coord::M - 1).is_ok(), "M-1 is the last citizen");
    }

    #[test]
    fn malformed_rank_inputs_refuse_by_name() {
        // Wrong length: exactly N or refusal, never positional acceptance.
        let short = [(0u128, 0u128, 0u128); 3];
        assert_eq!(
            coord::rank(&short),
            Err(CoordRefusal::AddressLengthInvalid { got: 3, want: coord::N })
        );
        // Out-of-range digit: the old collision rank((0,0,4)) == rank((0,1,0))
        // is now unrepresentable — the invalid tuple refuses before folding.
        let mut wave = [(0u128, 0u128, 0u128); 8];
        wave[0] = (0, 0, coord::KINDS); // kind == 4, outside [0, KINDS)
        assert_eq!(
            coord::rank(&wave),
            Err(CoordRefusal::AddressDigitOutsideField { index: 0, cell: 0, hp: 0, kind: coord::KINDS })
        );
    }

    #[test]
    fn ordered_is_the_identity_by_ruling() {
        // wave_coordinate.v0 is ORDERED (operator ruling 2026-08-15):
        // a permuted vector is a different coordinate, lawfully.
        let a: Vec<(u128, u128, u128)> = (0..8).map(|i| (i as u128, 0, 0)).collect();
        let b: Vec<(u128, u128, u128)> = (0..8).rev().map(|i| (i as u128, 0, 0)).collect();
        assert_ne!(coord::rank(&a).unwrap(), coord::rank(&b).unwrap());
    }
}
