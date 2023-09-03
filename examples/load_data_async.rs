use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::FlyCam;
use cfg_if::cfg_if;


cfg_if! {
  if #[cfg(target_arch = "wasm32")] {
    // use multithread;
  }
}

fn main() {
  let mut app = App::new();
  app
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "Ironverse Editor".into(),
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
      // app
      //   .add_plugin(multithread::plugin::CustomPlugin);
    }
  }

  app
    .add_startup_system(add_cam)
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







/*
  Load data from bevy
    Data
    Mesh
*/


