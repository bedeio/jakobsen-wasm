use super::constraint::{Cmp, Pair, Projective};

use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};
use serial_int::SerialGenerator;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref SERIAL_GEN: Mutex<SerialGenerator<usize>> = Mutex::new(SerialGenerator::new());
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2(pub f32, pub f32);

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2(x, y)
    }
}

#[wasm_bindgen]
impl Vec2 {
    #[wasm_bindgen(constructor)]
    pub fn new_js(x: f32, y: f32) -> JsValue {
        let v = &Vec2(x, y);
        JsValue::from_serde(v).unwrap()
    }

    pub fn zero() -> Vec2 {
        Vec2(0., 0.)
    }

    pub fn add(&mut self, other: Vec2) {
        self.0 += other.0;
        self.1 += other.1;
    }

    pub fn sub(&mut self, other: Vec2) {
        self.0 -= other.0;
        self.1 -= other.1;
    }

    pub fn dot(&mut self, other: Vec2) -> f32 {
        self.0 * other.0 + self.1 * other.1
    }

    pub fn scale(&mut self, scalar: f32) {
        self.0 *= scalar;
        self.1 *= scalar;
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.0 = x;
        self.1 = y;
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParticleId(usize);

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstraintId(usize);

pub struct Object {
    particles: Vec<ParticleId>,
    constraints: Vec<ConstraintId>,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct Particles {
    prev_pos: Vec<Vec2>,
    curr_pos: Vec<Vec2>,
    mass: Vec<f32>,
    force: Vec<Vec2>,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct Particle {
    pub prev_pos: Vec2,
    pub curr_pos: Vec2,
    pub mass: Vec2,
    pub force: Vec2,
}

impl Particle {
    pub fn new(prev_pos: Vec2, curr_pos: Vec2, mass: Vec2, force: Vec2) -> Particle {
        Particle {
            prev_pos,
            curr_pos,
            mass,
            force,
        }
    }
}

impl Particles {
    pub fn new() -> Particles {
        Particles {
            prev_pos: Vec::new(),
            curr_pos: Vec::new(),
            mass: Vec::new(),
            force: Vec::new(),
        }
    }

    pub fn prev_pos(&self) -> &Vec<Vec2> {
        &self.prev_pos
    }

    pub fn curr_pos(&self) -> &Vec<Vec2> {
        &self.curr_pos
    }

    pub fn mass(&self) -> &Vec<f32> {
        &self.mass
    }

    pub fn force(&self) -> &Vec<Vec2> {
        &self.force
    }

    pub fn set_prev_pos(&mut self, ind: usize, p: Vec2) {
        self.prev_pos[ind] = p;
    }

    pub fn set_curr_pos(&mut self, ind: usize, p: Vec2) {
        self.prev_pos[ind] = p;
    }

    pub fn add_prev_pos(&mut self, ind: usize, p: Vec2) {
        self.curr_pos[ind].add(p);
    }

    pub fn add_curr_pos(&mut self, ind: usize, p: Vec2) {
        self.curr_pos[ind].add(p);
    }

    pub fn sub_prev_pos(&mut self, ind: usize, p: Vec2) {
        self.curr_pos[ind].add(p);
    }

    pub fn sub_curr_pos(&mut self, ind: usize, p: Vec2) {
        self.curr_pos[ind].sub(p);
    }

    pub fn set_force(&mut self, ind: usize, f: Vec2) {
        self.force[ind] = f;
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct ParticleManager {
    pub(super) particles: Particles,
    pub(super) constraints: FxHashMap<ConstraintId, Projective>,
    pub(super) identifiers: FxHashMap<ParticleId, usize>,
}

impl ParticleManager {
    pub fn new() -> ParticleManager {
        ParticleManager {
            particles: Particles::new(),
            constraints: FxHashMap::default(),
            identifiers: FxHashMap::default(),
        }
    }

    pub fn add_constraint(&mut self, c: Projective) -> Option<Projective> {
        let next = SERIAL_GEN.lock().unwrap().generate();
        let id = ConstraintId(next);
        self.constraints.insert(id, c)
    }

    pub fn remove_constraint(&mut self, id: ConstraintId) -> Option<Projective> {
        self.constraints.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.particles.mass.len()
    }
}

#[wasm_bindgen]
impl ParticleManager {
    pub fn add(&mut self, prev_pos: Vec2, curr_pos: Vec2, mass: f32) {
        let parts = &mut self.particles;
        parts.prev_pos.push(prev_pos);
        parts.curr_pos.push(curr_pos);
        parts.mass.push(mass);
        //parts.force.push(force);
    }

    pub fn add_point_constraint(&mut self, p1: ParticleId, p2: ParticleId, dist: u32, cmp: Cmp) {
        let cons = Projective::ToPoint {
            particles: Pair(p1, p2),
            dist,
            cmp,
        };
        self.add_constraint(cons);
    }

    pub fn add_fixed_constraint(&mut self, particle: ParticleId, fixed: Vec2, dist: u32, cmp: Cmp) {
        let cons = Projective::ToFixed {
            particle,
            fixed,
            dist,
            cmp,
        };
        self.add_constraint(cons);
    }

    pub fn add_force(&mut self, ind: usize, f: Vec2) {
        self.particles.force[ind].add(f);
    }

    pub fn annul_forces(&mut self, ind: usize) {
        self.particles.force[ind] = Vec2::zero();
    }
}
