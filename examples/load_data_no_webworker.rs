use bevy::{prelude::*, window::PresentMode, tasks::{Task, AsyncComputeTaskPool}, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}};
use bevy_flycam::FlyCam;
use flume::{Sender, Receiver};
use voxels::chunk::{adjacent_keys, chunk_manager::{Chunk, ChunkManager}};
use instant::Instant;

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

  app
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    // .add_plugin(LogDiagnosticsPlugin::default())
    .insert_resource(LocalResource::default())
    .add_startup_system(add_cam)
    .add_system(load_data)
    .add_system(load_chunks)
    .run();
}

fn add_cam(
  mut commands: Commands,
) {
  commands
    .spawn(Camera3dBundle {
      transform: Transform::from_xyz(0.91, 11.64, -8.82)
        .looking_to(Vec3::new(0.03, -0.80, 0.59), Vec3::Y),
      ..Default::default()
    })
    .insert(FlyCam);

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
}

fn load_chunks(
  mut local_res: ResMut<LocalResource>,
  keyboard_input: Res<Input<KeyCode>>,
) {
  if keyboard_input.just_pressed(KeyCode::Space) {
    

    let keys = adjacent_keys(&[0, 0, 0], 5, true);
    info!("Initialize {} keys", keys.len());

    local_res.queued_keys.append(&mut keys.clone());
    
    local_res.keys_total = keys.len();
    local_res.keys_count = 0;
    local_res.duration = 0.0;
    local_res.done = false;
  }
}


fn load_data(
  mut local_res: ResMut<LocalResource>,
  time: Res<Time>,
) {

  let limit = 1.0 / 30.0;

  if time.delta_seconds() < limit {
    let mut current_time = 0.0;
    let noise = local_res.manager.noise.clone();

    loop {
      let start = Instant::now();
      if local_res.queued_keys.len() > 0 {
        let key = local_res.queued_keys.pop().unwrap();
        let chunk = ChunkManager::new_chunk(&key, 4, 4, noise);

        let _ = local_res.send.send(chunk);
      } else {
        break;
      }

      let duration = start.elapsed();
      current_time += duration.as_secs_f32();

      if current_time > limit {
        break;
      }
    }
  }




  // info!("update {:?}", time.delta_seconds());

  let r = local_res.recv.clone();
  for chunk in r.drain() {
    local_res.keys_count += 1;

    // info!("{:?}", chunk.key);
  }

  if local_res.keys_count != local_res.keys_total {
    local_res.duration += time.delta_seconds();
  }

  if !local_res.done && local_res.keys_count == local_res.keys_total {
    local_res.done = true;
    info!("Total duration {}", local_res.duration);
  }
}


#[derive(Resource)]
struct LocalResource {
  duration: f32,
  keys_count: usize,
  keys_total: usize,
  done: bool,

  send: Sender<Chunk>,
  recv: Receiver<Chunk>,

  queued_keys: Vec<[i64; 3]>,
  manager: ChunkManager,
}

impl Default for LocalResource {
  fn default() -> Self {
    let (send, recv) = flume::unbounded();

    Self {
      duration: 0.0,
      keys_count: 0,
      keys_total: 0,
      done: true,
      send: send,
      recv: recv,
      queued_keys: Vec::new(),
      manager: ChunkManager::default(),
    }
  }
}



#[derive(Component)]
struct ChunkData {
  chunk: Chunk
}

impl Default for ChunkData {
  fn default() -> Self {
    Self {
      chunk: Chunk::default(),
    }
  }
}





/*
  Load data from bevy
    Data
    Mesh
*/


