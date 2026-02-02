use crate::types::{process_task_image_data, Task, WorkerResult};
use rayon::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn worker_entry() -> Result<(), JsValue> {
    // Initialize logger
    let _ = console_log::init_with_level(log::Level::Debug);

    // Initialize Rayon thread pool
    // In a worker, window() might not be available, use self or navigator directly if possible
    // But web_sys::window() usually returns None in worker.
    // We can use WorkerGlobalScope.

    let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
    // Cast to WorkerGlobalScope to access navigator
    let worker_scope = global
        .clone()
        .unchecked_into::<web_sys::WorkerGlobalScope>();
    let navigator = worker_scope.navigator();
    let concurrency = navigator.hardware_concurrency() as usize;

    log::info!("Worker started. Hardware concurrency: {}", concurrency);

    log::info!("Worker initializing thread pool...");
    wasm_bindgen_futures::JsFuture::from(wasm_bindgen_rayon::init_thread_pool(concurrency)).await?;

    log::info!("Worker thread pool initialized");

    let global_clone = global.clone();

    // Handle messages
    let onmessage = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        let data = event.data();
        log::info!("Worker received message");

        // We expect a batch of tasks (Vec<Task>)
        if let Ok(tasks) = serde_wasm_bindgen::from_value::<Vec<Task>>(data) {
            log::info!("Worker processing {} tasks", tasks.len());
            // Process in parallel
            let results: Vec<WorkerResult> = tasks
                .into_par_iter()
                .map(|t| match t {
                    Task::Process {
                        image_data,
                        width,
                        height,
                        film,
                        config,
                        is_preview,
                    } => {
                        log::info!("Worker task start: {}x{}", width, height);
                        let result = process_task_image_data(
                            image_data, width, height, film, config, is_preview,
                        );
                        match &result {
                            WorkerResult::ProcessDone { .. } => {
                                log::info!("Worker task done");
                            }
                            WorkerResult::Error(_) => {
                                log::error!("Worker failed to create image buffer");
                            }
                        }
                        result
                    }
                })
                .collect();

            // Send back results
            let _ = global_clone.post_message(&serde_wasm_bindgen::to_value(&results).unwrap());
        }
    }) as Box<dyn FnMut(_)>);

    global.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(())
}
