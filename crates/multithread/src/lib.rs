#![feature(async_closure)]

use wasm_mt_pool::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};
use js_sys::ArrayBuffer;
use voxels::chunk::chunk_manager::*;
use flume::{Sender, Receiver};
use web_sys::CustomEvent;
use crate::plugin::Octree;

pub mod plugin;
mod test;

#[wasm_bindgen]
pub fn app() {
  let (send_queue, recv_queue) = flume::unbounded();
  spawn_local(async move {
    let ab_js = fetch_as_arraybuffer("crates/multithread/pkg/multithread.js").await.unwrap();
    let ab_wasm = fetch_as_arraybuffer("crates/multithread/pkg/multithread_bg.wasm").await.unwrap();
    run(recv_queue, ab_js, ab_wasm).await.unwrap();
  });

  recv_key_from_wasm(send_queue);
}

fn recv_key_from_wasm(send: Sender<[i64; 3]>) {
  let callback = Closure::wrap(Box::new(move |event: CustomEvent | {
    let data = event.detail().as_string().unwrap();
    let bytes = array_bytes::hex2bytes(data).unwrap();
    let a: Vec<i64> = bytes
      .chunks(8)
      .map(|a| {
        let a1: [u8; 8] = a[0..8].try_into().unwrap();
        i64::from_be_bytes(a1)
      })
      .collect();
    let key: [i64; 3] = a[0..3].try_into().unwrap();
    let _ = send.send(key);

    // console_ln!("recv key {:?}", key);
  }) as Box<dyn FnMut(CustomEvent)>);

  let window = web_sys::window().unwrap();
  let _ = window.add_event_listener_with_callback(
    "key",
    callback.as_ref().unchecked_ref()
  );

  callback.forget();
}

pub async fn run(
  recv: Receiver<[i64; 3]>,
  ab_js: ArrayBuffer, 
  ab_wasm: ArrayBuffer
) -> Result<(), JsValue> {
  let window = web_sys::window().unwrap();
  let threads = window.navigator().hardware_concurrency() as usize;
  // let size = 8;
  // console_ln!("num_cpus::get() {}", num_cpus::get());
  console_ln!("threads {}", threads);
  let pool = ThreadPool::new_with_arraybuffers(threads, ab_js, ab_wasm)
    .and_init().await?;

  while let Ok(msg) = recv.recv_async().await {
    let cb = move |result: Result<JsValue, JsValue>| {
      let r = result.unwrap();
      let ab = r.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
      let vec = js_sys::Uint8Array::new(ab);
  
      let bytes = vec.to_vec();
      let octree = Octree {
        key: msg,
        data: bytes,
      };
      
      let encoded: Vec<u8> = bincode::serialize(&octree).unwrap();
      let str = array_bytes::bytes2hex("", encoded);

      let window = web_sys::window().unwrap();
      let _ = window.post_message(&JsValue::from_str(&str), "/");
    };
  
    pool_exec!(pool, move || {
      let data = compute_voxel(msg);
      Ok(wasm_mt::utils::u8arr_from_vec(&data).buffer().into())
    }, cb);
  }

  Ok(())
}

fn compute_voxel(key: [i64; 3]) -> Vec<u8> {
  let manager = ChunkManager::default();
  let chunk = ChunkManager::new_chunk(&key, 4, 4, manager.noise);
  chunk.octree.data
}
