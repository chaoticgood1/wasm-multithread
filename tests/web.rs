

#![feature(async_closure)]

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);


// use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};

// #[wasm_bindgen_test]
// async fn basics() {
//   console_ln!("basics");
// }

// #[cfg(test)]
// mod tests {

//   use wasm_bindgen_test::*;

//   #[wasm_bindgen_test]
//   fn pass() {
//     assert_eq!(1, 1);
//   }

//   #[wasm_bindgen_test]
//   fn fail() {
//     assert_eq!(1, 1);
//   }
// }





use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn pass() {
  assert_eq!(1, 1);
}

#[wasm_bindgen_test]
fn fail() {
  assert_eq!(1, 1);
}
