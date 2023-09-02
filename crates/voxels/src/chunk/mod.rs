use hashbrown::HashMap;
use noise::NoiseFn;
use noise::OpenSimplex;
use num_traits::Pow;
use crate::data::voxel_octree::VoxelOctree;
use self::chunk_manager::*;

pub mod chunk_manager;


pub fn is_adjacent(key1: &[i64; 3], key2: &[i64; 3]) -> bool {
  let dist = 1;
  for i in 0..key1.len() {
    let diff = key1[i] - key2[i];
    if diff.abs() > dist {
      return false;
    }
  }
  true
}

pub fn adjacent_keys_map(
  key: &[u32; 3],
  range: i64,
  include_current: bool,
) -> HashMap<[u32; 3], bool> {
  let mut keys_around = HashMap::new();
  let start = -range as i64;
  // let start = 0; // FIXME: For testing
  let end = range + 1;

  for iter_x in start..end {
    for iter_y in start..end {
      for iter_z in start..end {
        let chunk_x = key[0] as i64 + iter_x;
        let chunk_y = key[1] as i64 + iter_y;
        let chunk_z = key[2] as i64 + iter_z;

        let chunk_key = [chunk_x as u32, chunk_y as u32, chunk_z as u32];
        if include_current {
          keys_around.insert(chunk_key, true);
        }

        if !include_current && !same_coord(key, &chunk_key) {
          keys_around.insert(chunk_key, true);
        }
      }
    }
  }
  keys_around
}

pub fn adjacent_keys_map2(
  key: &[i64; 3],
  range: i64,
  include_current: bool,
) -> HashMap<[i64; 3], bool> {
  let mut keys_around = HashMap::new();
  let start = -range as i64;
  // let start = 0; // FIXME: For testing
  let end = range + 1;

  for iter_x in start..end {
    for iter_y in start..end {
      for iter_z in start..end {
        let chunk_x = key[0] + iter_x;
        let chunk_y = key[1] + iter_y;
        let chunk_z = key[2] + iter_z;

        let chunk_key = [chunk_x, chunk_y, chunk_z];
        if include_current {
          keys_around.insert(chunk_key, true);
        }

        if !include_current && !same_coord_i64(key, &chunk_key) {
          keys_around.insert(chunk_key, true);
        }
      }
    }
  }
  keys_around
}

pub fn adjacent_keys(
  key: &[i64; 3], 
  range: i64,
  include_current: bool,
) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for iter_x in start..end {
    for iter_y in start..end {
      for iter_z in start..end {
        let chunk_x = key[0] + iter_x;
        let chunk_y = key[1] + iter_y;
        let chunk_z = key[2] + iter_z;

        let chunk_key = [chunk_x, chunk_y, chunk_z];
        if include_current {
          keys_around.push(chunk_key);
        }

        if !include_current && !same_coord_i64(key, &chunk_key) {
          keys_around.push(chunk_key);
        }
      }
    }
  }
  keys_around
}

pub fn adjacent_keys_lod(
  key: &[i64; 3],
  lod: i64,
  range: i64,
  include_current: bool,
) -> Vec<[i64; 4]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for iter_x in start..end {
    for iter_y in start..end {
      for iter_z in start..end {
        let chunk_x = key[0] + iter_x;
        let chunk_y = key[1] + iter_y;
        let chunk_z = key[2] + iter_z;

        let chunk_key = [chunk_x, chunk_y, chunk_z, lod];
        if include_current {
          keys_around.push(chunk_key);
        }

        let tmp_key = [key[0], key[1], key[2], lod];
        if !include_current && !same_coord2(&tmp_key, &chunk_key) {
          keys_around.push(chunk_key);
        }
      }
    }
  }
  keys_around
}

pub fn adj_delta_keys(prev_key: &[i64; 3], cur_key: &[i64; 3], range: i64) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let c_x = cur_key[0] + x;
        let c_y = cur_key[1] + y;
        let c_z = cur_key[2] + z;

        let diff_x = c_x - prev_key[0];
        let diff_y = c_y - prev_key[1];
        let diff_z = c_z - prev_key[2];

        if diff_x.abs() > range || diff_y.abs() > range || diff_z.abs() > range {
          let key = [c_x, c_y, c_z];
          keys_around.push(key);
        }
      }
    }
  }

  keys_around
}

