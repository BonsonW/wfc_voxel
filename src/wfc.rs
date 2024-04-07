use std::collections::HashMap;

use ndarray::Array3;
use rand::{rngs::StdRng, thread_rng, Rng, RngCore, SeedableRng};
use bitvec::prelude::*;

use super::{node::Node, NodeSet};

#[derive(Debug)]
pub enum Direction {
    POSX,
    NEGX,
    POSY,
    NEGY,
    POSZ,
    NEGZ,
}

pub static DIRECTIONS: &'static [Direction] = &[
    Direction::POSX,
    Direction::NEGX,
    Direction::POSY,
    Direction::NEGY,
    Direction::POSZ,
    Direction::NEGZ,
];


static OUT_OF_BOUNDS: [usize; 3] = [usize::MAX, usize::MAX, usize::MAX];

/// Solver for the Wave Function Collapse Algorithm.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Solver {
    data: Array3<BitVec>,
    ushape: [usize; 3],
    ishape: [i32; 3],
    node_dict: HashMap<usize, Node>,
    rng: StdRng,
    seed: u64,
}

#[allow(dead_code)]
impl Solver {
    
    /// Creates a new random `Solver` given the `shape` of the map you want to generate.
    /// `init_val` is the value each cell is initialized with. Use the bit mask from your `NodeData` if unsure.
    #[inline]
    pub fn new(shape: [usize; 3], init_val: &BitVec, node_set: &NodeSet) -> Self {
        let ishape = shape.map(|e| e as i32);
        let mut thread_rng = thread_rng();
        let seed = thread_rng.next_u64();
        Self {
            data: Array3::from_elem(shape, init_val.clone()),
            ushape: shape,
            ishape,
            node_dict: node_set.node_dict().clone(),
            rng: StdRng::seed_from_u64(seed),
            seed
        }
    }
    
    /// Creates a new `Solver` given a `seed` as u64 and the `shape` of the map you want to generate.
    /// `init_val` is the value each cell is initialized with. Use the bit mask from your `NodeData` if unsure.
    #[inline]
    pub fn from_seed(shape: [usize; 3], init_val: &BitVec, node_set: &NodeSet, seed: &u64) -> Self {
        let ishape = shape.map(|e| e as i32);
        Self {
            data: Array3::from_elem(shape, init_val.clone()),
            ushape: shape,
            ishape,
            node_dict: node_set.node_dict().clone(),
            rng: StdRng::seed_from_u64(*seed),
            seed: *seed
        }
    }
    
    /// Get the shape of the map.
    #[inline]
    pub fn shape(&self) -> &[usize; 3] {
        &self.ushape
    }
    
    /// Get the seed of the solver.
    #[inline]
    pub fn seed(&self) -> &u64 {
        &self.seed
    }
    
    /// Set the seed of the solver.
    #[inline]
    pub fn set_seed(&mut self, seed: &u64) {
        self.rng = StdRng::seed_from_u64(*seed);
        self.seed = *seed;
    }

    #[inline]
    fn options_at(&self, pos: &[usize; 3]) -> &BitVec {
        &self.data[*pos]
    }

    #[inline]
    fn options_at_mut(&mut self, pos: &[usize; 3]) -> &mut BitVec {
        &mut self.data[*pos]
    }
    
    /// Automatically solves the current map state.
    /// Returns the solved map if successful. Returns `None` if not.
    pub fn solve(&mut self) -> Option<Array3<usize>> {
        let mut ret = Array3::zeros(self.ushape);

        while !self.collapsed() {
            self.iterate();
        }

        for x in 0..self.ushape[0] {
            for y in 0..self.ushape[1] {
                for z in 0..self.ushape[2] {
                    if self.options_at(&[x, y, z]).not_any() {
                        // seed not solvable
                        return None;
                    } else {
                        ret[[x, y, z]] = self.options_at(&[x, y, z]).first_one().unwrap();
                    }
                }
            }
        }

        Some(ret)
    }

    #[inline]
    fn collapsed(&self) -> bool {
        for x in 0..self.ushape[0] {
            for y in 0..self.ushape[1] {
                for z in 0..self.ushape[2] {
                    if self.options_at(&[x, y, z]).count_ones() > 1 {
                        return false;
                    }
                }
            }
        }
        
        true
    }

    #[inline]
    fn iterate(&mut self) {
        let pos = self.get_min_entropy_pos();
        self.collapse_at(&pos);
        self.propagate_from(&pos);
    }
    
    fn collapse_at(&mut self, pos: &[usize; 3]) {
        let options = self.options_at(pos).iter_ones().collect::<Vec<usize>>();
        let to = options[self.rng.gen_range(0..options.len())];

        self.options_at_mut(pos).set_elements(0);
        self.options_at_mut(pos).set(to, true);
    }
    
