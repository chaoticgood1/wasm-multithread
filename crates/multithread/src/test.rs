
/* 
#[cfg(test)]
mod tests {
  use voxels::chunk::adjacent_keys;
  use wasm_mt::console_ln;
  use crate::plugin::{send_key, receive_octree_data};

  #[test]
  fn test_voxel_pos_to_key() -> Result<(), String> {
    let (send, recv) = flume::unbounded();
    receive_octree_data(send.clone());

    let keys = adjacent_keys(&[0, 0, 0], 1, true);
    for key in keys.iter() {
      send_key(*key);
    }

    let mut cur_index = 0;
    'main: loop {
      for _ in recv.drain() {
        cur_index += 1;

        console_ln!("cur_index {}", cur_index);

        if cur_index >= keys.len() {
          console_ln!("Break");
          break 'main;
        }
      }
    }

    Ok(())
  }
}

 */












/* #[cfg(test)]
mod tests {
  use voxels::chunk::adjacent_keys;
  use wasm_bindgen_futures::spawn_local;
  use wasm_mt::utils::console_ln;
  use wasm_bindgen_test::*;
  use crate::plugin::{send_key, receive_octree_data};

  #[wasm_bindgen_test]
  async fn test_loading_voxels() -> Result<(), String> {
    // spawn_local(async move {
    //   // loop {

    //   // }
    // });
    console_ln!("test1");
    async move {
      console_ln!("test2");

      let (send, recv) = flume::unbounded();
      let keys = adjacent_keys(&[0, 0, 0], 1, true);
      for key in keys.iter() {
        send_key(*key);
      }
      
      receive_octree_data(send.clone());

      let mut cur_index = 0;
      while let Ok(_) = recv.recv_async().await {
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
} */



