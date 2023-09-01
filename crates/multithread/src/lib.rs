#![feature(async_closure)]

use wasm_mt_pool::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};
use js_sys::ArrayBuffer;

#[wasm_bindgen]
pub fn app() {
  let (send, recv) = flume::unbounded();
  spawn_local(async move {
    let ab_js = fetch_as_arraybuffer("crates/multithread/pkg/multithread.js").await.unwrap();
    let ab_wasm = fetch_as_arraybuffer("crates/multithread/pkg/multithread_bg.wasm").await.unwrap();
    run(recv, ab_js, ab_wasm).await.unwrap();

    console_ln!("spawn_local");
  });

  // send.send([0, -1, 0]);

  // spawn_local(async move {
  //   loop {
  //     sleep(10_000).await;
  //     send.send([0, -1, 0]);
  //   }
  // });


/* 
  spawn_local(async move {
    let mut num = 0;
    loop {

      let key: Vec<i64> = vec![0, -1, 0];
      
      let k: Vec<[u8; 8]> = key.iter().map(|a| a.to_be_bytes()).collect();
      let mut bytes = Vec::new();
      for k1 in k.iter() {
        bytes.append(&mut k1.to_vec());
      }
      let str = array_bytes::bytes2hex("", &bytes);

      // console_ln!("b {:?}", bytes.len());
      // console_ln!("str {:?}", str);

      // let k = String::from_utf8_lossy(&key);
      let e = CustomEvent::new_with_event_init_dict(
        "key", CustomEventInit::new().detail(&JsValue::from_str(&str))
      ).unwrap();

      let window = web_sys::window().unwrap();
      let _ = window.dispatch_event(&e);

      // console_ln!("send");


      num += 1;
      sleep(1_000).await;
    }
  });
 */

  let callback = Closure::wrap(Box::new(move |event: CustomEvent | {
    // console_ln!("text_changed");
    let data = event.detail().as_string().unwrap();
    let bytes = array_bytes::hex2bytes(data).unwrap();
    let a: Vec<i64> = bytes
      .chunks(8)
      .map(|a| {
        let a1: [u8; 8] = a[0..8].try_into().unwrap();
        i64::from_be_bytes(a1)
      })
      .collect();
    
    // console_ln!("bytes {:?}", bytes);
    
    let key: [i64; 3] = a[0..3].try_into().unwrap();
    let _ = send.send(key);

    console_ln!("recv key {:?}", key);


    // let b = data.as_bytes();


    // let v = b.to_vec();

    // let a = <&[u8; 8]>::try_from(b);
    // console_ln!("a {:?}", a);

    // v.chunks(8).iter().map(|a| a).collect();


    // for c in v.chunks(8) {
    //   let l: [u8; 8] = c.iter()
    //   console_ln!("c {:?}", i64::from_be_bytes(c));
    // }


    // console_ln!("CustomEvent {:?}", v);



    // let window = web_sys::window().unwrap();
    // window.post_message(&JsValue::from_str("[0, -1, 0]"), "key_sender");

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
  let size = 2;
  let pool = ThreadPool::new_with_arraybuffers(size, ab_js, ab_wasm)
    .and_init().await?;
  
  while let Ok(msg) = recv.recv_async().await {
    console_ln!("msg {:?}", msg);

    let cb = move |result: Result<JsValue, JsValue>| {
      let r = result.unwrap();
      // console_ln!("callback: result: {:?}", r);

      let ab = r.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
      let vec = js_sys::Uint8Array::new(ab);

      let bytes = vec.to_vec();
      let str = String::from_utf8_lossy(&bytes);

      // console_ln!("str {:?}", str);

      let window = web_sys::window().unwrap();
      let _ = window.post_message(&JsValue::from_str(&str), "/");
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
  local_res: ResMut<LocalResource>,
) {
  let s = local_res.send.clone();
  let window = web_sys::window().unwrap();
  let cb2 = Closure::wrap(Box::new(move |event: MessageEvent| {
    // info!("origin {}", event.origin());

    let data = event.data();
    let d = data.as_string().unwrap();
    let _ = s.send(d.as_bytes().to_vec());
  }) as Box<dyn FnMut(MessageEvent)>);

  window
    .set_onmessage(Some(cb2.as_ref().unchecked_ref()));
  cb2.forget();
}

fn update(
  mut local_res: ResMut<LocalResource>,
  time: Res<Time>,
) {
  for bytes in local_res.recv.drain() {
    // info!("update() {:?}", bytes);
    info!("bevy recv");
  }

  if local_res.timer.tick(time.delta()).just_finished() {
    info!("send key");

    let key: Vec<i64> = vec![0, -1, 0];
      
    let k: Vec<[u8; 8]> = key.iter().map(|a| a.to_be_bytes()).collect();
    let mut bytes = Vec::new();
    for k1 in k.iter() {
      bytes.append(&mut k1.to_vec());
    }
    let str = array_bytes::bytes2hex("", &bytes);

    // console_ln!("b {:?}", bytes.len());
    // console_ln!("str {:?}", str);

    // let k = String::from_utf8_lossy(&key);
    let e = CustomEvent::new_with_event_init_dict(
      "key", CustomEventInit::new().detail(&JsValue::from_str(&str))
    ).unwrap();

    let window = web_sys::window().unwrap();
    let _ = window.dispatch_event(&e);
  }
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
      timer: Timer::from_seconds(1.0, TimerMode::Repeating),
    }
  }
}