pub fn in_range_by_chunk(pos1: &[i64; 3], pos2: &[i64; 3], range: i64) -> bool {
  let x = pos1[0] - pos2[0];
  let y = pos1[1] - pos2[1];
  let z = pos1[2] - pos2[2];
  // println!("x {} pos {:?} {:?}", x, pos1, pos2);
  x.abs() <= range && y.abs() <= range && z.abs() <= range
}

pub fn adjacent_keys_i64(key: &[i64; 3], range: i64, include_current: bool) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -range;
  // let start = 0; // FIXME: For testing
  let end = range + 1;

  for iter_x in start..end {
    for iter_y in start..end {
      for iter_z in start..end {
        let chunk_x = key[0] + iter_x;
        let chunk_y = key[1] + iter_y;
        let chunk_z = key[2] + iter_z;

        let chunk_key = [chunk_x, chunk_y, chunk_z];
        if include_current {
          keys_around.push(chunk_key);
        }

        if !include_current && !same_coord_i64(key, &chunk_key) {
          keys_around.push(chunk_key);
        }
      }
    }
  }
  keys_around
}

pub fn adjacent_keys_minmax(key: &[i64; 3], min: i64, max: i64) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -max;
  let end = max + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let chunk_x = key[0] + x;
        let chunk_y = key[1] + y;
        let chunk_z = key[2] + z;
        let c_key = [chunk_x, chunk_y, chunk_z];

        if !in_range2(key, &c_key, min, max) {
          continue;
        }

        keys_around.push([c_key[0], c_key[1], c_key[2]]);
      }
    }
  }
  keys_around
}

pub fn adjacent_keys_by_dist(key: &[i64; 3], range: i64) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let chunk_x = key[0] + x;
        let chunk_y = key[1] + y;
        let chunk_z = key[2] + z;

        if !in_range(key, &[chunk_x, chunk_y, chunk_z], range) {
          continue;
        }

        let chunk_key = [chunk_x, chunk_y, chunk_z];
        keys_around.push(chunk_key);
      }
    }
  }
  keys_around
}

pub fn adjacent_keys_min(key: &[i64; 3], range: i64, min: i64, lod: i64) -> Vec<[i64; 4]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let mut dist = i64::abs(x);
        if i64::abs(y) > dist {
          dist = i64::abs(y)
        }
        if i64::abs(z) > dist {
          dist = i64::abs(z)
        }
        if dist < min {
          continue;
        }

        let chunk_x = key[0] + x;
        let chunk_y = key[1] + y;
        let chunk_z = key[2] + z;

        let chunk_key = [chunk_x, chunk_y, chunk_z, lod];
        keys_around.push(chunk_key);
      }
    }
  }
  keys_around
}

pub fn in_range(pos1: &[i64; 3], pos2: &[i64; 3], range: i64) -> bool {
  let mut dist_sqr = 0;
  let r = range;
  for (index, val) in pos1.iter().enumerate() {
    let diff = val - pos2[index];
    dist_sqr += diff * diff;
  }
  dist_sqr <= r.pow(2)
}

pub fn in_rangef(pos1: &[i64; 3], pos2: &[i64; 3], range: f32) -> bool {
  let mut dist_sqr = 0.0;
  let r = range as f32;
  for (index, val) in pos1.iter().enumerate() {
    let diff = (val - pos2[index]) as f32;
    dist_sqr += diff * diff;
  }
  dist_sqr <= r.pow(2)
}

pub fn in_range2(pos1: &[i64; 3], pos2: &[i64; 3], min: i64, max: i64) -> bool {
  let mut dist_sqr = 0;
  for (index, val) in pos1.iter().enumerate() {
    let diff = val - pos2[index];
    dist_sqr += diff * diff;
  }
  dist_sqr > min.pow(2) && dist_sqr <= max.pow(2)
}

pub fn in_range2f(pos1: &[i64; 3], pos2: &[i64; 3], min: f32, max: f32) -> bool {
  let mut dist_sqr = 0;
  for (index, val) in pos1.iter().enumerate() {
    let diff = val - pos2[index];
    dist_sqr += diff * diff;
  }
  let d = dist_sqr as f32;
  d >= min.pow(2) && d <= max.pow(2)
}

