use super::Vec2D;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Projective {
    ToPoint { ind: usize, dist: u32, cmp: Cmp },
    ToFixed { fixed: Vec2D, dist: u32, cmp: Cmp },
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cmp {
    Less,
    Equal,
    Greater,
}
