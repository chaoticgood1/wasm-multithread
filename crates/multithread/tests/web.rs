#![feature(async_closure)]

use wasm_bindgen_test::wasm_bindgen_test_configure;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_bindgen_test::*;

// #[wasm_bindgen_test]
// fn pass() {
//   assert_eq!(1, 1);
// }

// #[wasm_bindgen_test]
// fn fail() {
//   assert_eq!(1, 1);
// }


#[cfg(test)]
mod tests {
  use multithread::plugin::{receive_octree_data, send_key};
  use voxels::chunk::adjacent_keys;
  use wasm_bindgen_futures::spawn_local;
  use wasm_mt::utils::console_ln;
  use wasm_bindgen_test::*;

  // #[wasm_bindgen_test]
  // fn pass() {
  //   assert_eq!(1, 1);
  // }

  // #[wasm_bindgen_test]
  // async fn test_loading_voxels() {
  //   // spawn_local(async move {
  //   //   // loop {

  //   //   // }
  //   // });
  //   console_ln!("test1");
    

  //   async move {
  //     console_ln!("test2");

  //     let (send, recv) = flume::unbounded();
  //     let keys = adjacent_keys(&[0, 0, 0], 1, true);
  //     for key in keys.iter() {
  //       send_key(*key);
  //     }
      
  //     receive_octree_data(send.clone());

  //     while let Ok(_) = recv.recv_async().await {
  //       console_ln!("recv");
  //     }

  //   //   let mut cur_index = 0;
  //   //   while let Ok(_) = recv.recv_async().await {
  //   //     cur_index += 1;

  //   //     console_ln!("cur_index {}", cur_index);

  //   //     if cur_index >= keys.len() {
  //   //       console_ln!("Break");
  //   //       return;
  //   //     }
  //   //   }
  //   }.await;
  // }

  #[wasm_bindgen_test]
  async fn test_loading_voxels() {
    // spawn_local(async move {
    //   // loop {

    //   // }
    // });
    console_ln!("test1");

    let (send, recv) = flume::unbounded();
    let keys = adjacent_keys(&[0, 0, 0], 1, true);
    for key in keys.iter() {
      send_key(*key);
    }
    
    receive_octree_data(send.clone());

    while let Ok(_) = recv.recv_async().await {
      console_ln!("recv");
    }
  }


}

