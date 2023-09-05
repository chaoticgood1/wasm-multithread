#![feature(async_closure)]

use std::{cell::RefCell, rc::Rc, future::Future, task::{Context, Poll}};

use wasm_mt_pool::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};
use js_sys::ArrayBuffer;
use voxels::{chunk::chunk_manager::*, data::{voxel_octree::{MeshData, VoxelMode}, surface_nets::VoxelReuse}};
use flume::{Sender, Receiver};
use web_sys::{CustomEvent, HtmlInputElement, CustomEventInit};
use crate::plugin::Octree;

pub mod plugin;

#[wasm_bindgen]
pub fn app() {
  let (send_queue, recv_queue) = flume::unbounded();
  let (send_chunk, recv_chunk) = flume::unbounded();

  recv_key_from_wasm(send_queue);
  recv_chunk_from_wasm(send_chunk);
  

  spawn_local(async move {
    let ab_js = fetch_as_arraybuffer("crates/multithread/pkg/multithread.js").await.unwrap();
    let ab_wasm = fetch_as_arraybuffer("crates/multithread/pkg/multithread_bg.wasm").await.unwrap();
    let window = web_sys::window().expect("no global `window` exists");
    let max_threads = window.navigator().hardware_concurrency() as usize;

    let document = window.document().expect("should have a document on window");
    let e = document.get_element_by_id("concurrency").unwrap();
    let input = e.dyn_into::<HtmlInputElement>().unwrap();
    let threads = input.value().parse::<usize>().unwrap();
    console_ln!("max threads {} current threads {}", max_threads, threads);
    let pool = ThreadPool::new_with_arraybuffers(threads, ab_js, ab_wasm)
      .and_init().await.unwrap();

    load_data(&pool, recv_queue, recv_chunk).await;
  });

  
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

    console_ln!("recv key {:?}", key);
  }) as Box<dyn FnMut(CustomEvent)>);

  let window = web_sys::window().unwrap();
  let _ = window.add_event_listener_with_callback(
    &EventType::KeySend.to_string(),
    callback.as_ref().unchecked_ref()
  );

  callback.forget();
}

fn recv_chunk_from_wasm(send: Sender<Chunk>) {
  let callback = Closure::wrap(Box::new(move |event: CustomEvent | {
    let data = event.detail().as_string().unwrap();
    let bytes = array_bytes::hex2bytes(data).unwrap();
    let chunk: Chunk = bincode::deserialize(&bytes).unwrap();

    console_ln!("calc_chunk {:?}", chunk.key);
    let _ = send.send(chunk);
  }) as Box<dyn FnMut(CustomEvent)>);

  let window = web_sys::window().unwrap();
  let _ = window.add_event_listener_with_callback(
    &EventType::ChunkSend.to_string(),
    callback.as_ref().unchecked_ref()
  );

  callback.forget();
}

