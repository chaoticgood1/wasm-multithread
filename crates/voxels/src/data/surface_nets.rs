use parry3d::math::Point;
use crate::utils::{coord_to_index, get_len_by_size};
use super::voxel_octree::*;
use crate::data::CUBE_EDGES;

const _CURRENT: [i8; 3] = [0, 0, 0];
const _RIGHT: [i8; 3] = [-1, 0, 0];
const _DOWN: [i8; 3] = [0, -1, 0];

const _BACK: [i8; 3] = [0, 0, -1];
const _BACK_DOWN: [i8; 3] = [0, -1, -1];

const _RIGHT_DOWN: [i8; 3] = [-1,-1, 0];
const _RIGHT_BACK: [i8; 3] = [-1, 0,-1];


#[derive(Clone)]
pub struct GridPosition {
  pub index: u32,
  pub pos: Option<[f32; 3]>,
}

impl Default for GridPosition {
  fn default() -> Self {
    GridPosition { index: u32::MAX, pos: None }
  }
}

/** 
 * Fastest way(as far as I know) to get voxel value at the cost of using memory usage as it 
 * is putting everything in one-dimensinal array
 */
#[derive(Clone)]
pub struct VoxelReuse {
  pub voxels: Vec<u8>,
  pub grid_pos: Vec<GridPosition>,
  pub size: u32,
}

impl VoxelReuse {
  pub fn new(depth: u32, loop_count: u32) -> Self {
    let size = (2 as u32).pow(depth as u32);
    let len = get_len_by_size(size, loop_count);
    let voxels = vec![0; len];
    
    let grid_pos_len = get_len_by_size(size - 1, loop_count);
    let grid_pos = vec![GridPosition::default(); grid_pos_len];

    VoxelReuse {
      voxels: voxels,
      grid_pos: grid_pos,
      size: size,
    }
  }
}

impl Default for VoxelReuse {
  fn default() -> Self {
    VoxelReuse::new(4, 3)
  }
}

#[derive(Default)]
struct Grid {
  pub pos: Option<[f32; 3]>,
  normal: [f32; 3],
  // weights: [f32; 4],
  // types: [u32; 4],
  types: [u32; 8],
  voxel_count: u8,
}

struct Layout {
  grids: Vec<Grid>,
  size: u32,
}

impl Layout {
  pub fn new(size: u32) -> Self {
    let mut grids = Vec::new();
    let len = get_len_by_size(size, 3);
    for _ in 0..len {
      grids.push(Grid::default());
    }
    Self {
      grids: grids,
      size: size,
    }
  }
}

pub fn get_surface_nets(
  octree: &VoxelOctree, 
  voxel_reuse: &mut VoxelReuse,
  colors: &Vec<[f32; 3]>,
  scale: f32,
) -> MeshData {
  let voxel_start = 0;
  let voxel_end = octree.get_size();
  for x in voxel_start..voxel_end {
    for y in voxel_start..voxel_end {
      for z in voxel_start..voxel_end {
        let voxel = octree.get_voxel(x, y, z);

        let index = coord_to_index(x, y, z, voxel_start, voxel_end);
        voxel_reuse.voxels[index] = voxel;
      }
    }
  }

  let mut data = MeshData::default();

  // Checking for each grid
  let start = 0;
  let end = octree.get_size() - 1;
  let mut layout = Layout::new(end);

  for x in start..end {
    for y in start..end {
      for z in start..end {
        init_grid(&mut layout, voxel_reuse, x, y, z, scale);
        detect_face_x(&mut data, &mut layout, voxel_reuse, x, y, z, colors);
        detect_face_y(&mut data, &mut layout, voxel_reuse, x, y, z, colors);
        detect_face_z(&mut data, &mut layout, voxel_reuse, x, y, z, colors);
      }
    }
  }

  data
}


