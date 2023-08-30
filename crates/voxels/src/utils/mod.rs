pub mod grid_hashmap;
use crate::chunk::voxel_pos_to_key;
use crate::data::voxel_octree::VoxelOctree;

pub struct Utils;

impl Utils {
  pub fn create_z_faces(right_index: u32, right_bottom_index: u32, bottom_index: u32) -> bool {
    right_index != std::u32::MAX
      && right_bottom_index != std::u32::MAX
      && bottom_index != std::u32::MAX
  }
  
  pub fn create_x_faces(back_index: u32, back_bottom_index: u32, bottom_index: u32) -> bool {
    back_index != std::u32::MAX && 
    back_bottom_index != std::u32::MAX && 
    bottom_index != std::u32::MAX
  }
  
  pub fn create_y_faces(right_back_index: u32, right_index: u32, back_index: u32) -> bool {
    right_back_index != std::u32::MAX && right_index != std::u32::MAX && back_index != std::u32::MAX
  }

  pub fn has_pos(cur_pos: &[f32; 3], pos: &[f32; 3], dir: &[i32; 3]) -> bool {
    let x = pos[0] as i32;
    let y = pos[1] as i32;
    let z = pos[2] as i32;
    let cur_x = cur_pos[0] as i32 + dir[0];
    let cur_y = cur_pos[1] as i32 + dir[1];
    let cur_z = cur_pos[2] as i32 + dir[2];
    cur_x == x && cur_y == y && cur_z == z
  }
  
  /* DEPRECATE LATER */
  pub fn has_voxel(octree: &VoxelOctree, pos: &[u32; 3], dir: &[u32; 3]) -> bool {
    octree.get_voxel(pos[0] + dir[0], pos[1] + dir[1], pos[2] + dir[2]) > 0
  }
  
  pub fn has_voxel2(voxels: &Vec<u8>, start: u32, end: u32, pos: &[u32; 3], dir: &[u32; 3]) -> bool {
    // octree.get_voxel(pos[0] + dir[0], pos[1] + dir[1], pos[2] + dir[2]) > 0
    let x = pos[0] + dir[0];
    let y = pos[1] + dir[1];
    let z = pos[2] + dir[2];
    let index = coord_to_index(x, y, z, start, end);
    if index >= voxels.len() {
      return false;
    }
    voxels[index] > 0
  }
}



pub fn posf32_to_world_key(pos: &[f32; 3], seamless_size: u32) -> [i64; 3] {
  let posi64 = [pos[0] as i64, pos[1] as i64, pos[2] as i64];
  // world_pos_to_key(&posi64, seamless_size)
  voxel_pos_to_key(&posi64, seamless_size)
}



#[derive(Debug)]
pub struct ChunkCoordinate {
  pub key: [i64; 3],
  pub local: [u32; 3]
}

pub fn coord_to_index(x: u32, y: u32, z: u32, start: u32, end: u32) -> usize {
  let diff = end - start;

  let start_x = x - start;
  let start_y = y - start;
  let start_z = z - start;

  let val_x = diff.pow(2) * start_x;
  let val_y = diff * start_y;
  let val_z = start_z;

  let res = val_x + val_y + val_z;
  res as usize
}

pub fn get_len_by_size(size: u32, loop_count: u32) -> usize {
  size.pow(loop_count) as usize
}



pub fn get_length(depth: u8) -> usize {
  let mut len = 0;
  for d in 1..depth {
    len += 8_i32.pow(d as u32);
  }
  len as usize
}




pub fn get_chunk_coords(
  pos: &[i64; 3], 
  chunk_size: u32, 
  seamless_size: u32
) -> Vec<ChunkCoordinate> {
  let mut coords = Vec::new();

  let keys = &potential_keys(pos, seamless_size);
  for key in keys.iter() {
    // println!("key {:?}", key);
    if has_local_coord(pos, key, chunk_size, seamless_size as i64) {
      let local = get_local_coord(pos, key, chunk_size);

      let coord = ChunkCoordinate {
        key: key.clone(),
        local: local
      };
      coords.push(coord);
      // println!("get_chunk_coords key {:?} coord {:?}", key, local);
    }

  }

  coords
}

