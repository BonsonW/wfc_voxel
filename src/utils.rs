use bitvec::prelude::BitVec;

use crate::{Direction, Solver};

/// Constrains the entire `X` axis of a map to the supplied bits.
pub fn constrain_x_axis(wfc_solver: &mut Solver, bits: &BitVec, mut x: i32) {
    if x < 0 { x = wfc_solver.shape()[0] as i32 - 1; }

    for y in 0..wfc_solver.shape()[1] {
        for z in 0..wfc_solver.shape()[2] {

            wfc_solver.constrain_list(&[x as usize, y, z], bits);
        }
    }
}

/// Collapses an area on the `X` axis of a map based by forcing the neighbors of cells in the specified area.
pub fn collapse_x_axis(wfc_solver: &mut Solver, bits: &BitVec, mut x: i32, dir: &Direction, y_shape: [usize; 2], z_shape: [usize; 2]) {
    if x < 0 { x = wfc_solver.shape()[0] as i32 - 1; }

    for y in y_shape[0]..y_shape[1] {
        for z in z_shape[0]..z_shape[1] {

            wfc_solver.force_neighbor(&[x as usize, y, z], bits, dir);
        }
    }
}

/// Constrains the entire `Y` axis of a map to the supplied bits.
pub fn constrain_y_axis(wfc_solver: &mut Solver, bits: &BitVec, mut y: i32) {
    if y < 0 { y = wfc_solver.shape()[1] as i32 - 1; }

    for x in 0..wfc_solver.shape()[0] {
        for z in 0..wfc_solver.shape()[2] {

            wfc_solver.constrain_list(&[x, y as usize, z], bits);
        }
    }
}

/// Collapses an area on the `Y` axis of a map based by forcing the neighbors of cells in the specified area.
pub fn collapse_y_axis(wfc_solver: &mut Solver, bits: &BitVec, mut y: i32, dir: &Direction, x_shape: [usize; 2], z_shape: [usize; 2]) {
    if y < 0 { y = wfc_solver.shape()[1] as i32 - 1; }

    for x in x_shape[0]..x_shape[1] {
        for z in z_shape[0]..z_shape[1] {

            wfc_solver.force_neighbor(&[x, y as usize, z], bits, dir);
        }
    }
}

/// Constrains the entire `Z` axis of a map to the supplied bits.
pub fn constrain_z_axis(wfc_solver: &mut Solver, bits: &BitVec, mut z: i32) {
    if z < 0 { z = wfc_solver.shape()[2] as i32 - 1; }

    for x in 0..wfc_solver.shape()[0] {
        for y in 0..wfc_solver.shape()[1] {

            wfc_solver.constrain_list(&[x, y, z as usize], bits);
        }
    }
}

/// Collapses an area on the `Z` axis of a map based by forcing the neighbors of cells in the specified area.
pub fn collapse_z_axis(wfc_solver: &mut Solver, bits: &BitVec, mut z: i32, dir: &Direction, y_shape: [usize; 2], x_shape: [usize; 2]) {
    if z < 0 { z = wfc_solver.shape()[2] as i32 - 1; }

    for x in x_shape[0]..x_shape[1] {
        for y in y_shape[0]..y_shape[1] {

            wfc_solver.force_neighbor(&[x, y, z as usize], bits, dir);
        }
    }
}