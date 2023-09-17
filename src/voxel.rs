use std::collections::{HashMap, HashSet};
use std::fs;
use std::str;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;

use ndarray::{Array3, Array2, Axis, s};

use crate::node::{Node, Sockets};

pub fn node_dict_from_directory(asset_dir: &String, shape: [usize; 3], exclusions: &HashSet<(&str, &str)>) -> HashMap<usize, Node> {
    let mut ret = HashMap::<usize, Node>::new();

    let mut side_socket_map = HashMap::<Array2<u8>, String>::new();
    let mut vert_socket_map = HashMap::<Array2<u8>, String>::new();
    let mut socket_serial:usize = 0;
    let mut node_serial:usize = 0;

    let paths = fs::read_dir(asset_dir).unwrap();

    for path in paths {
        let entry = path.unwrap();
        let vox_array = vox_array_from_xraw(entry.path().to_str().unwrap());
        let file_name = entry.file_name().into_string().unwrap();
        let asset_name = &String::from(&file_name[..file_name.len()-5]);

        // Create reference node
        let face_ny = vox_array.index_axis(Axis(0), 0).to_owned();
        let face_py = vox_array.index_axis(Axis(0), shape[0]-1).to_owned();
        let face_nx = vox_array.index_axis(Axis(1), 0).to_owned();
        let face_px = vox_array.index_axis(Axis(1), shape[1]-1).to_owned();
        let face_nz = vox_array.index_axis(Axis(2), 0).to_owned();
        let face_pz = vox_array.index_axis(Axis(2), shape[2]-1).to_owned();

        let mut ori_node = Node::new(0, asset_name);

        register_side_face(&mut side_socket_map, mirrored_face(&face_nx), &mut socket_serial, &mut ori_node.sockets.nx);
        register_side_face(&mut side_socket_map, face_px, &mut socket_serial, &mut ori_node.sockets.px);
        register_side_face(&mut side_socket_map, face_nz, &mut socket_serial, &mut ori_node.sockets.nz);
        register_side_face(&mut side_socket_map, mirrored_face(&face_pz), &mut socket_serial, &mut ori_node.sockets.pz);

        register_vert_face(&mut vert_socket_map, face_py.clone(), &mut socket_serial, &mut ori_node.sockets.py);
        register_vert_face(&mut vert_socket_map, face_ny.clone(), &mut socket_serial, &mut ori_node.sockets.ny);

        for rot in 1..4 {
            let mut rot_node = Node::new(rot, asset_name);
            rotate_side_sockets(&ori_node.sockets, &mut rot_node.sockets, rot);
            rotate_vert_socket(&ori_node.sockets.py, &mut rot_node.sockets.py, rot);
            rotate_vert_socket(&ori_node.sockets.ny, &mut rot_node.sockets.ny, rot);

            ret.insert(node_serial, rot_node);
            node_serial += 1;
        }

        ret.insert(node_serial, ori_node);
        node_serial += 1;
    }

    let node_map_cpy = ret.clone();

    // Find valid neighbors
    for (_, node) in &mut ret {
        node.valid_neighbors.px.resize(node_map_cpy.len(), false);
        node.valid_neighbors.nx.resize(node_map_cpy.len(), false);
        node.valid_neighbors.pz.resize(node_map_cpy.len(), false);
        node.valid_neighbors.nz.resize(node_map_cpy.len(), false);
        node.valid_neighbors.py.resize(node_map_cpy.len(), false);
        node.valid_neighbors.ny.resize(node_map_cpy.len(), false);
        
        for (other_id, other_node) in &node_map_cpy {
            if exclusions.contains(&(node.asset_name.as_str(), other_node.asset_name.as_str())) {
                continue;
            }
            if socket_matches(&node.sockets.px, &other_node.sockets.nx) { node.valid_neighbors.px.set(*other_id, true); }
            if socket_matches(&node.sockets.nx, &other_node.sockets.px) { node.valid_neighbors.nx.set(*other_id, true); }
            if socket_matches(&node.sockets.pz, &other_node.sockets.nz) { node.valid_neighbors.pz.set(*other_id, true); }
            if socket_matches(&node.sockets.nz, &other_node.sockets.pz) { node.valid_neighbors.nz.set(*other_id, true); }
            if socket_matches(&node.sockets.py, &other_node.sockets.ny) { node.valid_neighbors.py.set(*other_id, true); }
            if socket_matches(&node.sockets.ny, &other_node.sockets.py) { node.valid_neighbors.ny.set(*other_id, true); }
        }
    }

    ret
}

