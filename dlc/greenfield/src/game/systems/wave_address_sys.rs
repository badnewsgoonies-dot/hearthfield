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

use crate::game::components::{Enemy, WaveOrigin};
use crate::game::systems::tombstone_sys::{
    addressed_enemy_snapshot, wave_enemy_key, EnemyKeyAllocator, KillLog, ADDRESSED_WAVE_GENERATION,
};
use bevy::prelude::*;

/// The wave coordinate. Same seed ⇒ identical wave (reproducible, can't drift).
#[derive(Resource, Debug, Clone, Copy)]
pub struct WaveSeed(pub u64);
impl Default for WaveSeed {
    fn default() -> Self {
        WaveSeed(0xA17C_3D5E_9F2B_8146)
    }
}

/// Convenience index of addressed-wave seeds. The authoritative replay history
/// is `KillLog`; this cache alone makes no whole-session claim.
#[derive(Resource, Default, Debug)]
pub struct WaveHistory(pub Vec<u64>);

struct Rng(u64);
impl Rng {
    fn new(s: u64) -> Self {
        Rng(s ^ 0xD1B5_4A32_D192_ED03)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
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
    mut replay_history: ResMut<KillLog>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyW) {
        history.0.push(seed.0); // record the coordinate (append-only) before spawning
        let wave_ref = replay_history.append_wave_spawned(seed.0, ADDRESSED_WAVE_GENERATION);
        for (member_index, (x, y, _hp, kind)) in addressed_wave(seed.0).into_iter().enumerate() {
            let Some(enemy_key) = wave_enemy_key(wave_ref, member_index) else {
                error!("addressed-wave enemy key outside field");
                continue;
            };
            let snapshot = addressed_enemy_snapshot(x, y, kind);
            commands.spawn((
                snapshot.sprite(),
                snapshot.transform(),
                Enemy,
                enemy_key,
                WaveOrigin(wave_ref.0),
            ));
        }
        seed.0 = seed.0.wrapping_add(1); // next press addresses the next wave
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::components::{EnemyKey, WaveOrigin};
    use crate::game::systems::tombstone_sys::{
        replay_history, EnemyHistoryEvent, EnemySnapshot, HistoryEventRef, ReplayedEnemy,
    };

    #[test]
    fn wave_is_a_pure_function_of_the_seed() {
        for s in [0u64, 1, 42, 1000, u64::MAX] {
            assert_eq!(
                addressed_wave(s),
                addressed_wave(s),
                "same seed must give the same wave"
            );
        }
        // distinct seeds generally differ
        assert_ne!(addressed_wave(1), addressed_wave(2));
    }

    #[test]
    fn live_addressed_spawn_matches_the_append_only_replay_quotient() {
        let seed = 42;
        let expected_count = addressed_wave(seed).len();
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(WaveSeed(seed))
            .init_resource::<WaveHistory>()
            .init_resource::<KillLog>()
            .add_systems(Update, spawn_addressed_wave_system);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();

        let log = app.world().resource::<KillLog>();
        assert_eq!(log.records().len(), 1);
        assert!(matches!(
            log.records()[0].event,
            EnemyHistoryEvent::WaveSpawned {
                seed: recorded_seed,
                generation: ADDRESSED_WAVE_GENERATION,
            } if recorded_seed == seed
        ));
        let replayed = replay_history(log).expect("live history must replay");
        assert_eq!(replayed.live_enemies.len(), expected_count);

        let mut query = app
            .world_mut()
            .query::<(&EnemyKey, &WaveOrigin, &Transform, &Sprite)>();
        let mut live: Vec<_> = query
            .iter(app.world())
            .map(|(key, origin, transform, sprite)| ReplayedEnemy {
                entity_key: *key,
                full_snapshot: EnemySnapshot::capture(transform, sprite),
                causal_parent: Some(HistoryEventRef(origin.0)),
            })
            .collect();
        live.sort_by_key(|enemy| enemy.entity_key);

        assert_eq!(live, replayed.live_enemies);
        for (member_index, enemy) in live.iter().enumerate() {
            assert_eq!(
                enemy.entity_key,
                wave_enemy_key(HistoryEventRef(0), member_index).unwrap()
            );
        }
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
    // The game runtime consumes unrank/advance. The reverse boundary is retained
    // for external coordinate admission and round-trip certification.
    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EnemyComponent {
        Cell,
        Hp,
        Kind,
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Refusal {
        OutsideField {
            q: u128,
            upper_bound: u128,
        },
        LengthInvalid {
            actual: usize,
            expected: usize,
        },
        DigitOutsideField {
            enemy_index: usize,
            component: EnemyComponent,
            value: u128,
            upper_bound: u128,
        },
    }

    impl Refusal {
        pub const fn code(&self) -> &'static str {
            match self {
                Self::OutsideField { .. } => "address_outside_field",
                Self::LengthInvalid { .. } => "address_length_invalid",
                Self::DigitOutsideField { .. } => "address_digit_outside_field",
            }
        }
    }

    pub const GRID_W: u128 = 22;
    pub const GRID_H: u128 = 12;
    pub const CELLS: u128 = GRID_W * GRID_H; // 264 positions
    pub const HPS: u128 = 5; // hp ∈ {2..6}
    pub const KINDS: u128 = 4;
    pub const RADIX: u128 = CELLS * HPS * KINDS; // 5280 distinct enemies
    pub const N: usize = 8; // fixed wave size for this grammar

    const fn domain_size() -> u128 {
        let mut size = 1u128;
        let mut position = 0usize;
        while position < N {
            size *= RADIX;
            position += 1;
        }
        size
    }

    pub const DOMAIN_SIZE: u128 = domain_size();

    /// Unrank an in-domain coordinate into its ordered eight-enemy wave.
    pub fn unrank(mut q: u128) -> Result<Vec<(f32, f32, u8, u8)>, Refusal> {
        if q >= DOMAIN_SIZE {
            return Err(Refusal::OutsideField {
                q,
                upper_bound: DOMAIN_SIZE,
            });
        }

        let mut codes = vec![0u128; N];
        for i in (0..N).rev() {
            codes[i] = q % RADIX;
            q /= RADIX;
        }
        Ok(codes
            .into_iter()
            .map(|enemy| {
                let kind = (enemy % KINDS) as u8;
                let rest = enemy / KINDS;
                let hp = (rest % HPS) as u8 + 2;
                let cell = rest / HPS;
                let cx = (cell % GRID_W) as f32;
                let cy = (cell / GRID_W) as f32;
                (cx * 40.0 - 440.0, cy * 40.0 - 240.0, hp, kind)
            })
            .collect())
    }

    /// Rank exactly eight valid `(cell, hp_digit, kind)` entries in sequence order.
    #[allow(dead_code)]
    pub fn rank(enemies: &[(u128, u128, u128)]) -> Result<u128, Refusal> {
        if enemies.len() != N {
            return Err(Refusal::LengthInvalid {
                actual: enemies.len(),
                expected: N,
            });
        }

        let mut x = 0u128;
        for (enemy_index, &(cell, hp, kind)) in enemies.iter().enumerate() {
            for (component, value, upper_bound) in [
                (EnemyComponent::Cell, cell, CELLS),
                (EnemyComponent::Hp, hp, HPS),
                (EnemyComponent::Kind, kind, KINDS),
            ] {
                if value >= upper_bound {
                    return Err(Refusal::DigitOutsideField {
                        enemy_index,
                        component,
                        value,
                        upper_bound,
                    });
                }
            }
            x = x * RADIX + (cell * HPS + hp) * KINDS + kind;
        }
        Ok(x)
    }

    /// Advance within the non-cyclic v0 field. The upper rim is a refusal.
    pub fn advance(q: u128) -> Result<u128, Refusal> {
        if q >= DOMAIN_SIZE {
            return Err(Refusal::OutsideField {
                q,
                upper_bound: DOMAIN_SIZE,
            });
        }
        if q == DOMAIN_SIZE - 1 {
            return Err(Refusal::OutsideField {
                q: DOMAIN_SIZE,
                upper_bound: DOMAIN_SIZE,
            });
        }
        Ok(q + 1)
    }
}

/// The wave coordinate index (a true position in the wave-grammar, unlike the splitmix key).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveProgress {
    Ready(u128),
    Refused(coord::Refusal),
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveIndex(pub WaveProgress);

impl Default for WaveIndex {
    fn default() -> Self {
        Self(WaveProgress::Ready(0))
    }
}

/// Press **I** to spawn the wave at the current *coordinate* (then advance by one). Sequential
/// indices give adjacent waves — the legible, positional counterpart to W's avalanche key.
pub fn spawn_indexed_wave_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut idx: ResMut<WaveIndex>,
    mut enemy_keys: ResMut<EnemyKeyAllocator>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        let q = match idx.0 {
            WaveProgress::Ready(q) => q,
            WaveProgress::Refused(refusal) => {
                bevy::log::error!("wave_coordinate.v0 refused: {}", refusal.code());
                return;
            }
        };
        let wave = match coord::unrank(q) {
            Ok(wave) => wave,
            Err(refusal) => {
                bevy::log::error!("wave_coordinate.v0 refused: {}", refusal.code());
                idx.0 = WaveProgress::Refused(refusal);
                return;
            }
        };
        for (x, y, _hp, kind) in wave {
            let Some(enemy_key) = enemy_keys.allocate() else {
                bevy::log::error!("enemy key field exhausted; indexed wave refused");
                return;
            };
            let color = match kind {
                0 => Color::srgb(0.20, 0.60, 0.90),
                1 => Color::srgb(0.30, 0.80, 0.50),
                2 => Color::srgb(0.90, 0.80, 0.20),
                _ => Color::srgb(0.70, 0.40, 0.90),
            };
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
                Transform::from_xyz(x, y, 1.0),
                Enemy,
                enemy_key,
            ));
        }
        idx.0 = match coord::advance(q) {
            Ok(next_q) => WaveProgress::Ready(next_q),
            Err(refusal) => {
                bevy::log::error!("wave_coordinate.v0 refused: {}", refusal.code());
                WaveProgress::Refused(refusal)
            }
        };
    }
}

