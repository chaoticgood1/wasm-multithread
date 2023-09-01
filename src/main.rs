use bevy::{prelude::*, window::PresentMode};
use cfg_if::cfg_if;


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
      app
        .add_plugin(multithread::CustomPlugin);
    }
  }

  app.run();
}


#[cfg(test)]
mod tests {

  #[test]
  fn test_array_bytes() -> Result<(), String> {
    let key = [0, -1, 0];
    // let a = array_bytes::

    Ok(())
  }
}