pub fn delta_keys(prev_key: &[i64; 3], cur_key: &[i64; 3], range: i64) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let c_x = cur_key[0] + x;
        let c_y = cur_key[1] + y;
        let c_z = cur_key[2] + z;

        let key = [c_x, c_y, c_z];
        if in_range(cur_key, &key, range) && !in_range(prev_key, &key, range) {
          keys_around.push(key);
        }
      }
    }
  }

  keys_around
}

pub fn delta_keys_minmax(
  prev_key: &[i64; 3],
  cur_key: &[i64; 3],
  min: i64,
  max: i64,
) -> Vec<[i64; 3]> {
  let mut keys_around = Vec::new();
  let start = -max;
  let end = max + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let c_x = cur_key[0] + x;
        let c_y = cur_key[1] + y;
        let c_z = cur_key[2] + z;

        let chunk = [c_x, c_y, c_z];
        // Getting outer delta
        if in_range(cur_key, &chunk, max) && !in_range(prev_key, &chunk, max) {
          let chunk_key = [c_x, c_y, c_z];
          keys_around.push(chunk_key);
          continue;
        }

        // Getting inner delta
        if in_range(cur_key, &chunk, max)
          && !in_range(cur_key, &chunk, min)
          && in_range(prev_key, &chunk, min)
        {
          let chunk_key = [c_x, c_y, c_z];
          keys_around.push(chunk_key);
        }
      }
    }
  }

  keys_around
}

/** Deprecated */
pub fn unexplored_keys(
  cur_key: &[i64; 3],
  prev_key: &[i64; 3],
  range: i64,
  lod: i64,
) -> Vec<[i64; 4]> {
  let mut keys_around = Vec::new();
  let start = -range;
  let end = range + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let c_x = cur_key[0] + x;
        let c_y = cur_key[1] + y;
        let c_z = cur_key[2] + z;

        let chunk = [c_x, c_y, c_z];
        if in_range(cur_key, &chunk, range) && !in_range(prev_key, &chunk, range) {
          let chunk_key = [c_x, c_y, c_z, lod];
          keys_around.push(chunk_key);
        }
      }
    }
  }

  keys_around
}

/*
  Deprecate in favor of delta_keys_minmax()
*/
pub fn unexplored_keys2(
  cur_key: &[i64; 3],
  prev_key: &[i64; 3],
  min: i64,
  max: i64,
  lod: i64,
) -> Vec<[i64; 4]> {
  let mut keys_around = Vec::new();
  let start = -max;
  let end = max + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let c_x = cur_key[0] + x;
        let c_y = cur_key[1] + y;
        let c_z = cur_key[2] + z;

        let chunk = [c_x, c_y, c_z];
        if in_range(cur_key, &chunk, max) && !in_range(prev_key, &chunk, min) {
          let chunk_key = [c_x, c_y, c_z, lod];
          keys_around.push(chunk_key);
        }

        // if in_rangef(cur_key, &chunk, max as f32) &&
        // !in_rangef(prev_key, &chunk, min as f32) {
        //   let chunk_key = [c_x, c_y, c_z, lod];
        //   keys_around.push(chunk_key);
        // }

        // if in_range2(cur_key, &chunk, min, max) &&
        // !in_range2(prev_key, &chunk, min, max) {
        //   let chunk_key = [c_x, c_y, c_z, lod];
        //   keys_around.push(chunk_key);
        // }
      }
    }
  }

  keys_around
}

pub fn unexplored_keys2f(
  cur_key: &[i64; 3],
  prev_key: &[i64; 3],
  min: f32,
  max: f32,
  lod: i64,
) -> Vec<[i64; 4]> {
  let mut keys_around = Vec::new();
  let start = -max as i64;
  let end = max as i64 + 1;

  for x in start..end {
    for y in start..end {
      for z in start..end {
        let c_x = cur_key[0] + x;
        let c_y = cur_key[1] + y;
        let c_z = cur_key[2] + z;

        let chunk = [c_x, c_y, c_z];
        if in_range2f(cur_key, &chunk, min, max) && !in_range2f(prev_key, &chunk, min, max) {
          let chunk_key = [c_x, c_y, c_z, lod];
          keys_around.push(chunk_key);
        }
      }
    }
  }

  keys_around
}

// FIXME: Have a checker where the world pos should not be more or less that it should be
pub fn region_key_to_world_key(key: &[u32; 3], seamless_size: u32) -> [i64; 3] {
  let pos = region_key_to_world_pos(key, seamless_size);
  world_pos_to_key(&pos, seamless_size)
}

