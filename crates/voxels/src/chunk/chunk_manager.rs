use crate::{data::voxel_octree::{VoxelOctree, ParentValueType}, utils::get_chunk_coords};
use super::*;
use hashbrown::HashMap;
use noise::*;

#[derive(Default)]
pub struct LoadedChunk {
  pub key: [u32; 3],
  pub ttl: f32,
}

#[derive(Default)]
pub struct SubscribeData {
  pub chunks: HashMap<[u32; 3], VoxelOctree>,
  pub rays: Vec<[f32; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkMode {
  None,
  Loaded,
  Unloaded,
  Air,
  Inner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deployment {
  Production,
  Development,
}

#[derive(Clone, Debug)]
pub struct Chunk {
  pub key: [i64; 3],
  pub octree: VoxelOctree,
  pub mode: ChunkMode,
  pub is_default: bool,
}

impl Default for Chunk {
  fn default() -> Chunk {
    Chunk {
      key: [0, 0, 0],
      octree: VoxelOctree::new(0, 4),
      mode: ChunkMode::Unloaded,
      is_default: true,
    }
  }
}

#[derive(Clone)]
pub struct ChunkManager {
  pub chunks: HashMap<[i64; 3], Chunk>,
  pub colliders: HashMap<[i64; 3], Chunk>,
  pub depth: u32,
  pub chunk_size: u32,
  pub offset: u32,
  pub noise: OpenSimplex,
  pub height_scale: f64,
  pub frequency: f64,

  pub voxel_scale: f32,
  pub range: u8,
  pub colors: Vec<[f32; 3]>,
}

impl Default for ChunkManager {
  fn default() -> Self {
    let depth = 4;
    // let loop_count = 3; // indices/axes being used, [x, y, z]
    // let voxel_reuse = VoxelReuse::new(depth, loop_count);
    
    let noise = OpenSimplex::new().set_seed(1234);
    let offset = 2;
    let chunk_size = 2_i32.pow(depth) as u32;

    ChunkManager {
      chunks: HashMap::new(),
      colliders: HashMap::new(),
      depth: depth,
      chunk_size: chunk_size,
      offset: offset,
      noise: noise,
      height_scale: 16.0,
      frequency: 0.0125,
      voxel_scale: 1.0,
      range: 1,
      colors: vec![
        [1.0, 0.0, 0.0], 
        [0.0, 1.0, 0.0], 
        [0.0, 0.0, 1.0], 
        [0.0, 0.0, 0.0],

        [0.2, 0.0, 0.0],
        [0.4, 0.0, 0.0],
        [0.6, 0.0, 0.0],
        [0.8, 0.0, 0.0],

        [0.0, 0.2, 0.0],
        [0.0, 0.4, 0.0],
      ],
    }
  }
}

impl ChunkManager {

  pub fn new(
    depth: u32, 
    voxel_scale: f32, 
    range: u8,
    colors: Vec<[f32; 3]>,  
  ) -> Self {
    let noise = OpenSimplex::new().set_seed(1234);
    let offset = 2;
    let chunk_size = 2_i32.pow(depth) as u32;

    ChunkManager {
      chunks: HashMap::new(),
      colliders: HashMap::new(),
      depth: depth,
      chunk_size: chunk_size,
      offset: offset,
      noise: noise,
      height_scale: 16.0,
      frequency: 0.0125,
      voxel_scale: voxel_scale,
      range: range,
      colors: colors,
    }
  }

  /* TODO: Remove later */
  pub fn set_voxel1(&mut self, pos: &[i64; 3], voxel: u8) -> Vec<[i64; 3]> {
    let mut keys = Vec::new();
    let seamless_size = self.seamless_size();

    let w_key = world_pos_to_key(pos, seamless_size);
    // let w_key = voxel_pos_to_key(pos, seamless_size);
   

    for x in -1..1 {
      for y in -1..1 {
        for z in -1..1 {
          let key_x = w_key[0] + x;
          let key_y = w_key[1] + y;
          let key_z = w_key[2] + z;
          let key = [key_x, key_y, key_z];

          let chunk_sizei64 = (self.chunk_size - 1) as i64;
          let size = seamless_size as i64;

          let min_x = key_x * size;
          let max_x = min_x + chunk_sizei64;
          let min_y = key_y * size;
          let max_y = min_y + chunk_sizei64;
          let min_z = key_z * size;
          let max_z = min_z + chunk_sizei64;
          if pos[0] >= min_x
            && pos[0] <= max_x
            && pos[1] >= min_y
            && pos[1] <= max_y
            && pos[2] >= min_z
            && pos[2] <= max_z
          {
            let local_x = (pos[0] - min_x) as u32;
            let local_y = (pos[1] - min_y) as u32;
            let local_z = (pos[2] - min_z) as u32;

            if let Some(chunk) = self.get_chunk_mut(&key) {
              chunk.octree.set_voxel(local_x, local_y, local_z, voxel);

              //  FIXME: Check performance hit
              chunk.mode = chunk_mode(&chunk.octree);

              // if same_coord_i64(&[-1, 0, -1], &key) {
              //   c.mode = chunk_mode(&c.octree);
              // }
              chunk.is_default = false;
              keys.push(key);
            } else {
              let mut chunk = self.new_chunk3(&key, self.depth as u8);
              chunk.octree.set_voxel(local_x, local_y, local_z, voxel);

              //  FIXME: Check performance hit
              chunk.mode = chunk_mode(&chunk.octree);

              chunk.is_default = false;
              self.set_chunk(&key, &chunk);
              keys.push(key);
            }
          }
        }
      }
    }
    return keys;
  }

  pub fn set_voxel2(&mut self, pos: &[i64; 3], voxel: u8) -> Vec<([i64; 3], Chunk)> {
    let mut chunks = Vec::new();
    let chunk_size = self.chunk_size;
    let seamless_size = self.seamless_size();

    let coords = get_chunk_coords(pos, chunk_size, seamless_size);
    for coord in coords.iter() {
      let key = &coord.key;
      let local = &coord.local;

      // Refactor: Chunk already have keys, remove mapping here later

      if let Some(chunk) = self.get_chunk_mut(key) {
        chunk.octree.set_voxel(local[0], local[1], local[2], voxel);
        chunks.push((key.clone(), chunk.clone()));
      } else {
        let mut chunk = self.new_chunk3(&key, self.depth as u8);
        chunk.octree.set_voxel(local[0], local[1], local[2], voxel);
        self.set_chunk(key, &chunk);
        chunks.push((key.clone(), chunk.clone()));
      }
    }
    chunks
  }

  /**
    Returns 0 if the chunk is not loaded containing the coordinate
   */
  pub fn get_voxel(&self, pos: &[i64; 3]) -> u8 {
    let seamless_size = self.seamless_size();
    let key = voxel_pos_to_key(pos, seamless_size);
    // let key = world_pos_to_key(pos, seamless_size);
    
    let octree = match self.get_octree(&pos) {
      Some(o) => o,
      None => return 0
    };

    let sizei64 = seamless_size as i64;
    let local_x = pos[0] - (key[0] * sizei64);
    let local_y = pos[1] - (key[1] * sizei64);
    let local_z = pos[2] - (key[2] * sizei64);

    // println!("key1 {:?} local {} {} {}", key, local_x, local_y, local_z);

    octree.get_voxel(local_x as u32, local_y as u32, local_z as u32)
  }

  /**
   * Returns None if the chunk is not loaded containing the coordinate 
   */
  pub fn get_voxel_safe(&self, pos: &[i64; 3]) -> Option<u8> {
    let seamless_size = self.seamless_size();
    let key = voxel_pos_to_key(pos, seamless_size);
    
    let octree = match self.get_octree(&pos) {
      Some(o) => o,
      None => return None
    };

    let sizei64 = seamless_size as i64;
    let local_x = pos[0] - (key[0] * sizei64);
    let local_y = pos[1] - (key[1] * sizei64);
    let local_z = pos[2] - (key[2] * sizei64);

    // println!("key1 {:?} local {} {} {}", key, local_x, local_y, local_z);

    Some(octree.get_voxel(local_x as u32, local_y as u32, local_z as u32))
  }

  fn get_octree(&self, pos: &[i64; 3]) -> Option<&VoxelOctree> {
    let seamless_size = self.seamless_size();
    let key = &voxel_pos_to_key(pos, seamless_size);
    // let key = &world_pos_to_key(pos, seamless_size);
    // println!("get_octree() Key {:?}", key);
    let chunk = match self.get_chunk(key) {
      Some(o) => o,
      None => return None,
    };
    Some(&chunk.octree)
  }

  pub fn seamless_size(&self) -> u32 {
    self.chunk_size - self.offset
  }

  /**
    TODO: Deprecate later, in favor of using world coord instead of region coord
          We just have to do coord conversion when it is needed in the future
  */
  pub fn new_chunk(key: &[i64; 3], depth: u8, lod_level: u8, noise: OpenSimplex) -> Chunk {
    let size = 2_i32.pow(depth as u32) as u32;
    // if lod_level > depth {
    //   panic!("lod_level: {} cannot be higher than depth: {}", lod_level, depth);
    // }
    let seamless_size = size - 2;
    let region_key = world_key_to_region_key(key, seamless_size);

    let region_middle_pos = region_middle_pos(seamless_size) as i64;
    let start_x = (region_key[0] * seamless_size) + 0;
    let start_y = (region_key[1] * seamless_size) + 0;
    let start_z = (region_key[2] * seamless_size) + 0;

    let new_octree = VoxelOctree::new(0, depth);
    let mut chunk = Chunk {
      key: key.clone(),
      octree: new_octree,
      mode: ChunkMode::None,
      is_default: true,
    };

    let mut has_air = false;
    let mut has_value = false;
    let mut data = Vec::new();

    let diff = depth - lod_level;
    let step = diff as usize + 1;

    let start = 0;
    let end = size;
    for octree_x in (start..end).step_by(step) {
      for octree_y in (start..end).step_by(step) {
        for octree_z in (start..end).step_by(step) {
          let x = start_x + octree_x;
          let y = start_y + octree_y;
          let z = start_z + octree_z;
          
          let elevation = noise_elevation(&x, &z, &region_middle_pos, noise);
          let mid_y = y as i64 - region_middle_pos;

          /* Uncomment this later, testing for now */
          let voxel = if mid_y < elevation { 1 } else { 0 };
          // let voxel = if mid_y < 0 { 1 } else { 0 };
          data.push([octree_x, octree_y, octree_z, voxel]);

          /*
            TODO:
              Conditions to determine if Chunk is needed to be rendered and create collider
                Mode:
                  Empty/Air
                  Inner
                  Visible
                Air
                  If all values are 0
                Inner
                  If all values are 1
                Visible
                  ?
          */
          if octree_x <= end - 1
            && octree_y <= end - 1
            && octree_z <= end - 1
          {
            if voxel == 0 {
              has_air = true;
              // println!("Air {} {} {}", octree_x, octree_y, octree_z);
            }
            if voxel == 1 {
              has_value = true;
              // println!("Voxel {} {} {}", octree_x, octree_y, octree_z);
            }
          }
        }
      }
    }

    chunk.octree = VoxelOctree::new_from_3d_array(0, depth, &data, ParentValueType::Lod);
    // chunk.mode = chunk_mode(&chunk.octree);

    /*
      TODO: Have to update mode detector
    */
    if (!has_air && has_value) || (has_air && !has_value) {
      chunk.mode = ChunkMode::Air;  // Should be renamed as empty
    }
    if has_air && has_value {
      chunk.mode = ChunkMode::Loaded;
    }
    // println!("{} {} {}", has_air, has_value, end - 2);
    chunk
  }

  pub fn new_chunk2(key: &[i64; 3], depth: u32, lod_level: u8, noise: OpenSimplex) -> Chunk {
    ChunkManager::new_chunk(key, depth as u8, lod_level, noise)
  }

  pub fn new_chunk3(&self, key: &[i64; 3], lod_level: u8) -> Chunk {
    ChunkManager::new_chunk(key, self.depth as u8, lod_level, self.noise)
  }

  pub fn chunk_mode(self: &Self, key: &[i64; 3]) -> ChunkMode {
    let chunk = self.chunks.get(key);
    let mut mode = ChunkMode::Unloaded;
    if chunk.is_some() {
      mode = chunk.unwrap().mode;
    }
    mode
  }

  pub fn get_chunk(&self, key: &[i64; 3]) -> Option<&Chunk> {
    /* Later on, implement Spatial Partition or R-trees? */
    self.chunks.get(key)
  }

  pub fn get_chunk_mut(&mut self, key: &[i64; 3]) -> Option<&mut Chunk> {
    /* Later on, implement Spatial Partition or R-trees? */
    self.chunks.get_mut(key)
  }

  pub fn set_chunk(&mut self, key: &[i64; 3], chunk: &Chunk) {
    let c = self.chunks.get(key);
    if c.is_some() {
      if !chunk.is_default {
        self.chunks.insert(key.clone(), chunk.clone());
      }
    } else {
      self.chunks.insert(key.clone(), chunk.clone());
    }
  }

  pub fn remove_chunk(&mut self, key: &[i64; 3]) {
    let chunk_op = self.get_chunk(key);
    if chunk_op.is_some() {
      let chunk = chunk_op.unwrap();
      if chunk.is_default {
        self.chunks.remove(key);
      }
    }
  }

  pub fn len(&self) -> usize {
    self.chunks.len()
  }

  

  pub fn get_adj_chunks(&mut self, key: [i64; 3]) -> Vec<Chunk> {
    let mut chunks = Vec::new();

    let keys = adjacent_keys(&key, self.range as i64, true);
    for key in keys.iter() {
      let res = self.chunks.get(key);
      if res.is_some() {
        chunks.push(res.unwrap().clone());
      }

      if res.is_none() {
        let c = self.new_chunk3(key, self.depth as u8);
        chunks.push(c.clone());
        self.chunks.insert(*key, c);
      }
    }

    chunks
  }


}

#[cfg(test)]
mod tests {
  use crate::{data::{surface_nets::{GridPosition, VoxelReuse}, voxel_octree::VoxelMode}, utils::get_length};
  use super::*;

  #[test]
  fn test_set_and_get_voxel() -> Result<(), String> {
    let mut chunk_manager = ChunkManager::default();

    let start = -10;
    let end = 10;
    let mut new_value = 0;
    
    for x in start..end {
      for y in start..end {
        for z in start..end {
          new_value = if new_value == 255 { 0 } else { new_value + 1 };
          let pos = &[x, y, z];
          chunk_manager.set_voxel1(pos, new_value);
        }
      }
    }

    new_value = 0;
    for x in start..end {
      for y in start..end {
        for z in start..end {
          new_value = if new_value == 255 { 0 } else { new_value + 1 };
          let expected = new_value;
          let pos = &[x, y, z];
          let result = chunk_manager.get_voxel(pos);

          assert_eq!(result, expected, "at pos: {:?}", (x, y, z));
        }
      }
    }

    Ok(())
  }

  #[test]
  fn test_chunk_mode() -> Result<(), String> {
    let depth = 4;
    let len = get_length(depth as u8);
    let mut voxels = vec![0; len];
    let size = (2 as u32).pow(depth as u32);
    
    let grid_pos_len = get_length(size as u8 - 1);
    let mut grid_pos = Vec::new();
    for i in 0..grid_pos_len {
      grid_pos.push(GridPosition ::default());
    }

    let mut voxel_reuse = VoxelReuse {
      voxels: voxels,
      grid_pos: grid_pos,
      size: size,
    };

    let chunk_size = 16;
    let mut chunk_manager = ChunkManager::default();

    let color = vec![[0.0, 0.0, 0.0]];

    let keys = adjacent_keys(&[0, 0, 0], 5, true);
    for key in keys.iter() {
      let chunk = chunk_manager.new_chunk3(key, chunk_manager.depth as u8);
      let d = chunk.octree.compute_mesh(
        VoxelMode::SurfaceNets, 
        &mut voxel_reuse,
        &color,
        1.0
      );
      if d.indices.len() != 0 {
        assert_eq!(chunk.mode, ChunkMode::Loaded, "key {:?}", key);
      } else {
        assert_eq!(chunk.mode, ChunkMode::Air, "key {:?}", key);
      }
    }

    // let key = [-3, -1, 1];
    // let key = [-5, -1, -2];
    // let chunk = chunk_manager.new_chunk3(&key, chunk_manager.depth as u8);
    // assert_eq!(chunk.mode, ChunkMode::Loaded, "key {:?}", key);

    Ok(())
  }
}



/*
  Need to refactor ChunkManger(Defer)
  Make new features work first
  Then refactor once approved
*/

