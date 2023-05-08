use bitvec::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Node {
    pub rotation: u8,
    pub sockets: Sockets,
    pub valid_neighbors: Neighbors,
    pub asset_name: String,
}

impl Node {
    pub fn new(rotation: u8, asset_name: &String) -> Self {
        Self {
            rotation,
            sockets: Sockets {
                px: String::new(),
                nx: String::new(),
                py: String::new(),
                ny: String::new(),
                pz: String::new(),
                nz: String::new(),
            },
            valid_neighbors: Neighbors {
                px: BitVec::new(),
                nx: BitVec::new(),
                py: BitVec::new(),
                ny: BitVec::new(),
                pz: BitVec::new(),
                nz: BitVec::new(),
            },
            asset_name: asset_name.clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Neighbors {
    pub px: BitVec,
    pub nx: BitVec,
    pub py: BitVec,
    pub ny: BitVec,
    pub pz: BitVec,
    pub nz: BitVec,
}

#[derive(Clone, PartialEq)]
pub struct Sockets {
    pub px: String,
    pub nx: String,
    pub py: String,
    pub ny: String,
    pub pz: String,
    pub nz: String,
}
