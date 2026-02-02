use crate::types::{Task, WorkerResult};
use flume::{Receiver, Sender};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, Worker};

#[derive(Clone)]
pub struct ComputeBridge {
    _worker: Worker,
    task_sender: Sender<Task>,
    result_receiver: Receiver<WorkerResult>,
}

impl ComputeBridge {
    pub fn new() -> Self {
        // Create worker.
        // Note: The path "./worker.js" must match where Trunk puts the worker script.
        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        log::info!("Creating Compute Worker (ComputeBridge::new)...");
        let worker =
            Worker::new_with_options("./worker.js", &options).expect("Failed to create worker");

        let (task_tx, task_rx) = flume::unbounded::<Task>();
        let (result_tx, result_rx) = flume::unbounded::<WorkerResult>();

        // Setup worker response handler
        let result_tx_clone = result_tx.clone();
        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();
            if let Ok(results) = serde_wasm_bindgen::from_value::<Vec<WorkerResult>>(data) {
                log::info!("Bridge received {} results", results.len());
                for res in results {
                    let _ = result_tx_clone.send(res);
                }
            } else {
                log::error!("Bridge failed to deserialize results");
            }
        }) as Box<dyn FnMut(_)>);
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // Loop to send tasks to worker
        let worker_clone = worker.clone();
        wasm_bindgen_futures::spawn_local(async move {
            log::info!("Bridge task sender loop started");
            while let Ok(task) = task_rx.recv_async().await {
                log::info!("Bridge received task from channel, preparing batch...");
                let mut batch = vec![task];
                while let Ok(t) = task_rx.try_recv() {
                    batch.push(t);
                    if batch.len() >= 8 {
                        break;
                    }
                }
                log::info!("Bridge sending batch of {} tasks to worker", batch.len());
                if let Ok(val) = serde_wasm_bindgen::to_value(&batch) {
                    let _ = worker_clone.post_message(&val);
                } else {
                    log::error!("Bridge failed to serialize task batch");
                }
            }
            log::info!("Bridge task sender loop ended");
        });

        Self {
            _worker: worker,
            task_sender: task_tx,
            result_receiver: result_rx,
        }
    }

    pub fn submit_task(&self, task: Task) {
        log::info!("Bridge submit_task called");
        let _ = self.task_sender.send(task);
    }

    pub fn result_receiver(&self) -> Receiver<WorkerResult> {
        self.result_receiver.clone()
    }

    pub fn try_recv(&self) -> Option<WorkerResult> {
        self.result_receiver.try_recv().ok()
    }
}

impl Default for ComputeBridge {
    fn default() -> Self {
        Self::new()
    }
}
