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

#[wasm_bindgen]
pub fn app() {
  let (send_queue, recv_queue) = flume::unbounded();
  let (send_process, recv_process) = flume::unbounded();
  let (send_to_wasm, recv_to_wasm) = flume::unbounded();
  spawn_local(async move {
    let ab_js = fetch_as_arraybuffer("crates/multithread/pkg/multithread.js").await.unwrap();
    let ab_wasm = fetch_as_arraybuffer("crates/multithread/pkg/multithread_bg.wasm").await.unwrap();
    run(recv_process, send_to_wasm, ab_js, ab_wasm).await.unwrap();
  });

  recv_key_from_wasm(send_queue);
  process_queued_keys(recv_queue, send_process, recv_to_wasm);
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
  send_to_wasm: Sender<[i64; 3]>,
  ab_js: ArrayBuffer, 
  ab_wasm: ArrayBuffer
) -> Result<(), JsValue> {
  let size = 8;
  let pool = ThreadPool::new_with_arraybuffers(size, ab_js, ab_wasm)
    .and_init().await?;

  while let Ok(msg) = recv.recv_async().await {
    let s = send_to_wasm.clone();
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

      let _ = s.send(msg);
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


fn process_queued_keys(
  recv_queue: Receiver<[i64; 3]>, 
  send_process: Sender<[i64; 3]>,
  recv_to_wasm: Receiver<[i64; 3]>,
) {

  let fps = 60;
  let sleep_time = 1000 / fps;
  spawn_local(async move {
    let max_threads = 8;
    let mut current_threads = 0;
    let mut queued_keys = Vec::new();

    loop {
      for key in recv_queue.drain() {
        queued_keys.push(key);

        // console_ln!("queued_keys.len() {}", queued_keys.len());
      }

      if queued_keys.len() > 0 && current_threads < max_threads {
        // console_ln!("1queued {} current {}", queued_keys.len(), current_threads);

        let key = queued_keys.pop().unwrap();
        let _ = send_process.send(key);
        current_threads += 1;

        // console_ln!("process {:?}: {}", key, current_threads);
      }

      for k in recv_to_wasm.drain() {
        current_threads -= 1;
        // console_ln!("drain {:?}: {}", k, current_threads);
      }

      // console_ln!("2queued {} current {}", queued_keys.len(), current_threads);

      sleep(sleep_time).await;
    }
  });
}