fn init_grid(
  layout: &mut Layout, 
  voxel_reuse: &mut VoxelReuse, 
  x: u32, 
  y: u32, 
  z: u32,
  scale: f32,
) {
  let mut voxel_count = 0;
  let mut dists = [1.0; 8];

  // let mut voxels = [0; 4];
  let mut voxels = [0; 8];
  let mut voxel_index = 0;
  for x_offset in 0..2 {
    for y_offset in 0..2 {
      for z_offset in 0..2 {
        let corner_x = x_offset + x;
        let corner_y = y_offset + y;
        let corner_z = z_offset + z;
        
        let index = coord_to_index(corner_x, corner_y, corner_z, 0, voxel_reuse.size);
        if index >= voxel_reuse.voxels.len() {
          continue;
        }
        let voxel = voxel_reuse.voxels[index];
        if voxel > 0 {
          let x_index = x_offset;
          let y_index = y_offset << 1;
          let z_index = z_offset << 2;
          let corner_index = x_index + y_index + z_index;
          dists[corner_index as usize] = -1.0;
          voxel_count += 1;

          // let surrounding_voxel_limit = 4;
          // if voxel_index < surrounding_voxel_limit {
            // voxels[voxel_index] = voxel as u32;
          // }
          voxels[voxel_index] = voxel as u32;
          
          voxel_index += 1;
        }
      }
    }
  }

  let grid_index = coord_to_index(x, y, z, 0, layout.size);
  if voxel_count > 0 && voxel_count < 8 {
    let mut count = 0;
    let mut sum = [0.0, 0.0, 0.0];
    for (offset1, offset2) in CUBE_EDGES.iter() {
      if let Some(intersection) =
        estimate_surface_edge_intersection(*offset1, *offset2, dists[*offset1], dists[*offset2])
      {
        count += 1;
        sum[0] += intersection[0];
        sum[1] += intersection[1];
        sum[2] += intersection[2];
      }
    }
    let pos_x = (sum[0] / count as f32 + x as f32) * scale;
    let pos_y = (sum[1] / count as f32 + y as f32) * scale;
    let pos_z = (sum[2] / count as f32 + z as f32) * scale;
    let avg_pos = Some([pos_x, pos_y, pos_z]);

    layout.grids[grid_index].types = voxels;
    layout.grids[grid_index].voxel_count = voxel_index as u8;

    // Defer: Study how this works
      let normal_x = (dists[0b001] + dists[0b011] + dists[0b101] + dists[0b111])
      - (dists[0b000] + dists[0b010] + dists[0b100] + dists[0b110]);
    let normal_y = (dists[0b010] + dists[0b011] + dists[0b110] + dists[0b111])
      - (dists[0b000] + dists[0b001] + dists[0b100] + dists[0b101]);
    let normal_z = (dists[0b100] + dists[0b101] + dists[0b110] + dists[0b111])
      - (dists[0b000] + dists[0b001] + dists[0b010] + dists[0b011]);

    layout.grids[grid_index].pos = avg_pos;
    layout.grids[grid_index].normal = [normal_x, normal_y, normal_z];
  }

  
}

/*
  Always do counter-clockwise towards the normal of the triangle mesh
*/
fn detect_face_x(
  data: &mut MeshData,
  layout: &mut Layout, 
  voxel_reuse: &mut VoxelReuse, 
  x: u32, 
  y: u32, 
  z: u32,
  colors: &Vec<[f32; 3]>,
) {
  /*Detect grids to create surface mesh x-axis:
      0, 0, 0
      0,-1, 0
      0, 0,-1
      0,-1,-1
    Determine where they are facing relative to voxel value/type
  */
  let index = coord_to_index(x, y, z, 0, layout.size);
  let grid_000 = &layout.grids[index];
  if grid_000.pos.is_none() || y == 0 || z == 0 {
    return;
  }
  
  let index = coord_to_index(x, y - 1, z, 0, layout.size);
  let grid_010 = &layout.grids[index];
  if grid_010.pos.is_none() {
    return;
  }

  let index = coord_to_index(x, y, z - 1, 0, layout.size);
  let grid_001 = &layout.grids[index];
  if grid_001.pos.is_none() {
    return;
  }

  let index = coord_to_index(x, y - 1, z - 1, 0, layout.size);
  let grid_011 = &layout.grids[index];
  if grid_011.pos.is_none() {
    return;
  }

  let index = coord_to_index(x, y, z, 0, voxel_reuse.size); // Current
  let face_left = voxel_reuse.voxels[index] > 0;

  let index = coord_to_index(x + 1, y, z, 0, voxel_reuse.size); // Left
  let face_right = voxel_reuse.voxels[index] > 0; // (-1.0, 0.0, 0.0)

  let create = face_left ^ face_right;  // Only one should be true
  if create {
    let voxels = get_vertices_voxels(
      &grid_000, &grid_010, &grid_001, &grid_011
    );
    let color_000 = get_color(&voxels, &grid_000, colors);
    let color_010 = get_color(&voxels, &grid_010, colors);
    let color_011 = get_color(&voxels, &grid_011, colors);
    let color_001 = get_color(&voxels, &grid_001, colors);

    let start = 0;
    if face_left && x != start {
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_010.pos.unwrap());
      data.normals.push(grid_010.normal);
      data.colors.push(color_010);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_011.pos.unwrap());
      data.normals.push(grid_011.normal);
      data.colors.push(color_011);


      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_011.pos.unwrap());
      data.normals.push(grid_011.normal);
      data.colors.push(color_011);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_001.pos.unwrap());
      data.normals.push(grid_001.normal);
      data.colors.push(color_001);
    }

    let end_index = voxel_reuse.size - 1;
    if face_right && x != end_index {
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_011.pos.unwrap());
      data.normals.push(grid_011.normal);
      data.colors.push(color_011);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_010.pos.unwrap());
      data.normals.push(grid_010.normal);
      data.colors.push(color_010);


      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_001.pos.unwrap());
      data.normals.push(grid_001.normal);
      data.colors.push(color_001);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_011.pos.unwrap());
      data.normals.push(grid_011.normal);
      data.colors.push(color_011);
    }
  }
}

