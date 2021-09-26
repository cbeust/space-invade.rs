import * as wasm from "wasm-space-invaders";

wasm.start_emulator();
var memory = wasm.graphic_memory();

console.log("memory: " + memory[0]);

// wasm.greet();



