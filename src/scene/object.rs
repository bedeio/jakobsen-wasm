use super::{ConstraintId, ParticleId, Vec2};
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Object {
    particles: Vec<ParticleId>,
    constraints: Vec<ConstraintId>,
    convex_hull: Vec<Vec2>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ConvexHull {
    init: bool,
    vertices: Vec<Vec2>,
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

    fn compute_convex_hull(&self) {}
}
