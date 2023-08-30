use super::voxel_octree::MeshData;
use super::voxel_octree::*;

pub fn get_dual_contour(octree: &VoxelOctree, _start_pos: &[f32; 3]) -> MeshData {
  let positions = Vec::new();
  let normals = Vec::new();
  let uvs = Vec::new();
  let indices = Vec::new();

  // Checking for each grid
  for _x in 0..octree.get_size() {
    for _y in 0..octree.get_size() {
      for _z in 0..octree.get_size() {
      }
    }
  }
  MeshData {
    positions: positions,
    normals: normals,
    uvs: uvs,
    indices: indices,
    weights: Vec::new(),
    types_1: Vec::new(),
    types_2: Vec::new(),
    voxel_positions: Vec::new(),
    colors: Vec::new(),
  }
}
