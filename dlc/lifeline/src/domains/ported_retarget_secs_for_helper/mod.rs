//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

pub trait Rng {
    fn gen_range<A0>(&mut self, _a0: A0) -> f32 { 0.0 }
}
impl<T> Rng for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimalKind {
    #[default]
    Chicken,
    Cow,
    Dog,
    Rabbit,
}


pub fn retarget_secs_for_helper(kind: AnimalKind, rng: &mut impl Rng) -> f32 {
    match kind {
        AnimalKind::Chicken => rng.gen_range(0.6_f32..=1.4_f32),
        AnimalKind::Cow => rng.gen_range(3.5_f32..=6.0_f32),
        AnimalKind::Rabbit => rng.gen_range(0.25_f32..=0.8_f32),
        AnimalKind::Dog => rng.gen_range(0.5_f32..=1.2_f32),
        _ => rng.gen_range(2.0_f32..=4.0_f32),
    }
}


