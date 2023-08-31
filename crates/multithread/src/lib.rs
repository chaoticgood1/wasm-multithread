#![feature(async_closure)]

use wasm_mt_pool::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};
use js_sys::ArrayBuffer;

#[wasm_bindgen]
pub fn app() {
  spawn_local(async move {
    let ab_js = fetch_as_arraybuffer("crates/multithread/pkg/multithread.js").await.unwrap();
    let ab_wasm = fetch_as_arraybuffer("crates/multithread/pkg/multithread_bg.wasm").await.unwrap();
    run(ab_js, ab_wasm).await.unwrap();
  });
}

pub async fn run(ab_js: ArrayBuffer, ab_wasm: ArrayBuffer) -> Result<(), JsValue> {
  let size = 2;
  let pool = ThreadPool::new_with_arraybuffers(size, ab_js, ab_wasm)
    .and_init().await?;
  console_ln!("pool with {} threads is ready now!", size);

  let num = 4;
  console_ln!("{} closures:", num);
  for idx in 0..num {
    pool_exec!(pool, move || {
      console_ln!("idx: {}", idx); // not necessarily ordered
      Ok(JsValue::NULL)
    });
  }

  sleep(2_000).await; // Do sleep long enough to ensure all jobs are completed.
  assert_eq!(pool.count_pending_jobs(), 0);

  console_ln!("pool is getting dropped.");

  Ok(())
}





use bevy::prelude::*;
use flume;
use flume::{Sender, Receiver};
use voxels::chunk::chunk_manager::*;
use web_sys::{HtmlElement, HtmlInputElement, MessageEvent, InputEvent};


pub fn test_run() {
  info!("test_run");


  // let worker_handle = Rc::new(
  //   RefCell::new(Worker::new("./crates/multithread/worker.js").unwrap())
  // );



  // let (tx, rx) = flume::unbounded();

  // Rc::new(RefCell::new(Worker::new("./worker.js").unwrap()));

  // let pool = Rc::new(RefCell::new(WorkerPool::new(6).unwrap()));
  // pool.run(move || {
  //   // async_std::task::sleep(Duration::from_millis(10000));
  //   // tx.send("Testing");
  // });

  // for i in rx.iter() {
  //   info!("recieve {:?}", i);
  // }
  
}

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
  mut local_res: ResMut<LocalResource>,
) {
  // let (tx, rx) = flume::unbounded();
  // tx.send("1");
  // spawn_local(async move {
  //   let mt = WasmMt::new("crates/multithread/pkg/multithread.js").and_init().await.unwrap();
  //   while let Ok(msg) = rx.recv_async().await {
  //   //   console_ln!("Received: {}", msg);
  //     init_voxel(&mt).await;
  //   }
  // });
  let s = local_res.send.clone();
  let window = web_sys::window().unwrap();
  let cb2 = Closure::wrap(Box::new(move |event: MessageEvent| {
    let data = event.data();
    let d = data.as_string().unwrap();
    s.send(d.as_bytes().to_vec());
  }) as Box<dyn FnMut(MessageEvent)>);

  window
    .set_onmessage(Some(cb2.as_ref().unchecked_ref()));
  cb2.forget();
}

fn update(
  local_res: Res<LocalResource>,
) {
  for bytes in local_res.recv.drain() {
    info!("update() {:?}", bytes);
  }
}

#[derive(Resource)]
struct LocalResource {
  send: Sender<Vec<u8>>,
  recv: Receiver<Vec<u8>>,
}

impl Default for LocalResource {
  fn default() -> Self {
    let (send, recv) = flume::unbounded();
    Self {
      send: send,
      recv: recv,
    }
  }
}