use crate::types::{process_task_image_data, process_task_image_data_async, Task, WorkerResult};
use rayon::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(feature = "compute-gpu")]
use filmr::gpu::{init_gpu_context, GpuContext};

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

    #[cfg(feature = "compute-gpu")]
    let gpu_context = GpuContext::new().await;
    #[cfg(not(feature = "compute-gpu"))]
    let gpu_context: Option<()> = None;

    if let Some(ctx) = gpu_context {
        log::info!("Worker initialized with GPU support");
        #[cfg(feature = "compute-gpu")]
        init_gpu_context(ctx);
    } else {
        log::info!("Worker initialized (CPU only)");
    }

    let global_clone = global.clone();

    // Handle messages
    let onmessage = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        let data = event.data();
        let global_clone = global_clone.clone();

        wasm_bindgen_futures::spawn_local(async move {
            log::info!("Worker received message");

            // We expect a batch of tasks (Vec<Task>)
            if let Ok(tasks) = serde_wasm_bindgen::from_value::<Vec<Task>>(data) {
                log::info!("Worker processing {} tasks", tasks.len());

                #[cfg(feature = "compute-gpu")]
                let use_gpu = filmr::gpu::get_gpu_context().is_some();
                #[cfg(not(feature = "compute-gpu"))]
                let use_gpu = false;

                let results: Vec<WorkerResult> = if use_gpu {
                    log::info!("Using GPU processing path");
                    let mut res = Vec::with_capacity(tasks.len());
                    for t in tasks {
                        match t {
                            Task::Process {
                                image_data,
                                width,
                                height,
                                film,
                                config,
                                is_preview,
                            } => {
                                let result = process_task_image_data_async(
                                    image_data, width, height, film, config, is_preview,
                                )
                                .await;
                                res.push(result);
                            }
                        }
                    }
                    res
                } else {
                    log::info!("Using CPU processing path (Rayon)");
                    // Process in parallel
                    tasks
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
                        .collect()
                };

                // Send back results
                let _ = global_clone.post_message(&serde_wasm_bindgen::to_value(&results).unwrap());
            }
        });
    }) as Box<dyn FnMut(_)>);

    global.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(())
}