fn detect_face_y(
  data: &mut MeshData,
  layout: &mut Layout, 
  voxel_reuse: &mut VoxelReuse, 
  x: u32, 
  y: u32, 
  z: u32,
  colors: &Vec<[f32; 3]>,
) {
  if x == 0 || z == 0 {
    return;
  }

  let index0 = coord_to_index(x, y, z, 0, layout.size);
  let grid_000 = &layout.grids[index0];
  if grid_000.pos.is_none() || y == 0 || z == 0 {
    return;
  }

  let index1 = coord_to_index(x - 1, y, z, 0, layout.size);
  let grid_100 = &layout.grids[index1];
  if grid_100.pos.is_none() {
    return;
  }

  let index2 = coord_to_index(x, y, z - 1, 0, layout.size);
  let grid_001 = &layout.grids[index2];
  if grid_001.pos.is_none() {
    return;
  }

  let index3 = coord_to_index(x - 1, y, z - 1, 0, layout.size);
  let grid_101 = &layout.grids[index3];
  if grid_101.pos.is_none() {
    return;
  }

  let index = coord_to_index(x, y, z, 0, voxel_reuse.size); // Grid voxel below: Note: Should be current?
  let face_up = voxel_reuse.voxels[index] > 0;  // (0.0, 1.0, 0.0)

  let index = coord_to_index(x, y + 1, z, 0, voxel_reuse.size); // Grid voxel on top
  let face_down = voxel_reuse.voxels[index] > 0;

  let create = face_up ^ face_down;
  if create {
    let voxels = get_vertices_voxels(
      &grid_000, &grid_101, &grid_100, &grid_001
    );
    
    let color_000 = get_color(&voxels, &grid_000, colors);
    let color_101 = get_color(&voxels, &grid_101, colors);
    let color_100 = get_color(&voxels, &grid_100, colors);
    let color_001 = get_color(&voxels, &grid_001, colors);

    let start = 0;
    if face_up && y != start {

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);
      
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_101.pos.unwrap());
      data.normals.push(grid_101.normal);
      data.colors.push(color_101);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_100.pos.unwrap());
      data.normals.push(grid_100.normal);
      data.colors.push(color_100);


      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_001.pos.unwrap());
      data.normals.push(grid_001.normal);
      data.colors.push(color_001);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_101.pos.unwrap());
      data.normals.push(grid_101.normal);
      data.colors.push(color_101);

    }

    let end_index = voxel_reuse.size - 1;
    if face_down && y != end_index {
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_100.pos.unwrap());
      data.normals.push(grid_100.normal);
      data.colors.push(color_100);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_101.pos.unwrap());
      data.normals.push(grid_101.normal);
      data.colors.push(color_101);


      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_101.pos.unwrap());
      data.normals.push(grid_101.normal);
      data.colors.push(color_101);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_001.pos.unwrap());
      data.normals.push(grid_001.normal);
      data.colors.push(color_001);
    }
  }
}

