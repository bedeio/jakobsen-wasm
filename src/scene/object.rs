use super::{ConstraintId, ParticleId, Vec2};
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Object {
    particles: Vec<ParticleId>,
    edges: Vec<ConstraintId>,
}

impl Object {
    fn new() -> Object {}

    fn add_particle(&self, p: ParticleId) {
        self.particles.push(p);
    }

    fn add_constraint(&self, c: ConstraintId) {
        self.constraints.push(c);
    }

    fn validate_polygon(&self) -> bool {}
}
