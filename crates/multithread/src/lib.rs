#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, Clamped};
use wasm_bindgen_futures::spawn_local;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, Counter, debug_ln, sleep};
use std::rc::Rc;

mod julia_set;


use web_sys::{HtmlInputElement, MessageEvent};

#[wasm_bindgen]
pub fn app() {

  let (tx, rx) = flume::unbounded();
  
  
  spawn_local(async move {
    let mt = WasmMt::new("crates/multithread/pkg/multithread.js").and_init().await.unwrap();

    console_ln!("done1");
    while let Ok(msg) = rx.recv_async().await {
      console_ln!("Received: {}", msg);
      run(&mt).await;
    }
    console_ln!("done4");
  });

  spawn_local(async move {
    // sleep(1000).await;
    // tx.send_async("test1").await;
    // sleep(1000).await;
    // tx.send_async("test2").await;
  });


  // let callback = Closure::new(move || {
  //   tx.send("test3");
  //   console_ln!("text_changed");
  //   // console::log_1(&"oninput callback triggered".into());
  //   // let document = web_sys::window().unwrap().document().unwrap();

  // }) as Closure<dyn FnMut(web_sys::MessageEvent)>);

  let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
    tx.send("test3");
    console_ln!("text_changed");
  }) as Box<dyn FnMut(MessageEvent)>);

  let document = web_sys::window().unwrap().document().unwrap();
  document
    .get_element_by_id("inputText")
    .expect("#inputNumber should exist")
    .dyn_ref::<HtmlInputElement>()
    .expect("#inputNumber should be a HtmlInputElement")
    .set_oninput(Some(callback.as_ref().unchecked_ref()));
  callback.forget();
}

fn get_canvas_context(id: &str) -> CanvasRenderingContext2d {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(id).unwrap();
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    ctx
}

fn compute_image(
    width: u32,
    height: u32,
    use_arraybuffer: bool,
) -> Result<JsValue, JsValue> {
    julia_set::compute(
        width, height,
        0.00375, // scale adjusted for 800x800
        -0.15, 0.65, // C
        use_arraybuffer)
}

fn draw_image(
    ctx: &CanvasRenderingContext2d,
    data: &JsValue,
    width: u32,
    height: u32,
    use_arraybuffer: bool,
) {
    if use_arraybuffer {
        let ab = data.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
        let mut vec = js_sys::Uint8Array::new(ab).to_vec();
        let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut vec), width, height)
            .unwrap()
            .into();
        ctx.put_image_data(&data, 0.0, 0.0).unwrap();
    } else {
        ctx.put_image_data(
            data.dyn_ref::<ImageData>().unwrap(),
            0.0, 0.0).unwrap();
    };
}

async fn run_task(th: &wasm_mt::Thread) {
    let width: u32 = 800;
    let height: u32 = 800;

    let th_id = th.get_id().unwrap();
    console_ln!("th_{}: starting", th_id);

    let use_arraybuffer = true;

    let data = if use_arraybuffer {
        // `ArrayBuffer` workaround
        exec!(th, move || compute_image(width, height, use_arraybuffer))
            .await.unwrap()

        // TODO Support 'transfer' functionality in `wasm_mt`. (That's not the bottle
        // of this example app though.)
    } else {
        // FIXME !!!!
        //
        exec!(th, move || compute_image(width, height, use_arraybuffer))
            .await.unwrap()
        //
        // On Chrome/Opera, `debug_ln!()` shows
        //   on_message(): msg: JsValue(null); oops, `.await` will hang!!
        // On the contrary, an `ImageData` created via JavaScript below works though.
        // It seems? there's something odd going on inside
        //   `web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(...`
        // TODO check.
        //
        // exec_js!(th, "
        //     // https://developer.mozilla.org/en-US/docs/Web/API/ImageData/ImageData
        //     const arr = new Uint8ClampedArray(4 * 800 * 800);
        //     for (let i = 0; i < arr.length; i += 4) {
        //         arr[i + 0] = 0;    // R value
        //         arr[i + 1] = 190;  // G value
        //         arr[i + 2] = 0;    // B value
        //         arr[i + 3] = 255;  // A value
        //     }
        //     let imageData = new ImageData(arr, 800);
        //     return imageData;
        // ").await.unwrap()
    };
    // console_ln!("data: {:?}", data);

    console_ln!("th_{}: done", th_id);

    let ctx = get_canvas_context("drawing");
    draw_image(&ctx, &data, width, height, use_arraybuffer);
}

// main thread
pub async fn run(mt: &WasmMt) -> Result<(), JsValue> {
    // Instead of putting
    //   <canvas id="drawing" width="800" height="800"></canvas>
    // in index.html, dynamically appending a new canvas for
    // `wasm_bindgen_test` in tests/web.rs.
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    document.body().unwrap().append_child(&canvas)?;
    canvas.set_width(800);
    canvas.set_height(800);
    canvas.set_id("drawing");

    let num = 4;

    // Prepare threads

    let mut v: Vec<wasm_mt::Thread> = vec![];
    for i in 0..num {
        let th = mt.thread().and_init().await?;
        th.set_id(&i.to_string());
        v.push(th);
    }

    // Serial executor

    let perf = web_sys::window().unwrap().performance().unwrap();
    let time_start = perf.now();
    for i in 0..num {
        run_task(&v[i]).await;
    }
    console_ln!("serial executor: {} tasks in {:.2}ms", num, perf.now() - time_start);

    // Parallel executor

    let time_start = perf.now();
    let count = Rc::new(Counter::new());
    for th in v {
        let count = count.clone();
        spawn_local(async move {
            run_task(&th).await;

            if count.inc() == num {
                let perf = web_sys::window().unwrap().performance().unwrap();
                console_ln!("parallel executor {} tasks in {:.2}ms", num, perf.now() - time_start);
            }
        });
    }
    //====
    // v.into_iter().for_each(|th| spawn_local(async move {
    //     run_task(&th).await;
    // }));

    Ok(())
}




/*
  How to send data to worker?

  How to get data from worker
*/






use bevy::prelude::*;
use flume;

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
      .add_startup_system(init)
      .add_system(update);
  }
}

fn init(
) {
}

fn update() {

}