#[cfg(test)]
mod coord_tests {
    use super::{coord, spawn_indexed_wave_system, WaveIndex, WaveProgress};
    use crate::game::components::Enemy;
    use crate::game::systems::tombstone_sys::EnemyKeyAllocator;
    use bevy::prelude::*;

    mod oracle {
        const WIDTH: u128 = 22;
        const HEIGHT: u128 = 12;
        const HIT_POINT_VALUES: u128 = 5;
        const ENEMY_KINDS: u128 = 4;
        const ENEMIES_PER_WAVE: usize = 8;
        const ENEMY_COUNT: u128 = WIDTH * HEIGHT * HIT_POINT_VALUES * ENEMY_KINDS;

        const fn coordinate_count() -> u128 {
            let mut count = 1u128;
            let mut remaining = ENEMIES_PER_WAVE;
            while remaining > 0 {
                count *= ENEMY_COUNT;
                remaining -= 1;
            }
            count
        }

        pub const COORDINATE_COUNT: u128 = coordinate_count();

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Refusal {
            Outside,
            Length,
            Digit,
        }

        impl Refusal {
            pub const fn code(self) -> &'static str {
                match self {
                    Self::Outside => "address_outside_field",
                    Self::Length => "address_length_invalid",
                    Self::Digit => "address_digit_outside_field",
                }
            }
        }