    fn propagate_from(&mut self, pos: &[usize; 3]) {
        let mut pos_stack = vec![];
        pos_stack.push(*pos);

        while !pos_stack.is_empty() {
            let cur_pos = pos_stack.pop().unwrap();
            for dir in DIRECTIONS {
                let other_pos = self.add_dir_to_pos(&cur_pos, dir);
                if other_pos == OUT_OF_BOUNDS { continue; }

                let options = self.options_at(&other_pos).clone();
                let valid_neighbors = self.valid_neighbors(&cur_pos, dir);
                let mut pushed = false;

                for id in options.iter_ones() {
                    if valid_neighbors.get(id).unwrap() == false {
                        self.constrain(&other_pos, id);

                        if !pushed {
                            pos_stack.push(other_pos);
                            pushed = true;
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn add_dir_to_pos(&self, pos: &[usize; 3], dir: &Direction) -> [usize; 3] {
        let mut ret = pos.map(|e| e as i32);
        
        match dir {
            Direction::POSX => { ret[0] += 1 }
            Direction::NEGX => { ret[0] -= 1 }
            Direction::POSY => { ret[1] += 1 }
            Direction::NEGY => { ret[1] -= 1 }
            Direction::POSZ => { ret[2] += 1 }
            Direction::NEGZ => { ret[2] -= 1 }
        }

        if ret[0] >= self.ishape[0] { return OUT_OF_BOUNDS; }
        if ret[0] < 0               { return OUT_OF_BOUNDS; }
        if ret[1] >= self.ishape[1] { return OUT_OF_BOUNDS; }
        if ret[1] < 0               { return OUT_OF_BOUNDS; }
        if ret[2] >=self. ishape[2] { return OUT_OF_BOUNDS; }
        if ret[2] < 0               { return OUT_OF_BOUNDS; }

        ret.map(|e| e as usize)
    }

    #[inline]
    fn get_min_entropy_pos(&self) -> [usize; 3] {
        let mut min_entropy = usize::MAX;
        let mut ret = [0, 0, 0];

        for x in 0..self.ushape[0] {
            for y in 0..self.ushape[1] {
                for z in 0..self.ushape[2] {

                    let cur_entropy = self.options_at(&[x, y, z]).count_ones();

                    if cur_entropy >= min_entropy { continue; }
                    if cur_entropy < 2 { continue; }

                    ret = [x, y, z];
                    min_entropy = cur_entropy;
                }
            }
        }

        ret
    }

    #[inline]
    fn valid_neighbors(&self, pos: &[usize; 3], dir: &Direction) -> BitVec {
        let mut ret = BitVec::new();
        ret.resize(self.node_dict.len(), false);

        for id in self.options_at(pos).iter_ones() {
            let node = &self.node_dict[&id];
            
            match dir {
                Direction::POSX => { ret |= &node.valid_neighbors.px; }
                Direction::NEGX => { ret |= &node.valid_neighbors.nx; }
                Direction::POSY => { ret |= &node.valid_neighbors.py; }
                Direction::NEGY => { ret |= &node.valid_neighbors.ny; }
                Direction::POSZ => { ret |= &node.valid_neighbors.pz; }
                Direction::NEGZ => { ret |= &node.valid_neighbors.nz; }
            }
        }

        ret
    }

    #[inline]
    fn valid_neighbors_of_set(&self, node_ids: &BitVec, dir: &Direction) -> BitVec {
        let mut ret = BitVec::new();
        ret.resize(self.node_dict.len(), false);

        for id in node_ids.iter_ones() {
            let node = &self.node_dict[&id];
            
            match dir {
                Direction::POSX => { ret |= &node.valid_neighbors.px; }
                Direction::NEGX => { ret |= &node.valid_neighbors.nx; }
                Direction::POSY => { ret |= &node.valid_neighbors.py; }
                Direction::NEGY => { ret |= &node.valid_neighbors.ny; }
                Direction::POSZ => { ret |= &node.valid_neighbors.pz; }
                Direction::NEGZ => { ret |= &node.valid_neighbors.nz; }
            }
        }

        ret
    }
    
    /// Constrain the possible nodes at a specifc cell in the grid.
    pub fn constrain_list(&mut self, pos: &[usize; 3], bits: &BitVec) {
        for id in bits.iter_ones() {
            self.options_at_mut(pos).set(id, false);
        }
        self.propagate_from(pos);
    }
    
    #[inline]
    fn constrain(&mut self, pos: &[usize; 3], id: usize) {
        self.options_at_mut(pos).set(id, false);
    }
    
    /// Constrain the possible nodes at a specifc cell in the grid based on a set of neighbours you want for a specific direction.
    pub fn force_neighbor(&mut self, pos: &[usize; 3], bits: &BitVec, dir: &Direction) {
        let valid_neighbors = self.valid_neighbors_of_set(bits, dir);
        
        for id in self.options_at(pos).iter_ones().collect::<Vec<usize>>() {
            if valid_neighbors.get(id).unwrap() == false {
                self.constrain(pos, id);
            }
        }
        self.propagate_from(pos);
    }
}