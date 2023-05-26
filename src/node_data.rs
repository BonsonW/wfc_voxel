use std::collections::{HashMap, HashSet};

use bitvec::prelude::*;

use super::voxel;
use super::node::Node;

/// Contains mapping to all `Node`s and asset bit masks. `Node`s contain rules and metadata for each asset rotation.
#[derive(Clone)]
pub struct NodeData {
    node_dict: HashMap<usize, Node>,
    bit_mask: BitVec,
    asset_bits: HashMap<String, BitVec>,
}

impl NodeData {
    
    /// Creates a new `NodeData` instance given the directory to all voxel files. A `Node` is generated for each voxel file rotation around the `Y` axis.
    /// `node_size` is the array length of each voxel file. This should be uniform across dimensions and voxel files.
    /// `exclusions` is the list of asset mappings that you don't want connected.
    pub fn new(node_size: usize, directory: String, exclusions: HashSet<(&str, &str)>) -> NodeData {
        let node_dict = voxel::node_dict_from_directory(&directory, [node_size, node_size, node_size], &exclusions);
        
        let mut asset_bits = HashMap::new();
        let mut bit_mask = BitVec::new();
        bit_mask.resize(node_dict.len(), false);
        
        for (id, node) in &node_dict {
            asset_bits.entry(node.asset_name.clone()).or_insert(bit_mask.clone());
            asset_bits.get_mut(&node.asset_name).unwrap().set(*id, true);
        }
        
        bit_mask.fill(true);

        NodeData {
            node_dict,
            asset_bits,
            bit_mask,
        }
    }
    
    /// The dictionary to every `Node`.
    pub fn node_dict(&self) -> &HashMap<usize, Node> {
        &self.node_dict
    }
    
    /// A bit mask in the length of all `Node`s.
    pub fn bit_mask(&self) -> &BitVec {
        &self.bit_mask
    }
    
    /// The rotation of a `Node`.
    pub fn get_rotation(&self, node_id: &usize) -> Option<u8> {
        if let Some(node) = self.node_dict.get(node_id) {
            return Some(node.rotation);
        }
        None
    }
    
    /// The name of the asset a `Node` represents.
    pub fn get_asset_name(&self, node_id: &usize) -> Option<&String> {
        if let Some(node) = self.node_dict.get(node_id) {
            return Some(&node.asset_name);
        }
        None
    }
    
    /// The bit mask for all the `Node`s of a specific asset.
    pub fn asset_bits(&self, asset: &String) -> Option<&BitVec> {
        self.asset_bits.get(asset)
    }
}
