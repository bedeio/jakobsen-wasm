mod constraint;
mod particle;

pub use constraint::{Cmp, Pair, Projective};
pub use particle::ParticleManager;
pub use particle::Vec2;
pub use particle::{ConstraintId, ParticleId};
pub use std::sync::Arc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Scene {
    width: u32,
    height: u32,
    manager: ParticleManager,
    timestep: f32,
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Scene {
        let manager = ParticleManager::new();

        Scene {
            width,
            height,
            manager,
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
        2 * self.manager.len()
    }

    pub fn particle_positions(&self) -> *const Vec2 {
        self.manager.particles.curr_pos().as_ptr()
    }

    pub fn add_particle(&mut self, prev_pos: JsValue, curr_pos: JsValue, mass: f32) -> usize {
        let prev: Vec2 = prev_pos.into_serde().unwrap();
        let curr: Vec2 = curr_pos.into_serde().unwrap();
        self.manager.add(prev, curr, mass);

        return self.manager.len() - 1;
    }
    pub fn add_point_constraint(&mut self, p1: ParticleId, p2: ParticleId, dist: u32, cmp: Cmp) {
        let cons = Projective::ToPoint {
            particles: Pair(p1, p2),
            dist,
            cmp,
        };
        self.manager.add_constraint(cons);
    }

    pub fn add_fixed_constraint(
        &mut self,
        particle: ParticleId,
        fixed: JsValue,
        dist: u32,
        cmp: Cmp,
    ) {
        let fixed = fixed.into_serde().unwrap();
        let c = Projective::ToFixed {
            particle,
            fixed,
            dist,
            cmp,
        };
        self.manager.add_constraint(c);
    }

    pub fn set_curr_pos(&mut self, id: ParticleId, pos: JsValue) {
        let pos = pos.into_serde().unwrap();
        let ind = self.manager.identifiers.get(&id).unwrap();
        self.manager.particles.set_curr_pos(*ind, pos);
    }

    pub fn set_prev_pos(&mut self, id: ParticleId, pos: JsValue) {
        let pos = pos.into_serde().unwrap();
        let ind = self.manager.identifiers.get(&id).unwrap();
        self.manager.particles.set_prev_pos(*ind, pos);
    }

    pub fn set_force(&mut self, id: ParticleId, force: JsValue) {
        let force = force.into_serde().unwrap();
        let ind = self.manager.identifiers.get(&id).unwrap();

        self.manager.particles.set_force(*ind, force);
    }

    pub fn step(&mut self) {
        for _ in 0..3 {
            //self.satisfy_constraint_jacobian();
            for i in 0..self.manager.constraints.len() {
                self.satisfy_constraint(i);
            }
        }

        for p in 0..self.manager.len() {
            self.verlet_step(p);
        }
    }
}

impl Scene {
    pub fn verlet_step(&mut self, ind: usize) {
        // integration
        let parts = &mut self.manager.particles;
        let prev = parts.prev_pos()[ind];
        let mut curr = parts.curr_pos()[ind];
        let mut res_force = parts.force()[ind];
        let mass = parts.mass()[ind];
        parts.set_prev_pos(ind, curr);

        // x_{i+1} = 2*x_{i} - x_{i-1} + [ (h*h) * f_i/m_i ]
        res_force.scale(self.timestep * self.timestep / mass);
        curr.scale(2.0);
        curr.sub(prev);
        curr.add(res_force);
        parts.set_curr_pos(ind, curr);
    }

    pub fn satisfy_constraint(&mut self, ind_curr: usize) {
        let cons = &self.manager.constraints;
        let parts = &mut self.manager.particles;

        for c in cons.into_iter() {
            let con = c.1;
            match *con {
                Projective::ToPoint {
                    particles,
                    dist,
                    cmp,
                } => {
                    let ind1 = *self.manager.identifiers.get(&particles.0).unwrap();
                    let ind2 = *self.manager.identifiers.get(&particles.1).unwrap();
                    let cp = parts.curr_pos();

                    let mut delta = Scene::constraint_delta(cp[ind1], cp[ind2], dist, cmp);

                    delta.scale(0.5);
                    parts.sub_curr_pos(ind1, delta);
                    parts.add_curr_pos(ind2, delta);
                }
                Projective::ToFixed {
                    particle,
                    fixed,
                    dist,
                    cmp,
                } => {
                    let ind = *self.manager.identifiers.get(&particle).unwrap();
                    let cp = parts.curr_pos();

                    let delta = Scene::constraint_delta(cp[ind_curr], fixed, dist, cmp);
                    parts.sub_curr_pos(ind, delta);
                }
            }
        }
    }

    fn constraint_delta(pos: Vec2, c_pos: Vec2, c_dist: u32, cmp: Cmp) -> Vec2 {
        let tol = 1e-8;
        let dx = pos.0 - c_pos.0;
        let dy = pos.1 - c_pos.1;
        //TODO: remove this sqrt using scheme in Jakobsens paper
        let d = (dx * dx + dy * dy).sqrt();
        match cmp {
            Cmp::Less => {
                if d < c_dist as f32 {
                    return Vec2::zero();
                }
            }
            Cmp::Equal => {
                if d == c_dist as f32 {
                    return Vec2::zero();
                }
            }
            Cmp::Greater => {
                if d > c_dist as f32 {
                    return Vec2::zero();
                }
            }
        }

        // TODO: move tolerance magic value somewhere else
        if d < tol {
            return Vec2::zero();
        }

        let d_normed = (d - c_dist as f32) / d;

        let mut delta = Vec2::new(dx, dy);
        let scale = d_normed;
        delta.scale(scale);
        delta
    }
}
