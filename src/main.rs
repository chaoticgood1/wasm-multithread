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
        .add_plugin(multithread::plugin::CustomPlugin);
    }
  }

  app.run();
}


#[cfg(test)]
mod tests {

  #[test]
  fn test_array_bytes() -> Result<(), String> {
    let key: Vec<i64> = vec![0, -1, 0];
    let k: Vec<[u8; 8]> = key.iter().map(|a| a.to_be_bytes()).collect();

    
    let mut bytes = Vec::new();
    for k1 in k.iter() {
      bytes.append(&mut k1.to_vec());
    }

    // let str = String::from_utf8(bytes.clone()).unwrap();
    let str = array_bytes::bytes2hex("", &bytes);

    println!("k {:?}", k);
    println!("b {:?}", bytes);
    println!("str {:?}", str);

    println!("---");
    let b2 = array_bytes::hex2bytes(str).unwrap();
    // let a = <[u8; 8]>::try_from(bytes.as_slice());
    let a: Vec<i64> = b2
      .chunks(8)
      .map(|a| {
        let a1: [u8; 8] = a[0..8].try_into().unwrap();
        i64::from_be_bytes(a1)
      })
      .collect();

    println!("b2 {:?}", b2);
    println!("a {:?}", a);

    Ok(())
  }
}