pub fn get_chunk_coords2(
  pos: &[i64; 3], 
  chunk_size: u32, 
  seamless_size: u32
) -> Vec<ChunkCoordinate> {
  let mut coords = Vec::new();

  let keys = &potential_keys(pos, seamless_size);
  for key in keys.iter() {
    // println!("key {:?}", key);
    if has_local_coord(pos, key, chunk_size, seamless_size as i64) {
      let local = get_local_coord(pos, key, chunk_size);

      let coord = ChunkCoordinate {
        key: key.clone(),
        local: local
      };
      coords.push(coord);
      // println!("get_chunk_coords key {:?} coord {:?}", key, local);
    }

  }

  coords
}





pub fn potential_keys(pos: &[i64; 3], seamless_size: u32) -> Vec<[i64; 3]> {
  let mut keys = Vec::new();

  let start_key = voxel_pos_to_key(pos, seamless_size);
  for x in -1..1 {
    for y in -1..1 {
      for z in -1..1 {
        let key_x = start_key[0] + x;
        let key_y = start_key[1] + y;
        let key_z = start_key[2] + z;
        let key = [key_x, key_y, key_z];
        keys.push(key);
      }
    }
  }
  keys
}

pub fn has_local_coord(
  pos: &[i64; 3], 
  potential_key: &[i64; 3], 
  chunk_size: u32,
  seamless_size: i64
) -> bool {
  let min_x = potential_key[0] * seamless_size;
  let max_x = min_x + chunk_size as i64;
  let min_y = potential_key[1] * seamless_size;
  let max_y = min_y + chunk_size as i64;
  let min_z = potential_key[2] * seamless_size;
  let max_z = min_z + chunk_size as i64;

  // println!("min pos {:?} {:?} {} {}", pos, potential_key, min_x, max_x);

  // FIXME: Refactor later
  if (pos[0] >= min_x && pos[0] < max_x)
    && (pos[1] >= min_y && pos[1] < max_y)
    && (pos[2] >= min_z && pos[2] < max_z)
  {
    return true;
  }
  false
}

pub fn get_local_coord(
  pos: &[i64; 3],
  potential_key: &[i64; 3], 
  chunk_size: u32
) -> [u32; 3] {
  let partitioned_size = chunk_size as i64 - 2;

  let min_x = potential_key[0] * partitioned_size;
  let min_y = potential_key[1] * partitioned_size;
  let min_z = potential_key[2] * partitioned_size;

  /* 
    TODO
      Address the negative position later
  */
  let diff_x = pos[0] - min_x;
  let diff_y = pos[1] - min_y;
  let diff_z = pos[2] - min_z;

  let local_x = diff_x;
  let local_y = diff_y;
  let local_z = diff_z;

  // println!("local {} {} {}", local_x, local_y, local_z);
  [local_x as u32, local_y as u32, local_z as u32]
}


pub fn world_pos_to_octree_coord(pos: &[i64; 3], seamless_size: u32) -> OctreeCoord {
  let key = world_pos_to_octree_key(pos, seamless_size);

  let mut mx = pos[0] % seamless_size as i64;
  let mut my = pos[1] % seamless_size as i64;
  let mut mz = pos[2] % seamless_size as i64;
  if pos[0] < 0 {
    if mx != 0 {
      mx += seamless_size as i64;
    }
    
  }
  if pos[1] < 0 {
    if my != 0 {
      my += seamless_size as i64;
    }
  }
  if pos[2] < 0 {
    if mz != 0 {
      mz += seamless_size as i64;
    }
    
  }
  let local = [mx as u32, my as u32, mz as u32];
  OctreeCoord { key: key, local: local }
}

