mod constraints;
mod particles;

pub use constraints::Cmp;
pub use constraints::Projective;
pub use particles::Particles;
pub use particles::Vec2D;
pub use std::sync::Arc;

use wasm_bindgen::prelude::*;

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
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Scene {
        let mass = Vec::new();
        let curr_pos = Vec::new();
        let prev_pos = Vec::new();
        let particles = Particles::new(curr_pos, prev_pos, mass);

        Scene {
            width,
            height,
            particles,
            timestep: 1. / 60.,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn particles_length(&self) -> usize {
        //multiply by 2 since we are in 2 dimensions
        2 * self.particles.mass.len()
    }

    pub fn particle_positions(&self) -> *const Vec2D {
        self.particles.curr_pos.as_ptr()
    }

    pub fn add_particle(&mut self, prev_pos: JsValue, curr_pos: JsValue, mass: f32) -> usize {
        let prev: Vec2D = prev_pos.into_serde().unwrap();
        let curr: Vec2D = curr_pos.into_serde().unwrap();
        self.particles.add(prev, curr, mass);
        return self.particles.mass.len() - 1;
    }

    pub fn add_point_constraint(&mut self, ind: usize, to_ind: usize, dist: u32, cmp: Cmp) {
        let cons = Projective::ToPoint {
            ind: to_ind,
            dist,
            cmp,
        };
        self.particles.constraints[ind].push(cons);
    }

    pub fn add_fixed_constraint(&mut self, ind: usize, fixed: JsValue, dist: u32, cmp: Cmp) {
        let fixed = fixed.into_serde().unwrap();
        let cons = Projective::ToFixed { fixed, dist, cmp };
        self.particles.constraints[ind].push(cons);
    }

    pub fn set_curr_pos(&mut self, ind: usize, pos: JsValue) {
        let pos = pos.into_serde().unwrap();
        self.particles.curr_pos[ind] = pos;
    }

    pub fn set_prev_pos(&mut self, ind: usize, pos: JsValue) {
        let pos = pos.into_serde().unwrap();
        self.particles.prev_pos[ind] = pos;
    }

    pub fn set_force(&mut self, ind: usize, force: JsValue) {
        let force = force.into_serde().unwrap();
        self.particles.forces[ind] = force;
    }

    pub fn step(&mut self) {
        for _ in 0..3 {
            //self.satisfy_constraints_jacobian();
            for i in 0..self.particles.mass.len() {
                self.satisfy_constraints(i);
            }
        }

        for p in 0..self.particles.mass.len() {
            self.verlet_step(p);
        }
    }
}

impl Scene {
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
        let cons = &self.particles.constraints;
        let curr = &mut self.particles.curr_pos;

        for c in 0..cons[ind_curr].len() {
            let con = &cons[ind_curr][c];
            match *con {
                Projective::ToPoint { ind, dist, cmp } => {
                    let mut delta = Scene::constraint_delta(curr[ind_curr], curr[ind], dist, cmp);

                    delta.scale(0.5);
                    curr[ind_curr].sub(delta);

                    curr[ind].add(delta);
                }
                Projective::ToFixed { fixed, dist, cmp } => {
                    let delta = Scene::constraint_delta(curr[ind_curr], fixed, dist, cmp);
                    curr[ind_curr].sub(delta);
                }
            }
        }
    }

    fn constraint_delta(pos: Vec2D, c_pos: Vec2D, c_dist: u32, cmp: Cmp) -> Vec2D {
        let dx = pos.x - c_pos.x;
        let dy = pos.y - c_pos.y;
        //TODO: remove this sqrt using scheme in Jakobsens paper
        let d = (dx * dx + dy * dy).sqrt();
        match cmp {
            Cmp::Less => {
                if d < c_dist as f32 {
                    return Vec2D::zero();
                }
            }
            Cmp::Equal => {
                if d == c_dist as f32 {
                    return Vec2D::zero();
                }
            }
            Cmp::Greater => {
                if d > c_dist as f32 {
                    return Vec2D::zero();
                }
            }
        }

        // TODO: move tolerance magic value somewhere else
        if d < 10e-6 {
            return Vec2D::zero();
        }

        let d_normed = (d - c_dist as f32) / d;

        let mut delta = Vec2D::new(dx, dy);
        let scale = d_normed;
        delta.scale(scale);
        delta
    }
}
