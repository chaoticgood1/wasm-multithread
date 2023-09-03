use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::FlyCam;
use cfg_if::cfg_if;
use multithread::plugin::PluginResource;
use multithread::plugin::Octree;
use multithread::plugin::send_key;
use voxels::chunk::adjacent_keys;


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

  info!("add_cam");
}

fn load_chunks(
  mut local_res: ResMut<LocalResource>,
  keyboard_input: Res<Input<KeyCode>>,
) {
  if keyboard_input.just_pressed(KeyCode::Space) {
    let keys = adjacent_keys(&[0, 0, 0], 5, true);
    info!("Initialize {} keys", keys.len());
    for key in keys.iter() {
      send_key(*key);
    }

    local_res.keys_total = keys.len();
    local_res.keys_count = 0;
    local_res.duration = 0.0;
    local_res.done = false;
  }
}


fn load_data(
  plugin_res: Res<PluginResource>,
  mut local_res: ResMut<LocalResource>,
  time: Res<Time>,
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
    info!("wasm recieved");
    local_res.keys_count += 1;
    
    // let octree: Octree = bincode::deserialize(&bytes[..]).unwrap();
    // info!("bevy recv {:?}", octree.data);
  }
}


#[derive(Resource)]
struct LocalResource {
  duration: f32,
  keys_count: usize,
  keys_total: usize,
  done: bool,
}

impl Default for LocalResource {
  fn default() -> Self {
    Self {
      duration: 0.0,
      keys_count: 0,
      keys_total: 0,
      done: true,
    }
  }
}









/*
  Load data from bevy
    Data
    Mesh
*/


