use super::{ParticleId, Vec2};
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pair(pub ParticleId, pub ParticleId);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Projective {
    ToPoint {
        particles: Pair,
        dist: u32,
        cmp: Cmp,
    },
    ToFixed {
        particle: ParticleId,
        fixed: Vec2,
        dist: u32,
        cmp: Cmp,
    },
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cmp {
    Less,
    Equal,
    Greater,
}