pub fn region_key_to_world_pos(r_key: &[u32; 3], seamless_size: u32) -> [i64; 3] {
  let region_pos = region_key_to_pos(r_key, seamless_size);
  region_pos_to_world_pos(&region_pos, seamless_size)
}

pub fn region_key_to_pos(key: &[u32; 3], seamless_size: u32) -> [u32; 3] {
  [
    key[0] * seamless_size,
    key[1] * seamless_size,
    key[2] * seamless_size,
  ]
}

pub fn world_pos_to_region_key(pos: &[i64; 3], seamless_size: u32) -> [u32; 3] {
  let r_pos = world_pos_to_region_pos(pos, seamless_size);
  region_pos_to_key(&r_pos, seamless_size)
}

pub fn world_key_to_region_key(key: &[i64; 3], seamless_size: u32) -> [u32; 3] {
  let world_pos = world_key_to_pos(key, seamless_size);

  world_pos_to_region_key(&world_pos, seamless_size)
}

pub fn world_key_to_pos(key: &[i64; 3], seamless_size: u32) -> [i64; 3] {
  [
    key[0] * seamless_size as i64,
    key[1] * seamless_size as i64,
    key[2] * seamless_size as i64,
  ]
}

/* TODO: Might be convert what voxel_pos_to_key() implementation */
pub fn world_pos_to_key(pos: &[i64; 3], seamless_size: u32) -> [i64; 3] {
  let mut x = pos[0];
  let mut y = pos[1];
  let mut z = pos[2];
  let seamless_size_i64 = seamless_size as i64;

  // Between -0.epsilon to -seamless_size..., it should be -1
  if x < 0 {
    x -= seamless_size_i64;
  }
  if y < 0 {
    y -= seamless_size_i64;
  }
  if z < 0 {
    z -= seamless_size_i64;
  }
  [
    x / seamless_size_i64,
    y / seamless_size_i64,
    z / seamless_size_i64,
  ]
}

pub fn world_pos_to_key2(pos: &[i64; 3], seamless_size: u32) -> [i64; 3] {
  let seamless_size_i64 = seamless_size as i64;
  [
    pos[0] / seamless_size_i64,
    pos[1] / seamless_size_i64,
    pos[2] / seamless_size_i64,
  ]
}

pub fn region_pos_to_key(pos: &[u32; 3], seamless_size: u32) -> [u32; 3] {
  [
    pos[0] / seamless_size,
    pos[1] / seamless_size,
    pos[2] / seamless_size,
  ]
}

pub fn region_pos_to_world_key(pos: &[u32; 3], seamless_size: u32) -> [i64; 3] {
  let world_pos = region_pos_to_world_pos(pos, seamless_size);
  world_pos_to_key(&world_pos, seamless_size)
}

pub fn voxel_pos_to_key(pos: &[i64; 3], seamless_size: u32) -> [i64; 3] {
  let seamless_size_i64 = seamless_size as i64;

  let mut x = pos[0];
  let mut y = pos[1];
  let mut z = pos[2];

  // Between -0.epsilon to -seamless_size..., it should be -1
  if x < 0 {
    x += 1;
    x -= seamless_size_i64;
  }
  if y < 0 {
    y += 1;
    y -= seamless_size_i64;
  }
  if z < 0 {
    z += 1;
    z -= seamless_size_i64;
  }

  [
    x / seamless_size_i64,
    y / seamless_size_i64,
    z / seamless_size_i64,
  ]
}


// TODO: Create a f64 version for detecting the nearest target and new voxel position
pub fn region_pos_to_world_pos(pos: &[u32; 3], seamless_size: u32) -> [i64; 3] {
  let middle = region_middle_pos(seamless_size);
  [
    pos[0] as i64 - middle as i64,
    pos[1] as i64 - middle as i64,
    pos[2] as i64 - middle as i64,
  ]
}

pub fn world_pos_to_region_pos(pos: &[i64; 3], seamless_size: u32) -> [u32; 3] {
  let middle = region_middle_pos(seamless_size);
  [
    (pos[0] + middle as i64) as u32,
    (pos[1] + middle as i64) as u32,
    (pos[2] + middle as i64) as u32,
  ]
}

