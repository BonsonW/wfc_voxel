use std::collections::HashMap;

use ndarray::Array3;
use rand::Rng;
use bitvec::prelude::*;

use super::{node::Node, NodeData};

pub static POS_X: &'static [i32; 3] = &[1, 0, 0];
pub static NEG_X: &'static [i32; 3] = &[-1, 0, 0];
pub static POS_Y: &'static [i32; 3] = &[0, 1, 0];
pub static NEG_Y: &'static [i32; 3] = &[0, -1, 0];
pub static POS_Z: &'static [i32; 3] = &[0, 0, 1];
pub static NEG_Z: &'static [i32; 3] = &[0, 0, -1];

pub static DIRECTIONS: &'static [[i32; 3]] = &[
    *POS_X,
    *NEG_X,
    *POS_Y,
    *NEG_Y,
    *POS_Z,
    *NEG_Z,
];


static OUT_OF_BOUNDS: [usize; 3] = [9999, 9999, 9999];

/// Solver for the Wave Function Collapse Algorithm.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Solver {
    data: Array3<BitVec>,
    shape: [usize; 3],
    node_dict: HashMap<usize, Node>,
    wrap: bool
}

#[allow(dead_code)]
impl Solver {
    
    /// Creates a new `Solver` given the `shape` of the map you want to generate.
    /// `init_val` is the value each cell is initialized with. Use the bit mask from your `NodeData` if unsure.
    /// `wrap` determines if you want edge nodes to join with nodes on the opposite edge.
    #[inline]
    pub fn new(shape: [usize; 3], init_val: &BitVec, node_data: &NodeData, wrap: bool) -> Self {
        Self {
            data: Array3::from_elem(shape, init_val.clone()),
            shape,
            node_dict: node_data.node_dict().clone(),
            wrap
        }
    }
    
    /// Get the shape of the map.
    #[inline]
    pub fn shape(&self) -> &[usize; 3] {
        &self.shape
    }

    #[inline]
    fn options_at(&self, pos: &[usize; 3]) -> &BitVec {
        &self.data[*pos]
    }

    #[inline]
    fn options_at_mut(&mut self, pos: &[usize; 3]) -> &mut BitVec {
        &mut self.data[*pos]
    }
    
    /// Automatically solves the current map state. Returns the solved map.
    pub fn solve(&mut self) -> Array3<usize> {
        let mut ret = Array3::zeros(self.shape);

        let mut h = 0;
        while !self.collapsed() {
            if h > 9999 {
                println!("wfc took too long");
                return ret;
            }
            h += 1;
            self.iterate();
        }

        for x in 0..self.shape[0] {
            for y in 0..self.shape[1] {
                for z in 0..self.shape[2] {
                    if self.options_at(&[x, y, z]).not_any() {
                        println!("unsolvable seed given");
                        return ret;
                    } else {
                        ret[[x, y, z]] = self.options_at(&[x, y, z]).first_one().unwrap();
                    }
                }
            }
        }

        ret
    }

    #[inline]
    fn collapsed(&self) -> bool {
        for x in 0..self.shape[0] {
            for y in 0..self.shape[1] {
                for z in 0..self.shape[2] {
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
        let mut rng = rand::thread_rng();

        let options = self.options_at(pos).iter_ones().collect::<Vec<usize>>();
        let to = options[rng.gen_range(0..options.len())];

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
    fn add_dir_to_pos(&self, pos: &[usize; 3], dir: &[i32; 3]) -> [usize; 3] {
        let mut ret = pos.map(|e| e as i32);
        let shape = self.shape.map(|e| e as i32);

        ret[0] += dir[0];
        ret[1] += dir[1];
        ret[2] += dir[2];

        if self.wrap {
            if ret[0] >= shape[0] { ret[0] = 0; }
            if ret[0] < 0         { ret[0] = shape[0] - 1; }
            if ret[1] >= shape[1] { return OUT_OF_BOUNDS; }
            if ret[1] < 0         { return OUT_OF_BOUNDS; }
            if ret[2] >= shape[2] { ret[2] = 0; }
            if ret[2] < 0         { ret[2] = shape[2] - 1; }
        } else {
            if ret[0] >= shape[0] { return OUT_OF_BOUNDS; }
            if ret[0] < 0         { return OUT_OF_BOUNDS; }
            if ret[1] >= shape[1] { return OUT_OF_BOUNDS; }
            if ret[1] < 0         { return OUT_OF_BOUNDS; }
            if ret[2] >= shape[2] { return OUT_OF_BOUNDS; }
            if ret[2] < 0         { return OUT_OF_BOUNDS; }
        }

        ret.map(|e| e as usize)
    }

    #[inline]
    fn get_min_entropy_pos(&self) -> [usize; 3] {
        let mut min_entropy = usize::MAX;
        let mut ret = [0, 0, 0];

        for x in 0..self.shape[0] {
            for y in 0..self.shape[1] {
                for z in 0..self.shape[2] {

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
    fn valid_neighbors(&self, pos: &[usize; 3], dir: &[i32; 3]) -> BitVec {
        let mut ret = BitVec::new();
        ret.resize(self.node_dict.len(), false);

        for id in self.options_at(pos).iter_ones() {
            let node = &self.node_dict[&id];

            if      dir == POS_X { ret |= &node.valid_neighbors.px; }
            else if dir == NEG_X { ret |= &node.valid_neighbors.nx; }
            else if dir == POS_Y { ret |= &node.valid_neighbors.py; }
            else if dir == NEG_Y { ret |= &node.valid_neighbors.ny; }
            else if dir == POS_Z { ret |= &node.valid_neighbors.pz; }
            else if dir == NEG_Z { ret |= &node.valid_neighbors.nz; }
            else { panic!("{} {} {} is an invalid direction", dir[0], dir[1], dir[2]); }
        }

        ret
    }

    #[inline]
    fn valid_neighbors_of_set(&self, node_ids: &BitVec, dir: &[i32; 3]) -> BitVec {
        let mut ret = BitVec::new();
        ret.resize(self.node_dict.len(), false);

        for id in node_ids.iter_ones() {
            let node = &self.node_dict[&id];

            if      dir == POS_X { ret |= &node.valid_neighbors.px; }
            else if dir == NEG_X { ret |= &node.valid_neighbors.nx; }
            else if dir == POS_Y { ret |= &node.valid_neighbors.py; }
            else if dir == NEG_Y { ret |= &node.valid_neighbors.ny; }
            else if dir == POS_Z { ret |= &node.valid_neighbors.pz; }
            else if dir == NEG_Z { ret |= &node.valid_neighbors.nz; }
            else { panic!("{} {} {} is an invalid direction", dir[0], dir[1], dir[2]); }
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
    pub fn force_neighbor(&mut self, pos: &[usize; 3], bits: &BitVec, dir: &[i32; 3]) {
        let valid_neighbors = self.valid_neighbors_of_set(bits, dir);
        
        for id in self.options_at(pos).iter_ones().collect::<Vec<usize>>() {
            if valid_neighbors.get(id).unwrap() == false {
                self.constrain(pos, id);
            }
        }
        self.propagate_from(pos);
    }
}