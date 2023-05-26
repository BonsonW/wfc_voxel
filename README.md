# Wave Function Collapse for Voxel Input

[<img alt="github" src="https://img.shields.io/badge/github-BonsonW/wfc_voxel-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/BonsonW/wfc_voxel)
[<img alt="crates.io" src="https://img.shields.io/crates/v/wfc_voxel.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/wfc_voxel)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-wfc_voxel-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/wfc_voxel/)

This crate provides a solver for tile-based Wave Function Collapse. It automatically generates rules for map generation from preliminary voxel files. 
Currently, the crate only supports [MagicaVoxel](https://ephtracy.github.io/) `XRAW` file formats.

## Installation

Add to your current working directory:

`cargo add wfc_voxel`

Or add to your Cargo.toml:

`wfc_voxel = "target_version"`

## Usage

```rust
// Initalize NodeData from directory
let node_data = NodeData::new(NODE_SIZE, "path/to/voxel/files", exclusions);

// Initialize Solver
let solver = Solver::new([MAP_WIDTH, MAP_HEIGHT, MAP_WIDTH], node_data.bit_mask(), &node_data, false);

// Get solved map
let map = solver.solve().unwrap();
```

## Examples

See [Isometric Demo](https://github.com/BonsonW/isometric_demo) for an example project.
<img src="https://raw.githubusercontent.com/BonsonW/wfc_voxel/master/assets/preview.gif" alt="Preview"/>