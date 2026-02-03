#[cfg(feature = "compute-gpu")]
use tracing::info;

#[cfg(feature = "compute-gpu")]
use std::sync::OnceLock;

#[cfg(feature = "compute-gpu")]
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
static GPU_CONTEXT: OnceLock<GpuContext> = OnceLock::new();

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
pub fn get_gpu_context() -> Option<&'static GpuContext> {
    if let Some(ctx) = GPU_CONTEXT.get() {
        return Some(ctx);
    }

    // Initialize if not present
    let ctx = block_on(GpuContext::new())?;
    let _ = GPU_CONTEXT.set(ctx);
    GPU_CONTEXT.get()
}

// WASM Implementation using Unsafe Wrapper to bypass Send/Sync check
// Safety: This is only safe if accessed from the single worker thread where it was initialized.
#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
struct WasmGpuHolder(GpuContext);

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
unsafe impl Send for WasmGpuHolder {}

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
unsafe impl Sync for WasmGpuHolder {}

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
static WASM_GPU_CONTEXT: OnceLock<WasmGpuHolder> = OnceLock::new();

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
pub fn init_gpu_context(context: GpuContext) {
    let _ = WASM_GPU_CONTEXT.set(WasmGpuHolder(context));
}

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
pub fn get_gpu_context() -> Option<&'static GpuContext> {
    WASM_GPU_CONTEXT.get().map(|h| &h.0)
}

#[cfg(feature = "compute-gpu")]
impl GpuContext {
    pub async fn new() -> Option<Self> {
        info!("Initializing WGPU context");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok()?;

        info!("Using GPU adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Filmr Compute Device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: Default::default(),
            })
            .await
            .ok()?;

        Some(Self { device, queue })
    }
}

/// Helper to run future synchronously on current thread (if possible)
#[cfg(feature = "compute-gpu")]
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    pollster::block_on(future)
}
