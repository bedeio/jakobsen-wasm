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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Particles {
    prev_pos: Vec<Vec2D>,
    curr_pos: Vec<Vec2D>,
    mass: Vec<f32>,
    #[serde(skip_serializing)]
    constraints: Vec<Vec<Constraint>>,
    forces: Vec<Vec2D>,
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

    pub fn add(&mut self, prev_pos: Vec2D, curr_pos: Vec2D, mass: f32) {
        self.prev_pos.push(prev_pos);
        self.curr_pos.push(curr_pos);
        self.mass.push(mass);
        self.constraints.push(Vec::new());
        self.forces.push(Vec2D::zero());
    }

    pub fn add_constraint(&mut self, ind: usize, c: Constraint) {
        self.constraints[ind].push(c);
    }

    pub fn remove_constraint(&mut self, ind: usize, c: Constraint) {
        let index = self.constraints[ind].iter().position(|i| *i == c);
        if let Some(idx) = index {
            self.constraints.remove(idx);
        } else {
            //log!("Attempting to remove non-existent constraint.");
        }
    }

    pub fn add_force(&mut self, ind: usize, f: Vec2D) {
        self.forces[ind].add(f);
    }

    pub fn annul_forces(&mut self, ind: usize) {
        self.forces[ind] = Vec2D::new(0.0, 0.0);
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
    particles: Particles,
    timestep: f32,
}

#[wasm_bindgen]
impl Scene {
    pub fn new(width: u32, height: u32) -> Scene {
        let mass = Vec::new();
        let curr_pos = Vec::new();
        let prev_pos = Vec::new();
        let particles = Particles::new(curr_pos, prev_pos, mass);
        // let mut constraints = Vec::new();
        // constraints.push(Constraint::ToFixed {
        //     fixed: Vec2D { x: 0, y: 0 },
        // });
        Scene {
            width,
            height,
            particles,
            timestep: 1. / 15.,
        }
    }

    pub fn init_rope(&mut self) {
        let gravity = Vec2D::new(0., 10.);
        let anchor = Vec2D::new(100., 10.);

        let seg_len: u32 = 8;
        for i in 0..20 {
            let x = (i * seg_len) as f32 + anchor.x;
            let y = anchor.y;
            let pos = Vec2D::new(x, y);
            //let mut p = Particle::new(pos, pos, 1.);
            self.particles.add(pos, pos, 1.);
            self.particles.add_force(i as usize, gravity);

            if i == 0 {
                self.particles.add_constraint(
                    i as usize,
                    Constraint::ToFixed {
                        fixed: anchor,
                        dist: seg_len,
                    },
                );
            } else {
                self.particles.add_constraint(
                    i as usize,
                    Constraint::ToPoint {
                        ind: (i - 1) as usize,
                        dist: seg_len,
                    },
                );
            }
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

    pub fn particle_positions(&self) -> *const Vec2D {
        self.particles.curr_pos.as_ptr()
    }

    pub fn step(&mut self) {
        for p in 0..self.particles.mass.len() {
            self.verlet_step(p);
        }

        for _ in 0..5 {
            for i in 0..self.particles.mass.len() {
                self.satisfy_constraints(i);

                // let mut v = std::mem::take(&mut self.particles);
                // let p = &mut v[i];
                // self.satisfy_constraints2(p);
                // self.particles = v;
            }
        }
    }

    pub fn verlet_step(&mut self, ind: usize) {
        // integration
        let prev = self.particles.prev_pos[ind];
        let mut curr = self.particles.curr_pos[ind];
        let mut res_force = self.particles.forces[ind];
        let mass = self.particles.mass[ind];
        self.particles.prev_pos[ind] = curr;

        // x_{i+1} = 2*x_{i} - x_{i-1} + [ (h*h) * f_i/m_i ]
        res_force.scale(self.timestep * self.timestep / mass);
        curr.scale(2.0);
        curr.sub(prev);
        curr.add(res_force);
        self.particles.curr_pos[ind] = curr;
    }

    pub fn satisfy_constraints(&mut self, ind_curr: usize) {
        //let curr_pos = self.particles.curr_pos.clone();
        for c in 0..self.particles.constraints[ind_curr].len() {
            match self.particles.constraints[ind_curr][c] {
                Constraint::ToPoint { ind, dist } => {
                    let (mut delta, d_normed) =
                        self.constraint_delta(ind_curr, self.particles.curr_pos[ind], dist);
                    delta.scale(0.5 * d_normed);
                    self.particles.curr_pos[ind_curr].sub(delta);
                    self.particles.curr_pos[ind].add(delta);
                }
                Constraint::ToFixed { fixed, dist } => {
                    let (mut delta, d_normed) = self.constraint_delta(ind_curr, fixed, dist);
                    delta.scale(d_normed);
                    self.particles.curr_pos[ind_curr].sub(delta);
                }
            }
        }
    }

    fn constraint_delta(&self, ind: usize, c_pos: Vec2D, c_dist: u32) -> (Vec2D, f32) {
        let dx = self.particles.curr_pos[ind].x - c_pos.x;
        let dy = self.particles.curr_pos[ind].y - c_pos.y;
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
