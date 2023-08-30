use bevy::{prelude::*, window::PresentMode};
use cfg_if::cfg_if;

use multithread;

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
      multithread::run();
    }
  }

  app.run();

}