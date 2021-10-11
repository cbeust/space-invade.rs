mod utils;

use wasm_bindgen::prelude::*;
use emulator::emulator_state::*;
use emulator::emulator::Emulator;
use emulator::emulator::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, Cedric 2!");
}

#[wasm_bindgen]
pub fn start_emulator() {
    log("Spawning the emulator...");
    emulator::emulator::spawn_emulator();
    log("... done!");
}

#[wasm_bindgen]
pub fn graphic_memory() -> Vec<u8> {
    emulator::emulator::graphic_memory()
}