        pub fn unrank(mut address: u128) -> Result<Vec<(f32, f32, u8, u8)>, Refusal> {
            if address >= COORDINATE_COUNT {
                return Err(Refusal::Outside);
            }
            let mut encoded = [0u128; ENEMIES_PER_WAVE];
            let mut position = ENEMIES_PER_WAVE;
            while position > 0 {
                position -= 1;
                encoded[position] = address % ENEMY_COUNT;
                address /= ENEMY_COUNT;
            }
            Ok(encoded
                .into_iter()
                .map(|value| {
                    let kind = (value % ENEMY_KINDS) as u8;
                    let without_kind = value / ENEMY_KINDS;
                    let hp = (without_kind % HIT_POINT_VALUES) as u8 + 2;
                    let cell = without_kind / HIT_POINT_VALUES;
                    let column = (cell % WIDTH) as f32;
                    let row = (cell / WIDTH) as f32;
                    (column * 40.0 - 440.0, row * 40.0 - 240.0, hp, kind)
                })
                .collect())
        }

        pub fn rank(values: &[(u128, u128, u128)]) -> Result<u128, Refusal> {
            if values.len() != ENEMIES_PER_WAVE {
                return Err(Refusal::Length);
            }
            let mut address = 0u128;
            for &(cell, hp, kind) in values {
                if cell >= WIDTH * HEIGHT || hp >= HIT_POINT_VALUES || kind >= ENEMY_KINDS {
                    return Err(Refusal::Digit);
                }
                let enemy = cell * HIT_POINT_VALUES * ENEMY_KINDS + hp * ENEMY_KINDS + kind;
                address = address * ENEMY_COUNT + enemy;
            }
            Ok(address)
        }

