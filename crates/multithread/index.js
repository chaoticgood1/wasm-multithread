export default async function (name) {
  // const pkgJs = await (await fetch(`./pkg/${name}.js`)).text();
  const pkgJs = await (await fetch(`crates/multithread/pkg/${name}.js`)).text();

  // Create the 'pure' version of the wasm_bindgen's `init()`
  const initJs = `return () => { const document = undefined; const location = {}; ${pkgJs} return wasm_bindgen; };`;
  const init = (new Function(initJs)).call(null);

  const wbg = init();
  // const wasm = await wbg(`./pkg/${name}_bg.wasm`);
  const wasm = await wbg(`crates/multithread/pkg/${name}_bg.wasm`);
  // console.log('wasm:', wasm);
  wbg.app();

  return wbg;
}
