use bevy::prelude::*;
use flume;
use flume::{Sender, Receiver};

use web_sys::{MessageEvent, CustomEvent, CustomEventInit};
use wasm_bindgen::prelude::*;

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(LocalResource::default())
      .add_startup_system(init)
      .add_system(update);
  }
}

fn init(
  local_res: ResMut<LocalResource>,
) {
  receive_octree_data(local_res.send.clone());


  for i in 0..20000 {
    let key = [0, -1, 0];
    send_key(key);
  }
}

fn update(
  mut local_res: ResMut<LocalResource>,
  time: Res<Time>,
) {
  for bytes in local_res.recv.drain() {
    // info!("update() {:?}", bytes);
    info!("wasm recieved");
    
    let octree: Octree = bincode::deserialize(&bytes[..]).unwrap();
    // info!("bevy recv {:?}", octree.data);
  }

  if local_res.timer.tick(time.delta()).just_finished() {
    // info!("send key");

    // for i in 0..20 {
    //   let key = [0, -1, 0];
    //   send_key(key);
    // }
    
  }
}


pub fn receive_octree_data(send: Sender<Vec<u8>>) {
  let window = web_sys::window().unwrap();
  let cb2 = Closure::wrap(Box::new(move |event: MessageEvent| {
    info!("origin {}", event.origin());

    let data = event.data();
    let d = data.as_string().unwrap();
    let _ = send.send(array_bytes::hex2bytes(d).unwrap());
  }) as Box<dyn FnMut(MessageEvent)>);

  window
    .set_onmessage(Some(cb2.as_ref().unchecked_ref()));
  cb2.forget();
}


pub fn send_key(key: [i64; 3]) {
  let k: Vec<[u8; 8]> = key.iter().map(|a| a.to_be_bytes()).collect();
  let mut bytes = Vec::new();
  for k1 in k.iter() {
    bytes.append(&mut k1.to_vec());
  }
  let str = array_bytes::bytes2hex("", &bytes);

  let e = CustomEvent::new_with_event_init_dict(
    "key", CustomEventInit::new().detail(&JsValue::from_str(&str))
  ).unwrap();

  let window = web_sys::window().unwrap();
  let _ = window.dispatch_event(&e);
}


#[derive(Resource)]
struct LocalResource {
  send: Sender<Vec<u8>>,
  recv: Receiver<Vec<u8>>,
  timer: Timer,
}

impl Default for LocalResource {
  fn default() -> Self {
    let (send, recv) = flume::unbounded();
    Self {
      send: send,
      recv: recv,
      timer: Timer::from_seconds(100.0, TimerMode::Repeating),
    }
  }
}


use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Octree {
  pub key: [i64; 3],
  pub data: Vec<u8>,
}



/* 
#[cfg(test)]
mod tests {
  use super::{LocalResource, send_key, receive_octree_data};
  use voxels::chunk::adjacent_keys;
  use wasm_bindgen_futures::spawn_local;
  use wasm_mt::utils::console_ln;
  use wasm_bindgen_test::*;

  #[wasm_bindgen_test]
  async fn test_loading_voxels() -> Result<(), String> {
    

    // spawn_local(async move {
    //   // loop {

    //   // }
    // });
    console_ln!("test1");
    async move {
      console_ln!("test2");

      let res = LocalResource::default();
      let keys = adjacent_keys(&[0, 0, 0], 1, true);
      for key in keys.iter() {
        send_key(*key);
      }
      
      receive_octree_data(res.send.clone());

      let mut cur_index = 0;
      while let Ok(_) = res.recv.recv_async().await {
        cur_index += 1;

        console_ln!("cur_index {}", cur_index);

        if cur_index >= keys.len() {
          console_ln!("Break");
          return;
        }
      }
    }.await;


    Ok(())
  }
}
 */






 