pub fn region_middle_pos(seamless_size: u32) -> u32 {
  (u32::MAX / 2) - ((u32::MAX / 2) % seamless_size)
}

pub fn same_coord(pos1: &[u32; 3], pos2: &[u32; 3]) -> bool {
  pos1[0] == pos2[0] && pos1[1] == pos2[1] && pos1[2] == pos2[2]
}

pub fn same_coord_i64(pos1: &[i64; 3], pos2: &[i64; 3]) -> bool {
  pos1[0] == pos2[0] && pos1[1] == pos2[1] && pos1[2] == pos2[2]
}

pub fn same_coord2(pos1: &[i64; 4], pos2: &[i64; 4]) -> bool {
  for index in 0..pos1.len() {
    if pos1[index] != pos2[index] {
      return false;
    }
  }
  true
}

pub fn chunk_mode(octree: &VoxelOctree) -> ChunkMode {
  let mut mode = ChunkMode::None;

  let size = octree.get_size();
  // let start = 1;
  // let end = size - 1;

  let start = 1;
  let end = size - 2;

  let mut is_air = false;
  let mut has_value = false;
  for x in start..end {
    for y in start..end {
      for z in start..end {
        let voxel = octree.get_voxel(x, y, z);

        if voxel == 1 {
          has_value = true;
        }
        
        if voxel == 0 {
          is_air = true;
        }
        
      }
    }
  }
  if (!is_air && has_value) || (is_air && !has_value) {
    mode = ChunkMode::Air;  // Should be renamed as empty
  }
  // if has_value && !is_air {
  //   mode = ChunkMode::Inner;
  // }
  if is_air && has_value {
    mode = ChunkMode::Loaded;
  }
  // println!("{} {}", is_air, has_value);
  mode
}

fn noise_elevation(x: &u32, z: &u32, middle: &i64, noise: OpenSimplex) -> i64 {
  let frequency = 0.0125;
  let height_scale = 16.0;
  let fx = (*x as i64 - middle) as f64 * frequency;
  let fz = (*z as i64 - middle) as f64 * frequency;
  let noise = noise.get([fx, fz]);
  let elevation = (noise * height_scale) as i64;
  elevation
}

pub fn get_dist(pos1: &[i64; 3], pos2: &[i64; 3]) -> f32 {
  let mut dist_sqr = 0;
  for (index, val) in pos1.iter().enumerate() {
    let diff = val - pos2[index];
    dist_sqr += diff * diff;
  }
  (dist_sqr as f32).sqrt()
}


