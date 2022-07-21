mod utils;

use serde::{Deserialize, Serialize};

use wasm_bindgen::prelude::*;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PANIC_HOOK: () = {
        utils::set_panic_hook();
    };
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2D {
    x: f32,
    y: f32,
}

#[wasm_bindgen]
impl Vec2D {
    pub fn new(x: f32, y: f32) -> Vec2D {
        Vec2D { x, y }
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Particle {
    prev_pos: Vec2D,
    curr_pos: Vec2D,
    mass: f32,
    constraints: Vec<Constraint>,
    forces: Vec2D,
}

impl Particle {
    pub fn new(curr_pos: Vec2D, prev_pos: Vec2D, mass: f32) -> Particle {
        Particle {
            prev_pos,
            curr_pos,
            mass,
            constraints: Vec::new(),
            forces: Vec2D::new(0.0, 0.0),
        }
    }

    pub fn add_constraint(&mut self, c: Constraint) {
        self.constraints.push(c);
    }

    pub fn remove_constraint(&mut self, c: Constraint) {
        let index = self.constraints.iter().position(|i| *i == c);
        if let Some(idx) = index {
            self.constraints.remove(idx);
        } else {
            log!("Attempting to remove non-existent constraint.");
        }
    }

    pub fn add_force(&mut self, f: Vec2D) {
        self.forces.add(f);
    }

    pub fn annul_forces(&mut self) {
        self.forces = Vec2D::new(0.0, 0.0);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    ToPoint { ind: usize, dist: u32 },
    ToFixed { fixed: Vec2D, dist: u32 },
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Scene {
    width: u32,
    height: u32,
    particles: Vec<Particle>,
    timestep: f32,
}

#[wasm_bindgen]
impl Scene {
    pub fn new(width: u32, height: u32) -> Scene {
        let particles = Vec::new();
        // let mut constraints = Vec::new();
        // constraints.push(Constraint::ToFixed {
        //     fixed: Vec2D { x: 0, y: 0 },
        // });
        Scene {
            width,
            height,
            particles,
            timestep: 1.0 / 60.0,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn particles(&self) -> JsValue {
        JsValue::from_serde(&self.particles).unwrap()
    }

    pub fn step(&mut self) {
        for p in self.particles.iter_mut() {
            Scene::verlet_step(self.timestep, p);
        }

        for _ in 0..10 {
            for i in 0..self.particles.len() {
                self.satisfy_constraints(i);

                //let mut v = std::mem::replace(&mut self.particles, vec![]);
                let mut v = std::mem::take(&mut self.particles);

                let p = &mut v[i];
                self.satisfy_constraints2(p);
                self.particles = v;
            }
        }
    }

    pub fn verlet_step(timestep: f32, p: &mut Particle) {
        // integration
        let prev = p.prev_pos;
        let mut curr = p.curr_pos;
        let mut res_force = p.forces;
        let mass = p.mass;
        p.prev_pos = curr;

        // x_{i+1} = 2*x_{i} - x_{i-1} + [ (h*h) * f_i/m_i ]
        res_force.scale(timestep * timestep / mass);
        curr.scale(2.0);
        curr.sub(prev);
        curr.add(res_force);
        p.curr_pos = curr;
    }

    pub fn satisfy_constraints2(&mut self, p: &mut Particle) {
        for c in p.constraints.iter() {
            match *c {
                Constraint::ToPoint { ind, dist } => {
                    let c_pos = self.particles[ind].curr_pos;
                    let (mut delta, d_normed) = Scene::constraint_delta(p, c_pos, dist);
                    delta.scale(0.5 * d_normed);
                    p.curr_pos.sub(delta);
                    self.particles[ind].curr_pos.add(delta)
                }
                Constraint::ToFixed { fixed, dist } => {
                    let (mut delta, d_normed) = Scene::constraint_delta(p, fixed, dist);
                    delta.scale(d_normed);
                    p.curr_pos.sub(delta);
                }
            }
        }
    }

    pub fn satisfy_constraints(&mut self, ind2: usize) {
        //let p = &mut self.particles[ind];
        for c in 0..self.particles[ind2].constraints.len() {
            match self.particles[ind2].constraints[c] {
                Constraint::ToPoint { ind, dist } => {
                    let c_pos = self.particles[ind].curr_pos;
                    let (mut delta, d_normed) =
                        Scene::constraint_delta(&self.particles[ind2], c_pos, dist);
                    delta.scale(0.5 * d_normed);
                    self.particles[ind2].curr_pos.sub(delta);
                    //p.curr_pos.sub(delta);
                    self.particles[ind].curr_pos.add(delta);
                }
                Constraint::ToFixed { fixed, dist } => {
                    let (mut delta, d_normed) =
                        Scene::constraint_delta(&self.particles[ind2], fixed, dist);
                    delta.scale(d_normed);
                    (&self.particles[ind2]).clone().curr_pos.sub(delta);
                }
            }
        }
    }

    fn constraint_delta(p: &Particle, c_pos: Vec2D, c_dist: u32) -> (Vec2D, f32) {
        let dx = p.curr_pos.x - c_pos.x;
        let dy = p.curr_pos.y - c_pos.y;
        //TODO: remove this using scheme in Jakobsens paper
        let d = (dx * dx + dy * dy).sqrt();
        //TODO: make sure distance is positive
        if d == 0. {
            return (Vec2D::new(0., 0.), 0.);
        }
        let d_normed = (d - c_dist as f32) / d;

        (Vec2D::new(dx, dy), d_normed)
    }
}