fn detect_face_z(
  data: &mut MeshData,
  layout: &mut Layout, 
  voxel_reuse: &mut VoxelReuse, 
  x: u32, 
  y: u32, 
  z: u32,
  colors: &Vec<[f32; 3]>,
) {
  if x == 0 || y == 0 {
    return;
  }

  let index = coord_to_index(x, y, z, 0, layout.size);
  let grid_000 = &layout.grids[index];
  if grid_000.pos.is_none() || y == 0 || z == 0 {
    return;
  }

  let index = coord_to_index(x - 1, y, z, 0, layout.size);
  let grid_100 = &layout.grids[index];
  if grid_100.pos.is_none() || y == 0 || z == 0 {
    return;
  }

  let index = coord_to_index(x, y - 1, z, 0, layout.size);
  let grid_010 = &layout.grids[index];
  if grid_010.pos.is_none() || y == 0 || z == 0 {
    return;
  }

  let index = coord_to_index(x - 1, y - 1, z, 0, layout.size);
  let grid_110 = &layout.grids[index];
  if grid_110.pos.is_none() || y == 0 || z == 0 {
    return;
  }

  let index = coord_to_index(x, y, z, 0, voxel_reuse.size); // Current
  let face_front = voxel_reuse.voxels[index] > 0; // (0.0, 0.0, 1.0)

  let index = coord_to_index(x, y, z + 1, 0, voxel_reuse.size); // Forward
  let face_back = voxel_reuse.voxels[index] > 0;

  let create = face_front ^ face_back;
  if create {
    let voxels = get_vertices_voxels(
      &grid_000, &grid_100, &grid_010, &grid_110
    );
    
    let color_000 = get_color(&voxels, &grid_000, colors);
    let color_100 = get_color(&voxels, &grid_100, colors);
    let color_010 = get_color(&voxels, &grid_010, colors);
    let color_110 = get_color(&voxels, &grid_110, colors);

    let start = 0;
    if face_front && z != start {
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);
      
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_110.pos.unwrap());
      data.normals.push(grid_110.normal);
      data.colors.push(color_110);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_010.pos.unwrap());
      data.normals.push(grid_010.normal);
      data.colors.push(color_010);


      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_100.pos.unwrap());
      data.normals.push(grid_100.normal);
      data.colors.push(color_100);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_110.pos.unwrap());
      data.normals.push(grid_110.normal);
      data.colors.push(color_110);
    }

    let end_index = voxel_reuse.size - 1;
    if face_back && z != end_index {
      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_010.pos.unwrap());
      data.normals.push(grid_010.normal);
      data.colors.push(color_010);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_110.pos.unwrap());
      data.normals.push(grid_110.normal);
      data.colors.push(color_110);


      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_000.pos.unwrap());
      data.normals.push(grid_000.normal);
      data.colors.push(color_000);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_110.pos.unwrap());
      data.normals.push(grid_110.normal);
      data.colors.push(color_110);

      data.indices.push(data.positions.len() as u32);
      data.positions.push(grid_100.pos.unwrap());
      data.normals.push(grid_100.normal);
      data.colors.push(color_100);
    }
  }
}

fn get_color(
  _voxels: &[u32; 4], 
  grid: &Grid, 
  mapped_colors: &Vec<[f32; 3]>
) -> [f32; 3] {
  let mut color = [0.0, 0.0, 0.0];

  for voxel in grid.types.iter() {
    if *voxel > 0 {
      let color_index = *voxel as usize - 1;
      color[0] += mapped_colors[color_index][0];
      color[1] += mapped_colors[color_index][1];
      color[2] += mapped_colors[color_index][2];
    }
  }

  color[0] /= grid.voxel_count as f32;
  color[1] /= grid.voxel_count as f32;
  color[2] /= grid.voxel_count as f32;
  
  color
}

