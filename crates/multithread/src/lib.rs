#![feature(async_closure)]

use wasm_mt_pool::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};
use js_sys::ArrayBuffer;

#[wasm_bindgen]
pub fn app() {
  // let (send, recv) = flume::unbounded();
  // spawn_local(async move {
  //   let ab_js = fetch_as_arraybuffer("crates/multithread/pkg/multithread.js").await.unwrap();
  //   let ab_wasm = fetch_as_arraybuffer("crates/multithread/pkg/multithread_bg.wasm").await.unwrap();
  //   run(recv, ab_js, ab_wasm).await.unwrap();

  //   console_ln!("spawn_local");
  // });

  // send.send([0, -1, 0]);

  // spawn_local(async move {
  //   loop {
  //     sleep(10_000).await;
  //     send.send([0, -1, 0]);
  //   }
  // });



  spawn_local(async move {
    let mut num = 0;
    loop {
      // let document = web_sys::window().unwrap().document().unwrap();
      // document
      //   .get_element_by_id("inputText")
      //   .expect("#inputNumber should exist")
      //   .dyn_ref::<HtmlInputElement>()
      //   .expect("#resultField should be a HtmlInputElement")
      //   .set_value(&num.to_string());


      // let window = web_sys::window().unwrap();
      // window.post_message(&JsValue::from_str("[0, -1, 0]"), "/");

      let key: Vec<i64> = vec![0, -1, 0];
      let k: Vec<[u8; 8]> = key.iter().map(|a| a.to_be_bytes()).collect();

      console_ln!("k {:?}", k);

      // let k = String::from_utf8_lossy(&key);
      // let e = CustomEvent::new_with_event_init_dict(
      //   "key", CustomEventInit::new().detail(&JsValue::from_str(&str))
      // ).unwrap();

      // let window = web_sys::window().unwrap();
      // window.dispatch_event(&e);

      // console_ln!("send");


      num += 1;
      sleep(1_000).await;
    }
  });


  let callback = Closure::wrap(Box::new(move |event: CustomEvent | {
    // console_ln!("text_changed");
    let data = event.detail().as_string();
    console_ln!("CustomEvent {:?}", data);



    // let window = web_sys::window().unwrap();
    // window.post_message(&JsValue::from_str("[0, -1, 0]"), "key_sender");

  }) as Box<dyn FnMut(CustomEvent)>);

  let window = web_sys::window().unwrap();
  window.add_event_listener_with_callback(
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
  let size = 2;
  let pool = ThreadPool::new_with_arraybuffers(size, ab_js, ab_wasm)
    .and_init().await?;
  
  while let Ok(msg) = recv.recv_async().await {
    console_ln!("msg {:?}", msg);

    let cb = move |result: Result<JsValue, JsValue>| {
      let r = result.unwrap();
      // console_ln!("callback: result: {:?}", r);

      let ab = r.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
      let mut vec = js_sys::Uint8Array::new(ab);

      let bytes = vec.to_vec();
      let str = String::from_utf8_lossy(&bytes);

      console_ln!("str {:?}", str);

      let window = web_sys::window().unwrap();
      window.post_message(&JsValue::from_str(&str), "/");
    };

    pool_exec!(pool, move || {
      let data = compute_voxel(msg);
      Ok(wasm_mt::utils::u8arr_from_vec(&data).buffer().into())

      // let bytes = data.to_vec();
      // let str = String::from_utf8_lossy(&bytes);

      // console_ln!("bytes {:?}", bytes);
      // console_ln!("str {:?}", str);
      // s.send(str.to_string());

      // let window = web_sys::window().unwrap();
      // window.post_message(&JsValue::from_str(&str), "/");

      // console_ln!("Test");

      // console_ln!("idx: {}", idx); // not necessarily ordered
      // Ok(JsValue::NULL)
    }, cb);

    
  }

  console_ln!("run");

  
  // let size = 2;
  // let pool = ThreadPool::new_with_arraybuffers(size, ab_js, ab_wasm)
  //   .and_init().await?;
  // console_ln!("pool with {} threads is ready now!", size);

  // let num = 4;
  // console_ln!("{} closures:", num);
  // for idx in 0..num {
  //   pool_exec!(pool, move || {
  //     console_ln!("idx: {}", idx); // not necessarily ordered
  //     Ok(JsValue::NULL)
  //   });
  // }

  // sleep(2_000).await; // Do sleep long enough to ensure all jobs are completed.
  // assert_eq!(pool.count_pending_jobs(), 0);

  // console_ln!("pool is getting dropped.");

  Ok(())
}

fn compute_voxel(key: [i64; 3]) -> Vec<u8> {
  let manager = ChunkManager::default();
  let chunk = ChunkManager::new_chunk(&key, 4, 4, manager.noise);
  chunk.octree.data




  // console_ln!("compute_voxel()");

  // Ok(wasm_mt::utils::u8arr_from_vec(&chunk.octree.data).buffer().into())
  // Ok(wasm_mt::utils::u8arr_from_vec(&[index as u8]).buffer().into())
}





use bevy::prelude::*;
use flume;
use flume::{Sender, Receiver};
use voxels::chunk::chunk_manager::*;
use web_sys::{HtmlElement, HtmlInputElement, MessageEvent, InputEvent, CustomEvent, CustomEventInit};
use js_sys::JSON;

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
    info!("origin {}", event.origin());

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