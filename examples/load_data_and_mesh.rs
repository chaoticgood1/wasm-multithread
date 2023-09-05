use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::{FlyCam, NoCameraAndGrabPlugin, NoCameraPlayerPlugin};
use cfg_if::cfg_if;
use multithread::plugin::Octree;
use multithread::plugin::PluginResource;
use multithread::plugin::send_chunk;
use multithread::plugin::send_key;
use voxels::chunk::{adjacent_keys, world_pos_to_key};
use voxels::chunk::chunk_manager::Chunk;
use voxels::chunk::chunk_manager::ChunkManager;
use voxels::data::voxel_octree::VoxelOctree;
use voxels::utils::key_to_world_coord_f32;


cfg_if! {
  if #[cfg(target_arch = "wasm32")] {
    use multithread;
  }
}

fn main() {
  let mut app = App::new();
  app
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "Wasm Multithread".into(),
        resolution: (800., 600.).into(),
        present_mode: PresentMode::AutoVsync,
        fit_canvas_to_parent: true,
        prevent_default_event_handling: false,
        ..default()
      }),
      ..default()
    }));
  
  cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
      app
        .add_plugin(multithread::plugin::CustomPlugin);
    }
  }

  app
    .add_plugin(NoCameraPlayerPlugin)
    .insert_resource(LocalResource::default())
    .add_startup_system(add_cam)
    .add_system(load_data)
    .add_system(load_chunks)
    .run();
}

fn add_cam(
  mut commands: Commands,
  local_res: Res<LocalResource>,
) {
  let pos = Vec3::new(0.91, 11.64, -8.82);
  let key = world_pos_to_key(
    &[pos.x as i64, pos.y as i64, pos.z as i64], 
    local_res.manager.seamless_size()
  );

  commands
    .spawn(Camera3dBundle {
      transform: Transform::from_translation(pos)
        .looking_to(Vec3::new(0.03, -0.80, 0.59), Vec3::Y),
      ..Default::default()
    })
    .insert(FlyCam)
    .insert(Player {
      prev_key: [i64::MIN, i64::MIN, i64::MIN],
      key: key,
    });

  // Sun
  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      color: Color::rgb(0.98, 0.95, 0.82),
      shadows_enabled: true,
      illuminance: 10000.0,
      ..default()
    },
    transform: Transform::from_xyz(0.0, 50.0, 0.0)
      .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
    ..default()
  });

  commands.spawn(PointLightBundle {
    point_light: PointLight {
      intensity: 6000.0,
      ..Default::default()
    },
    transform: Transform::from_xyz(6.0, 15.0, 6.0),
    ..Default::default()
  });


  // for _ in 0..20 {
  //   send_key([0, -1, 0]);
    // let chunk = ChunkManager::new_chunk(&[0, 0, 0], 4, 4, manager.noise);
    // send_chunk(chunk);
  // }
  let keys = adjacent_keys(&[0, 0, 0], 1, true);
  for key in keys.iter() {
    // send_key(*key);

    let chunk = ChunkManager::new_chunk(key, 4, 4, local_res.manager.noise);
    send_chunk(chunk);
  }
  
}

fn load_chunks(
  local_res: Res<LocalResource>,
  keyboard_input: Res<Input<KeyCode>>,

  mut commands: Commands,
  chunk_graphics: Query<(Entity, &ChunkGraphics)>,

  mut players: Query<(&Transform, &mut Player)>,
) {
  for (trans, mut player) in &mut players {
    let t = trans.translation;
    let key = world_pos_to_key(
      &[t.x as i64, t.y as i64, t.z as i64], 
      local_res.manager.seamless_size()
    );

    if key != player.key {
      for (entity, _graphics) in &chunk_graphics {
        commands.entity(entity).despawn_recursive();
      }


      player.prev_key = player.key;
      player.key = key;
      let keys = adjacent_keys(&key, 1, true);
      info!("Initialize {} keys", keys.len());

      for key in keys.iter() {
        // send_key(*key);
        let chunk = ChunkManager::new_chunk(key, 4, 4, local_res.manager.noise);
        send_chunk(chunk);
      }
    }



    
  }


  // if keyboard_input.just_pressed(KeyCode::Space) {
  //   let keys = adjacent_keys(&[0, 0, 0], 1, true);
  //   info!("Initialize {} keys", keys.len());

  //   for key in keys.iter() {
  //     // send_key(*key);
  //     let chunk = ChunkManager::new_chunk(key, 4, 4, local_res.manager.noise);
  //     send_chunk(chunk);
  //   }

  //   local_res.keys_total = keys.len();
  //   local_res.keys_count = 0;
  //   local_res.duration = 0.0;
  //   local_res.done = false;


  //   for (entity, _graphics) in &chunk_graphics {
  //     commands.entity(entity).despawn_recursive();
  //   }
  // }
}


fn load_data(
  plugin_res: Res<PluginResource>,
  mut local_res: ResMut<LocalResource>,
  time: Res<Time>,

  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  chunk_graphics: Query<(Entity, &ChunkGraphics)>,
  
) {
  if local_res.keys_count != local_res.keys_total {
    local_res.duration += time.delta_seconds();
  }

  if !local_res.done && local_res.keys_count == local_res.keys_total {
    local_res.done = true;
    info!("Total duration {}", local_res.duration);
  }

  for bytes in plugin_res.recv.drain() {
    // info!("update() {:?}", bytes);
    info!("wasm_recv_data");
    local_res.keys_count += 1;
    
    let octree: Octree = bincode::deserialize(&bytes[..]).unwrap();
    let chunk = Chunk {
      key: octree.key,
      octree: VoxelOctree::new_from_bytes(octree.data),
      ..Default::default()
    };

    send_chunk(chunk);
  }

  for data in plugin_res.recv_mesh.drain() {
    info!("wasm_recv_mesh {:?}", data.key);

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone());
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.clone());
    render_mesh.set_indices(Some(Indices::U32(data.indices.clone())));

    let mesh_handle = meshes.add(render_mesh);
    let mut pos = key_to_world_coord_f32(&data.key, local_res.manager.seamless_size());

    let mat = materials.add(Color::rgb(0.7, 0.7, 0.7).into());
    commands
      .spawn(MaterialMeshBundle {
        mesh: mesh_handle,
        material: mat,
        transform: Transform::from_translation(pos.into()),
        ..default()
      })
      .insert(ChunkGraphics);
  }
}







#[derive(Resource)]
struct LocalResource {
  duration: f32,
  keys_count: usize,
  keys_total: usize,
  done: bool,
  manager: ChunkManager,
}

impl Default for LocalResource {
  fn default() -> Self {
    Self {
      duration: 0.0,
      keys_count: 0,
      keys_total: 0,
      done: true,
      manager: ChunkManager::default(),
    }
  }
}


#[derive(Component)]
pub struct ChunkGraphics;


#[derive(Component, Debug, Clone)]
pub struct Player {
  pub prev_key: [i64; 3],
  pub key: [i64; 3],
}




/*
  Load data from bevy
    Data
    Mesh
*/


