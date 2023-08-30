// // synchronously, using the browser, import out shim JS scripts
// // importScripts('raytrace_parallel.js');
// importScripts('./wasm/multithread.js');

// // Wait for the main thread to send us the shared module/memory. Once we've got
// // it, initialize it all with the `wasm_bindgen` global we imported via
// // `importScripts`.
// //
// // After our first message all subsequent messages are an entry point to run,
// // so we just do that.
// self.onmessage = event => {
//   let initialised = wasm_bindgen(...event.data).catch(err => {
//     // Propagate to main `onerror`:
//     setTimeout(() => {
//       throw err;
//     });
//     // Rethrow to keep promise rejected and prevent execution of further commands:
//     throw err;
//   });

//   self.onmessage = async event => {
//     // This will queue further commands up until the module is fully initialised:
//     await initialised;
//     wasm_bindgen.child_entry_point(event.data);
//   };
// };



// The worker has its own scope and no direct access to functions/objects of the
// global scope. We import the generated JS file to make `wasm_bindgen`
// available which we need to initialize our Wasm code.

// importScripts('./pkg/wasm_in_web_worker.js');
importScripts('./wasm/multithread.js');

console.log('Initializing worker')

// In the worker, we have a different struct that we want to use as in
// `index.js`.
const {NumberEval} = wasm_bindgen;

async function init_wasm_in_worker() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`.
    // await wasm_bindgen('./pkg/wasm_in_web_worker_bg.wasm');
    await wasm_bindgen('./wasm/multithread_bg.wasm');

    // Create a new object of the `NumberEval` struct.
    var num_eval = NumberEval.new();

    // Set callback to handle messages passed to the worker.
    self.onmessage = async event => {
        // By using methods of a struct as reaction to messages passed to the
        // worker, we can preserve our state between messages.
        var worker_result = num_eval.is_even(event.data);

        // Send response back to be handled by callback in main thread.
        self.postMessage(worker_result);
    };
};

init_wasm_in_worker();