#[inline]
pub fn vox_array_from_xraw(path: &str) -> Array3<u8> {
    let f = File::open(path)
        .expect(&format!("could not open specified file {}", path));

    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)
        .expect(&format!("could not read specified file {}", path));

    let magic = str::from_utf8(&buffer[0..4]).unwrap();
    assert_eq!(magic, "XRAW");

    let bits_per_index = buffer[7];
    assert_eq!(bits_per_index, 8);

    let width = usize_from_bits(&buffer[8..12]);
    let height = usize_from_bits(&buffer[12..16]);
    let depth = usize_from_bits(&buffer[16..20]);

    let data = &buffer[24..(24 + (width * height * depth) as usize)];

    Array3::from_shape_vec((width, height, depth), data.to_vec()).unwrap()
}

#[inline]
fn socket_matches (a: &String, b: &String) -> bool {
    let a_last = a.chars().last().unwrap();
    let b_last = b.chars().last().unwrap();

    if (a_last == 'f' && b_last == 'm' || a_last == 'm' && b_last == 'f') && a[..a.len()-1] == b[..b.len()-1] {
        return true;
    } else if a == b && a_last != 'f' && b_last != 'f' && a_last != 'm' && b_last != 'm' {
        return true;
    }

    false
}

#[inline]
fn rotate_side_sockets(original: &Sockets, sockets: &mut Sockets, rotation: u8) {
    match rotation {
        1 => {
            sockets.px = original.pz.clone();
            sockets.nx = original.nz.clone();
            sockets.pz = original.nx.clone();
            sockets.nz = original.px.clone();
        },
        2 => {
            sockets.px = original.nx.clone();
            sockets.nx = original.px.clone();
            sockets.pz = original.nz.clone();
            sockets.nz = original.pz.clone();
        },
        3 => {
            sockets.px = original.nz.clone();
            sockets.nx = original.pz.clone();
            sockets.pz = original.px.clone();
            sockets.nz = original.nx.clone();
        },
        _ => panic!("{} is an invalid number of rotations", rotation)
    }
}

#[inline]
fn rotate_vert_socket(original: &String, socket: &mut String, rotation: u8) {
    let original_rot = original.chars().last().unwrap();
    *socket = original.clone();
    if original_rot != 'i' {
        let mut new_rot = original_rot.to_digit(10).unwrap() as u8;
        new_rot += rotation;
        if new_rot > 3 {
            new_rot -= 4;
        }
        socket.replace_range(socket.len()-1..socket.len(), &new_rot.to_string());
    }
}

#[inline]
fn register_vert_face(socket_map: &mut HashMap::<Array2<u8>, String>, face: Array2<u8>, serial: &mut usize, socket: &mut String) {
    if !socket_map.contains_key(&face) {
        let rot_0 = face.clone();
        let rot_1 = rotated_array_p90(&rot_0);
        let rot_2 = rotated_array_p90(&rot_1);
        let rot_3 = rotated_array_p90(&rot_2);

        if rot_0 == rot_1 && rot_1 == rot_2 && rot_2 == rot_3 {
            socket_map.insert(rot_0, serial.to_string() + "_i");
        } else {
            socket_map.insert(rot_0, serial.to_string() + "_0");
            socket_map.insert(rot_1, serial.to_string() + "_1");
            socket_map.insert(rot_2, serial.to_string() + "_2");
            socket_map.insert(rot_3, serial.to_string() + "_3");
        }

        *serial += 1;
    }
    *socket = socket_map[&face].clone();
}

#[inline]
fn register_side_face(socket_map: &mut HashMap::<Array2<u8>, String>, face: Array2<u8>, serial: &mut usize, socket: &mut String) {
    if !socket_map.contains_key(&face) {
        let mirror = mirrored_face(&face);

        if face == mirror {
            socket_map.insert(mirror, serial.to_string() + "s");
        } else {
            socket_map.insert(mirror, serial.to_string() + "f");
            socket_map.insert(face.clone(), serial.to_string() + "m");
        }

        *serial += 1;
    }
    *socket = socket_map[&face].clone();
}

#[inline]
fn rotated_array_p90(arr: &Array2<u8>) -> Array2<u8> {
    let mut ret = arr.clone();
    ret = ret.reversed_axes();
    ret = ret.slice(s![0..ret.shape()[0]; 1, ..; -1]).to_owned();
    
    ret
}

#[inline]
fn usize_from_bits(array: &[u8]) -> usize {
    ((array[0] as usize) <<  0) +
    ((array[1] as usize) <<  8) +
    ((array[2] as usize) << 16) +
    ((array[3] as usize) << 24)
}

#[inline]
fn mirrored_face(face: &Array2<u8>) -> Array2<u8> {
    face.slice(s![0..face.shape()[0]; 1, ..; -1]).to_owned()
}