        pub fn advance(address: u128) -> Result<u128, Refusal> {
            if address >= COORDINATE_COUNT - 1 {
                return Err(Refusal::Outside);
            }
            Ok(address + 1)
        }
    }

    fn production_code<T>(result: Result<T, coord::Refusal>) -> Result<T, &'static str> {
        result.map_err(|refusal| refusal.code())
    }

    fn oracle_code<T>(result: Result<T, oracle::Refusal>) -> Result<T, &'static str> {
        result.map_err(oracle::Refusal::code)
    }

    #[test]
    fn rank_is_the_genuine_inverse_of_unrank() {
        for q in [
            0u128,
            1,
            5279,
            5280,
            123_456_789,
            u64::MAX as u128,
            coord::DOMAIN_SIZE - 1,
        ] {
            let w = coord::unrank(q).expect("declared address must unrank");
            let enemies: Vec<(u128, u128, u128)> = w
                .iter()
                .map(|&(x, y, hp, kind)| {
                    let cx = ((x + 440.0) / 40.0) as u128;
                    let cy = ((y + 240.0) / 40.0) as u128;
                    (cy * coord::GRID_W + cx, hp as u128 - 2, kind as u128)
                })
                .collect();
            assert_eq!(coord::rank(&enemies), Ok(q), "rank∘unrank must be identity");
        }
    }

    #[test]
    fn independent_oracle_agrees_on_values_domains_and_refusal_map() {
        assert_eq!(coord::DOMAIN_SIZE, 604_047_902_015_764_404_633_600_000_000);
        assert_eq!(coord::DOMAIN_SIZE, oracle::COORDINATE_COUNT);

        for q in [
            0,
            1,
            5279,
            5280,
            123_456_789,
            u64::MAX as u128,
            coord::DOMAIN_SIZE - 1,
            coord::DOMAIN_SIZE,
            coord::DOMAIN_SIZE + 1,
            7 + 2 * coord::DOMAIN_SIZE,
            u128::MAX,
        ] {
            assert_eq!(
                production_code(coord::unrank(q)),
                oracle_code(oracle::unrank(q)),
                "unrank disagreement at {q}"
            );
        }

        let valid = vec![(0, 0, 0); 8];
        let short = vec![(0, 0, 0), (1, 1, 1), (2, 2, 2)];
        let mut invalid_cell = valid.clone();
        invalid_cell[3].0 = 264;
        let mut invalid_hp = valid.clone();
        invalid_hp[4].1 = 5;
        let mut invalid_kind = valid.clone();
        invalid_kind[5].2 = 4;
        for values in [&valid, &short, &invalid_cell, &invalid_hp, &invalid_kind] {
            assert_eq!(
                production_code(coord::rank(values)),
                oracle_code(oracle::rank(values)),
                "rank refusal map disagreement for {values:?}"
            );
        }

        for q in [
            0,
            coord::DOMAIN_SIZE - 2,
            coord::DOMAIN_SIZE - 1,
            coord::DOMAIN_SIZE,
            u128::MAX,
        ] {
            assert_eq!(
                production_code(coord::advance(q)),
                oracle_code(oracle::advance(q)),
                "progression disagreement at {q}"
            );
        }
    }

    #[test]
    fn hostile_battery_flips_aliases_and_malformed_values_to_refusals() {
        let m = coord::DOMAIN_SIZE;

        assert_eq!(
            coord::unrank(m).unwrap_err().code(),
            "address_outside_field"
        );
        assert_eq!(
            coord::unrank(m + 1).unwrap_err().code(),
            "address_outside_field"
        );
        assert_eq!(
            coord::unrank(7 + 2 * m).unwrap_err().code(),
            "address_outside_field"
        );
        assert_eq!(
            coord::unrank(u128::MAX).unwrap_err().code(),
            "address_outside_field"
        );
        assert!(coord::unrank(m - 1).is_ok());

        // This exact three-enemy row produced 132_050 before the length law.
        let short = [(0, 0, 0), (1, 1, 1), (2, 2, 2)];
        assert_eq!(
            coord::rank(&short).unwrap_err().code(),
            "address_length_invalid"
        );

        let mut invalid = vec![(0, 0, 0); coord::N];
        invalid[0].2 = coord::KINDS;
        let mut formerly_colliding = vec![(0, 0, 0); coord::N];
        formerly_colliding[0].1 = 1;
        assert_eq!(
            coord::rank(&invalid).unwrap_err().code(),
            "address_digit_outside_field"
        );
        assert!(coord::rank(&formerly_colliding).is_ok());

        let mut ordered = vec![(0, 0, 0); coord::N];
        ordered[0] = (1, 1, 1);
        let mut permuted = ordered.clone();
        permuted.swap(0, 1);
        assert_ne!(
            coord::rank(&ordered).unwrap(),
            coord::rank(&permuted).unwrap(),
            "ordered wave identity must distinguish a permutation"
        );
    }

    #[test]
    fn every_invalid_enemy_component_names_its_structural_refusal() {
        for (component, value) in [
            (coord::EnemyComponent::Cell, coord::CELLS),
            (coord::EnemyComponent::Hp, coord::HPS),
            (coord::EnemyComponent::Kind, coord::KINDS),
        ] {
            let mut enemies = vec![(0, 0, 0); coord::N];
            match component {
                coord::EnemyComponent::Cell => enemies[6].0 = value,
                coord::EnemyComponent::Hp => enemies[6].1 = value,
                coord::EnemyComponent::Kind => enemies[6].2 = value,
            }
            assert_eq!(
                coord::rank(&enemies),
                Err(coord::Refusal::DigitOutsideField {
                    enemy_index: 6,
                    component,
                    value,
                    upper_bound: value,
                })
            );
        }
    }

    #[test]
    fn live_progression_spawns_the_last_wave_once_then_halts_on_refusal() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<EnemyKeyAllocator>()
            .insert_resource(WaveIndex(WaveProgress::Ready(coord::DOMAIN_SIZE - 1)))
            .add_systems(Update, spawn_indexed_wave_system);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyI);
        app.update();

        assert!(matches!(
            app.world().resource::<WaveIndex>().0,
            WaveProgress::Refused(coord::Refusal::OutsideField {
                q: coord::DOMAIN_SIZE,
                upper_bound: coord::DOMAIN_SIZE,
            })
        ));
        let first_count = app.world_mut().query::<&Enemy>().iter(app.world()).count();
        assert_eq!(first_count, coord::N);

        app.update();
        let second_count = app.world_mut().query::<&Enemy>().iter(app.world()).count();
        assert_eq!(second_count, first_count, "a refused rim cannot respawn");
    }
}