// FIXME: Remove later
fn get_vertices_voxels(
  grid_0: &Grid,
  _grid_1: &Grid,
  _grid_2: &Grid,
  _grid_3: &Grid,
) -> [u32; 4] {
  let all_voxels = [0; 4];
  let mut _index = 0;
  for _i in 0..grid_0.types.len() {
    // let type0 = grid_0.types[i];
    // let type1 = grid_1.types[i];
    // let type2 = grid_2.types[i];
    // let type3 = grid_3.types[i];



    // if !all_voxels.contains(&type0) {
    //   // println!("index {} {:?}", index, all_voxels);
    //   all_voxels[index] = type0;
    //   index += 1;
    //   if index == 4 {
    //     break;
    //   }
    // }
    // if !all_voxels.contains(&type1) {
    //   // println!("index {} {:?}", index, all_voxels);
    //   all_voxels[index] = type1;
    //   index += 1;
    //   if index == 4 {
    //     break;
    //   }
    // }
    // if !all_voxels.contains(&type2) {
    //   // println!("index {} {:?}", index, all_voxels);
    //   all_voxels[index] = type2;
    //   index += 1;
    //   if index == 4 {
    //     break;
    //   }
    // }
    // if !all_voxels.contains(&type3) {
    //   // println!("index {} {:?}", index, all_voxels);
    //   all_voxels[index] = type3;
    //   index += 1;
    //   if index == 4 {
    //     break;
    //   }
    // }
  }
  // modify_to_texture_indices(&mut all_voxels);
  all_voxels
}

pub fn estimate_surface_edge_intersection(
  offset1: usize,
  offset2: usize,
  value1: f32,
  value2: f32,
) -> Option<[f32; 3]> {
  if (value1 < 0.0) == (value2 < 0.0) {
    return None;
  }

  let interp1 = value1 / (value1 - value2);
  let interp2 = 1.0 - interp1;
  let position = [
    (offset1 & 1) as f32 * interp2 + (offset2 & 1) as f32 * interp1,
    ((offset1 >> 1) & 1) as f32 * interp2 + ((offset2 >> 1) & 1) as f32 * interp1,
    ((offset1 >> 2) & 1) as f32 * interp2 + ((offset2 >> 2) & 1) as f32 * interp1,
  ];

  Some(position)
}

pub fn has_position_indices_for_x(back_index: u32, back_bottom_index: u32, bottom_index: u32) -> bool {
  back_index != std::u32::MAX && 
  back_bottom_index != std::u32::MAX && 
  bottom_index != std::u32::MAX
}

pub fn has_position_indices_for_y(right_back_index: u32, right_index: u32, back_index: u32) -> bool {
  right_back_index != std::u32::MAX && right_index != std::u32::MAX && back_index != std::u32::MAX
}


pub fn has_position_indices_for_z(right_index: u32, right_bottom_index: u32, bottom_index: u32) -> bool {
  right_index != std::u32::MAX
    && right_bottom_index != std::u32::MAX
    && bottom_index != std::u32::MAX
}

/*
  Code for client side
*/
#[derive(Clone)]
pub struct MeshColliderData {
  pub positions: Vec<Point<f32>>,
  pub indices: Vec<[u32; 3]>,
}




