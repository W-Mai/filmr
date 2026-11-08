import init, { worker_entry } from './filmr_app.js';

console.log("JS Worker wrapper started");

async function run() {
    console.log("JS Worker wrapper initializing WASM...");
    try {
        await init();
        console.log("JS Worker wrapper WASM initialized, starting entry...");
        await worker_entry();
        console.log("JS Worker wrapper entry finished (should not happen for worker listener)");
    } catch (e) {
        console.error("JS Worker wrapper failed:", e);
    }
}

run();