pub fn world_pos_to_octree_key(pos: &[i64; 3], seamless_size: u32) -> [i64; 3] {
  let mut x = pos[0];
  let mut y = pos[1];
  let mut z = pos[2];
  let seamless_size_i64 = seamless_size as i64;

  /*
    14  to  27 =  1
    0   to  13 =  0
    -1  to -14 = -1
    -15 to -27 = -2
  */

  /*
    if negative:
      num = num + 1
      key = (num / seamless_size) + 1
  */
  // Between -0.epsilon to -seamless_size..., it should be -1
  let mut kx = x / seamless_size_i64;
  let mut ky = y / seamless_size_i64;
  let mut kz = z / seamless_size_i64;
  if x < 0 {
    x += 1;
    kx = (x / seamless_size_i64) - 1;
  }
  if y < 0 {
    y += 1;
    ky = (y / seamless_size_i64) - 1;
  }
  if z < 0 {
    z += 1;
    kz = (z / seamless_size_i64) - 1;
  }
  [kx, ky, kz]
}


pub fn key_to_world_coord_f32(key: &[i64; 3], seamless_size: u32) -> [f32; 3] {
  [
    (key[0] * seamless_size as i64) as f32,
    (key[1] * seamless_size as i64) as f32,
    (key[2] * seamless_size as i64) as f32,
  ]
}

pub struct OctreeCoord {
  pub key: [i64; 3],
  pub local: [u32; 3]
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_coord_to_index1() -> Result<(), String> {
    let default_value = 0;
    let mut octree = VoxelOctree::new(default_value, 4);

    let start = 1;
    let end = octree.get_size() - 2;

    // println!("start: end: {} {}",start, end);
    let mut index = 0;
    for x in start..end {
      for y in start..end {
        for z in start..end {
          // println!("right {} {} {} {}", x, y, z, index);
          assert_eq!(
            index,
            coord_to_index(x, y, z, start, end),
            "pos {} {} {}",
            x,
            y,
            z
          );
          index += 1;
        }
      }
    }

    Ok(())
  }

  #[test]
  fn test_coord_to_index2() -> Result<(), String> {
    let default_value = 0;
    let mut octree = VoxelOctree::new(default_value, 4);

    let start = 0;
    let end = octree.get_size() - 1;

    // println!("start: end: {} {}",start, end);
    let mut index = 0;
    for x in start..end {
      for y in start..end {
        for z in start..end {
          // println!("right {} {} {} {}", x, y, z, index);
          assert_eq!(
            index,
            coord_to_index(x, y, z, start, end),
            "pos {} {} {}",
            x,
            y,
            z
          );
          index += 1;
        }
      }
    }

    Ok(())
  }

  #[test]
  fn test_coord_to_index3() -> Result<(), String> {
    let size = 15;
    let start = 0;
    let end = size;
    let mut index = 0;
    for x in start..end {
      for y in start..end {
        for z in start..end {
          assert_eq!(
            index,
            coord_to_index(x, y, z, start, end),
            "pos {} {} {}",
            x,
            y,
            z
          );
          index += 1;
        }
      }
    }

    let axes = 3;
    let len = get_len_by_size(size, axes);
    assert_eq!(index, len);

    Ok(())
  }

  #[test]
  fn test_get_len_by_size() -> Result<(), String> {
    let size = 14;
    let loop_count = 3;
    let len = get_len_by_size(size, loop_count);
    let mut expected_len = 0;
    for x in 0..size {
      for y in 0..size {
        for z in 0..size {
          expected_len += 1;
        }
      }
    }

    assert_eq!(len, expected_len);
    Ok(())
  }




  #[test]
  fn test_local_coord() -> Result<(), String> {
    let size = 16;
    let seamless_size = size - 2;
    
    let positions = [
      [ 14, 27, 28],
      [ 0, 13, 0],
      [-1,-14,-15],
      [-28,-29,-42],
    ];
    let expected = [
      OctreeCoord { key: [1, 1, 2], local: [0, 13, 0]},
      OctreeCoord { key: [0, 0, 0], local: [0, 13, 0]},
      OctreeCoord { key: [-1, -1, -2], local: [13, 0, 13]},
      OctreeCoord { key: [-2, -3, -3], local: [0, 13, 0]},
    ];
    for (i, pos) in positions.iter().enumerate() {
      let result = world_pos_to_octree_coord(pos, seamless_size);
      assert_eq!(result.key, expected[i].key, "Wrong key at index {}", i);
      assert_eq!(result.local, expected[i].local, "Wrong local at index {}", i);
    }

    Ok(())
  }


}