#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  /* Update later */
  #[test]
  fn test_creation_mesh_data_positions() -> Result<(), String> {
    let mut octree = VoxelOctree::new_from_3d_array(
      0, 4,
      &vec![
        [2, 2, 2, 1],
        // [3, 2, 2, 2],
      ].to_vec(),
      ParentValueType::DefaultValue,
    );

    let colors = vec![
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
    ];

    let data = octree.compute_mesh(
      VoxelMode::SurfaceNets,
      &mut VoxelReuse::new(4, 3),
      &colors,
      1.0
    );

    /*Set the expected and actual result here
      Order matters for now to make it easier to check(probably should be)

      The positions through x, y and z are switching between 1.833 and 2.167
        Reverse order of setting
          z -> y -> x
          Ex:
            Forward vector: (0.0, -1.0, 0.0)
            1.833, 1.833, 1.833
            1.833, 1.833, 2.167
            1.833, 2.167, 2.167
      Defining the positions?
        Should be done with defining the indices
     */
    
    let mut index = 0;
    for (index, value) in data.positions.iter().enumerate() {
      if index % 3 == 0 {
        println!();
      }

      let pos = format!("{:.1}, {:.1}, {:.1}", value[0], value[1], value[2]);
      println!("{} {:?} {:?}", pos, data.types_1[index], data.weights[index]);

      
    }

    // for p in data.indices.iter().enumerate() {
    //   println!("{:?}", p);
    // }

    // for p in data.normals.iter().enumerate() {
    //   println!("{:?}", p);
    // }

    // for p in data.weights.iter().enumerate() {
    //   println!("{:?}", p);
    // }

    // for p in data.types_1.iter().enumerate() {
    //   println!("{:?}", p);
    // }

    // for p in data.types_1.iter().enumerate() {
    //   println!("{:?}", p);
    // }


    Ok(())
  }


  #[test]
  fn test_one_voxel_mesh_data() -> Result<(), String> {
    let positions = load_vec3f32("assets/1_voxel_positions.data");
    let weights = load_vec4f32("assets/1_voxel_weights.data");
    let types = load_vec4u32("assets/1_voxel_types.data");

    let mut octree = VoxelOctree::new_from_3d_array(
      0, 4,
      &vec![
        [2, 2, 2, 1],
        // [3, 2, 2, 2],
      ].to_vec(),
      ParentValueType::DefaultValue,
    );

    let colors = vec![
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
    ];

    let data = octree.compute_mesh(
      VoxelMode::SurfaceNets,
      &mut VoxelReuse::new(4, 3),
      &colors,
      1.0
    );
    for (index, value) in positions.iter().enumerate() {
      assert_eq!(value, &data.positions[index], "at index {}", index);
    }

    for (index, value) in weights.iter().enumerate() {
      assert_eq!(value, &data.weights[index]);
    }

    for (index, value) in types.iter().enumerate() {
      assert_eq!(value, &data.types_1[index]);
    }
    Ok(())
  }

  #[test]
  fn test_2_voxel_mesh_data() -> Result<(), String> {
    let positions = load_vec3f32("assets/2_voxel_positions.json");
    let weights = load_vec4f32("assets/2_voxel_weights.json");
    let types = load_vec4u32("assets/2_voxel_types.json");

    let mut octree = VoxelOctree::new_from_3d_array(
      0, 4,
      &vec![
        [2, 2, 2, 1],
        [3, 2, 2, 2],
      ].to_vec(),
      ParentValueType::DefaultValue,
    );

    let colors = vec![
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
      [0.0, 0.0, 0.0],
    ];

    let data = octree.compute_mesh(
      VoxelMode::SurfaceNets,
      &mut VoxelReuse::new(4, 3),
      &colors,
      1.0
    );
    for (index, value) in positions.iter().enumerate() {
      assert_eq!(&data.positions[index], value, "at index {}", index);
    }

    println!("data.types_1.len(): {:?}", data.types_1.len());
    for (index, value) in types.iter().enumerate() {
      // println!("index {:?}", value);

      assert_eq!(&data.types_1[index], value, "Wrong texture indices at index {}", index);
    }

    for (index, value) in data.weights.iter().enumerate() {
      println!("w {:?}", data.weights[index]);
    }

    for (index, value) in weights.iter().enumerate() {
      println!("weight {:?}", data.weights[index]);

      assert_eq!(&data.weights[index], value, "Wrong weights at index {}", index);
    }

    


    
    Ok(())
  }



  fn load_vecu32(path: &str) -> Vec<u32> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let vec: Vec<u32> = match ron::from_str(&data) {
      Ok(m) => m,
      Err(e) => {
        println!("e {:?}", e);
        return Vec::new() 
      }
    };
    vec
  }
  
  fn load_vec3f32(path: &str) -> Vec<[f32; 3]> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let vec: Vec<[f32; 3]> = match ron::from_str(&data) {
      Ok(m) => m,
      Err(e) => {
        println!("e {:?}", e);
        return Vec::new() 
      }
    };
    vec
  }
  
  fn load_vec3u32(path: &str) -> Vec<[u32; 3]> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let vec: Vec<[u32; 3]> = match ron::from_str(&data) {
      Ok(m) => m,
      Err(e) => {
        println!("e {:?}", e);
        return Vec::new() 
      }
    };
    vec
  }
  
  fn load_vec4u32(path: &str) -> Vec<[u32; 4]> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let vec: Vec<[u32; 4]> = match ron::from_str(&data) {
      Ok(m) => m,
      Err(e) => {
        println!("e {:?}", e);
        return Vec::new() 
      }
    };
    vec
  }
  
  fn load_vec4f32(path: &str) -> Vec<[f32; 4]> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let vec: Vec<[f32; 4]> = match ron::from_str(&data) {
      Ok(m) => m,
      Err(e) => {
        println!("e {:?}", e);
        return Vec::new() 
      }
    };
    vec
  }
  
}