/// Returns adjacent keys based on voxel scale
/// Scale should be rational number, otherwise 
/// the range will just round off to nearest value
pub fn adj_keys_by_scale(world_key: [i64; 3], _range: i64, scale: f32) -> Vec<[i64; 3]> {
  // TODO: Implement the range parameter

  /*
    Translate the world key to voxel key with voxel scale

  
   */

  let mut keys = Vec::new();

  let div = (1.0 / scale) as i64;
  let min = -div;
  let max = div * 2;

  let voxel_key = [
    world_key[0] * div,
    world_key[1] * div,
    world_key[2] * div,
  ];

  for x in min..max {
    for y in min..max {
      for z in min..max {
        let k = [
          voxel_key[0] + x,
          voxel_key[1] + y,
          voxel_key[2] + z
        ];
        keys.push(k);  
      }
    }
  }
  keys
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_voxel_pos_to_key() -> Result<(), String> {
    let chunk_size = 14;

    let pos = [0, 13, 14];
    let key = voxel_pos_to_key(&pos, chunk_size);
    assert_eq!(key, [0, 0, 1]);

    let pos = [27, 28, 41];
    let key = voxel_pos_to_key(&pos, chunk_size);
    assert_eq!(key, [1, 2, 2]);

    let pos = [-1, -14, -15];
    let key = voxel_pos_to_key(&pos, chunk_size);
    assert_eq!(key, [-1, -1, -2]);

    let pos = [-28, -29, -42];
    let key = voxel_pos_to_key(&pos, chunk_size);
    assert_eq!(key, [-2, -3, -3]);
    Ok(())
  }

  #[test]
  fn test_adjacent_keys() -> Result<(), String> {
    let key = [0, 0, 0];
    let range = 1;
    let keys = adjacent_keys(&key, range, true);

    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= key[i] - range);
        assert!(k[i] <= key[i] + range);
      }  
    }
    
    Ok(())
  }

  #[test]
  fn test_adjacent_keys_by_scale_1() -> Result<(), String> {
    let key = [0, 0, 0];
    let scale = 1.0;
    let range = (1.0 / scale) as i64;
    let keys = adj_keys_by_scale(key, range, 1.0);

    assert_eq!(keys.len(), 27);

    // Checking validity of values
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -1);
        assert!(k[i] <= 1);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }

    Ok(())
  }

  #[test]
  fn test_adjacent_keys_by_scale_1_key_10() -> Result<(), String> {
    let key = [10, 10, 10];
    let scale = 1.0;
    let range = (1.0 / scale) as i64;
    let keys = adj_keys_by_scale(key, range, 1.0);

    assert_eq!(keys.len(), 27);

    // Checking validity of values
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= 9);
        assert!(k[i] <= 11);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }

    Ok(())
  }


  #[test]
  fn test_adjacent_keys_by_scale_1_key_negative_10() -> Result<(), String> {
    let key = [-10, -10, -10];
    let scale = 1.0;
    let range = (1.0 / scale) as i64;
    let keys = adj_keys_by_scale(key, range, 1.0);

    assert_eq!(keys.len(), 27);

    // Checking validity of values
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -11);
        assert!(k[i] <= -9);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }

    Ok(())
  }


  #[test]
  fn test_adjacent_keys_by_scale_0_5() -> Result<(), String> {
    let key = [0, 0, 0];
    let scale = 0.5;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 216);
    /*
      Expected Result
      [-2, -2, -2] -> [3, 3, 3]
     */

    // Checking validity of values
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -2);
        assert!(k[i] <= 3);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }

  #[test]
  fn test_adjacent_keys_by_scale_0_5_key_negative_1() -> Result<(), String> {
    let key = [-1, -1, -1];
    let scale = 0.5;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 216);
    
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -4);
        assert!(k[i] <= 1);
      }
    }

    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }

  #[test]
  fn test_adjacent_keys_by_scale_0_5_key_negative_10() -> Result<(), String> {
    let key = [-10, -10, -10];
    let scale = 0.5;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 216);

    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -22);
        assert!(k[i] <= -17);
      }
    }

    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }


  #[test]
  fn test_adjacent_keys_by_scale_0_25() -> Result<(), String> {
    let key = [0, 0, 0];
    let scale = 0.25;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 1728);

    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -4);
        assert!(k[i] <= 7);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }

  #[test]
  fn test_adjacent_keys_by_scale_0_25_key_10() -> Result<(), String> {
    let key = [10, 10, 10];
    let scale = 0.25;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 1728);
    // Checking validity of values
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= 36);
        assert!(k[i] <= 47);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }


  #[test]
  fn test_adjacent_keys_by_scale_0_25_key_negative_10() -> Result<(), String> {
    let key = [-10, -10, -10];
    let scale = 0.25;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 1728);

    // Expected Result [-17, -17, -17] -> [-6, -6, -6]
    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -44);
        assert!(k[i] <= -33);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }


  #[test]
  fn test_adjacent_keys_by_scale_0_2() -> Result<(), String> {
    let key = [0, 0, 0];
    let scale = 0.2;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 3375);

    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -5);
        assert!(k[i] <= 9);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }

  #[test]
  fn test_adjacent_keys_by_scale_0_2_key_10() -> Result<(), String> {
    let key = [10, 10, 10];
    let scale = 0.2;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 3375);

    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= 45);
        assert!(k[i] <= 59);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }



  #[test]
  fn test_adjacent_keys_by_scale_0_2_key_negative_10() -> Result<(), String> {
    let key = [-10, -10, -10];
    let scale = 0.2;
    let range = 1;
    let keys = adj_keys_by_scale(key, range, scale);

    assert_eq!(keys.len(), 3375);

    for k in keys.iter() {
      for i in 0..k.len() {
        assert!(k[i] >= -55);
        assert!(k[i] <= -41);
      }
    }

    // Checking duplicates
    let mut eval_keys = Vec::new();
    for k in keys.iter() {
      assert!(!eval_keys.contains(k));
      if !eval_keys.contains(k) {
        eval_keys.push(k.clone());
      }
    }
    
    Ok(())
  }

  

}


/*
  Refactor later on
    Remove unused functions
*/