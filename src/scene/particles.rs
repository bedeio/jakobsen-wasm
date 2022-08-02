use super::constraints::Cmp;
use super::constraints::Projective;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub fn new(x: f32, y: f32) -> Vec2D {
        Vec2D { x, y }
    }
}

#[wasm_bindgen]
impl Vec2D {
    #[wasm_bindgen(constructor)]
    pub fn new_js(x: f32, y: f32) -> JsValue {
        let v = &Vec2D { x, y };
        JsValue::from_serde(v).unwrap()
    }

    pub fn zero() -> Vec2D {
        Vec2D { x: 0., y: 0. }
    }

    pub fn add(&mut self, other: Vec2D) {
        self.x += other.x;
        self.y += other.y;
    }

    pub fn sub(&mut self, other: Vec2D) {
        self.x -= other.x;
        self.y -= other.y;
    }

    pub fn dot(&mut self, other: Vec2D) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn scale(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct Particles {
    pub(super) prev_pos: Vec<Vec2D>,
    pub(super) curr_pos: Vec<Vec2D>,
    pub(super) mass: Vec<f32>,
    pub(super) constraints: Vec<Vec<Projective>>,
    pub(super) forces: Vec<Vec2D>,
}

impl Particles {
    pub fn new(curr_pos: Vec<Vec2D>, prev_pos: Vec<Vec2D>, mass: Vec<f32>) -> Particles {
        Particles {
            prev_pos,
            curr_pos,
            mass,
            constraints: Vec::new(),
            forces: Vec::new(),
        }
    }

    pub fn add_constraint(&mut self, ind: usize, c: Projective) {
        self.constraints[ind].push(c);
    }

    pub fn remove_constraint(&mut self, ind: usize, c: Projective) {
        let index = self.constraints[ind].iter().position(|i| *i == c);
        if let Some(idx) = index {
            self.constraints.remove(idx);
        } else {
            //log!("Attempting to remove non-existent constraint.");
        }
    }
}

#[wasm_bindgen]
impl Particles {
    pub fn add(&mut self, prev_pos: Vec2D, curr_pos: Vec2D, mass: f32) {
        self.prev_pos.push(prev_pos);
        self.curr_pos.push(curr_pos);
        self.mass.push(mass);
        self.constraints.push(Vec::new());
        self.forces.push(Vec2D::zero());
    }

    pub fn add_point_constraint(&mut self, ind: usize, to_ind: usize, dist: u32, cmp: Cmp) {
        let cons = Projective::ToPoint {
            ind: to_ind,
            dist,
            cmp,
        };
        self.constraints[ind].push(cons);
    }

    pub fn add_fixed_constraint(&mut self, ind: usize, fixed: Vec2D, dist: u32, cmp: Cmp) {
        let cons = Projective::ToFixed { fixed, dist, cmp };
        self.constraints[ind].push(cons);
    }

    pub fn add_force(&mut self, ind: usize, f: Vec2D) {
        self.forces[ind].add(f);
    }

    pub fn annul_forces(&mut self, ind: usize) {
        self.forces[ind] = Vec2D::new(0.0, 0.0);
    }
}