async fn load_data(
  pool: &ThreadPool,
  recv: Receiver<[i64; 3]>,
  recv_chunk: Receiver<Chunk>,
) {

  while let Ok(chunk) = recv_chunk.recv_async().await {
    console_ln!("load_data {:?}", chunk.key);
  }



  // let r = Rc::new(RefCell::new(Channels { 
  //   recv: recv.clone(),
  //   recv_chunk: recv_chunk.clone(),
  // }));
  let r = Rc::new(RefCell::new(Channels {}));

  while let Ok(res) = (
    ChannelFuture { recv: recv.clone(), recv_chunk: recv_chunk.clone() }
  ).await {
    console_ln!("res {} {}", res.keys.len(), res.chunks.len());

    for key in res.keys.iter() {
      let key = key.clone();
      let cb = move |result: Result<JsValue, JsValue>| {
        let r = result.unwrap();
        let ab = r.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
        let vec = js_sys::Uint8Array::new(ab);
    
        let bytes = vec.to_vec();
        let octree = Octree {
          key: key,
          data: bytes,
        };
        
        let encoded: Vec<u8> = bincode::serialize(&octree).unwrap();
        let str = array_bytes::bytes2hex("", encoded);
  
        let e = CustomEvent::new_with_event_init_dict(
          &EventType::KeyRecv.to_string(), CustomEventInit::new().detail(&JsValue::from_str(&str))
        ).unwrap();
  
        let window = web_sys::window().unwrap();
        let _ = window.dispatch_event(&e);
      };
    
      pool_exec!(pool, move || {
        let data = compute_voxel(key);
        Ok(wasm_mt::utils::u8arr_from_vec(&data).buffer().into())
      }, cb);
    }

    for chunk in res.chunks.iter() {
      let chunk = chunk.clone();
      // console_ln!("process_chunk2 {:?}", chunk.key);

      let cb = move |result: Result<JsValue, JsValue>| {
        let r = result.unwrap();
        let ab = r.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
        let vec = js_sys::Uint8Array::new(ab).to_vec();
        let str = array_bytes::bytes2hex("", vec);
  
        console_ln!("recv_chunk test");
        let e = CustomEvent::new_with_event_init_dict(
          &EventType::ChunkRecv.to_string(), CustomEventInit::new().detail(&JsValue::from_str(&str))
        ).unwrap();
  
        let window = web_sys::window().unwrap();
        let _ = window.dispatch_event(&e);
      };
  
      pool_exec!(pool, move || {
        let mesh = compute_mesh(chunk);
        let encoded: Vec<u8> = bincode::serialize(&mesh).unwrap();
  
        Ok(wasm_mt::utils::u8arr_from_vec(&encoded).buffer().into())
      }, cb);
    }
    
  }

  console_ln!("Done1");

}

fn compute_voxel(key: [i64; 3]) -> Vec<u8> {
  let manager = ChunkManager::default();
  let chunk = ChunkManager::new_chunk(&key, 4, 4, manager.noise);
  chunk.octree.data
}

fn compute_mesh(chunk: Chunk) -> MeshData {
  chunk.octree.compute_mesh(
    VoxelMode::SurfaceNets, 
    &mut VoxelReuse::default(), 
    &vec!([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]), 
    1.0, 
    chunk.key
  )
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum EventType {
  KeySend,
  KeyRecv,
  ChunkSend,
  ChunkRecv,
}

impl ToString for EventType {
  fn to_string(&self) -> String {
    match self {
      EventType::KeySend => String::from("KeySend"),
      EventType::KeyRecv => String::from("KeyRecv"),
      EventType::ChunkSend => String::from("ChunkSend"),
      EventType::ChunkRecv => String::from("ChunkRecv"),
    }
  }
}

struct Channels {
  // recv: Receiver<[i64; 3]>,
  // recv_chunk: Receiver<Chunk>,
}

type ChannerRef = Rc<RefCell<Channels>>;


struct ChannelFuture {
  // unit: ChannerRef,
  recv: Receiver<[i64; 3]>,
  recv_chunk: Receiver<Chunk>,
}

use std::pin::Pin;
impl Future for ChannelFuture {
  type Output = Result<Res, String>;
  fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
    // let recv = self.unit.borrow().recv.clone();
    // let recv_chunk = self.unit.borrow().recv_chunk.clone();

    console_ln!("Testing");

    let recv = self.recv.clone();
    let recv_chunk = self.recv_chunk.clone();
    
    let mut res = Res::default();
    for key in recv.drain() {
      res.keys.push(key);
    }
    for chunk in recv_chunk.drain() {
      res.chunks.push(chunk);
    }

    // let mut m = self.unit.borrow_mut();
    // m.recv = self.recv.clone();
    // m.recv_chunk = self.recv_chunk.clone();

    if !res.keys.is_empty() || !res.chunks.is_empty() {
      return Poll::Ready(Ok(res));
    }

    Poll::Pending
  }
}

// async fn load_res(unit: ChannerRef) -> Result<Res, String> {
//   ChannelFuture {
//     unit,
//   };
// }

#[derive(Default)]
struct Res {
  keys: Vec<[i64; 3]>,
  chunks: Vec<Chunk>,
}
