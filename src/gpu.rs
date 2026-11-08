#[cfg(feature = "compute-gpu")]
use tracing::info;
#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
use tracing::warn;

#[cfg(feature = "compute-gpu")]
use std::sync::OnceLock;

#[cfg(feature = "compute-gpu")]
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[cfg(feature = "compute-gpu")]
pub struct GpuBuffer {
    pub buffer: wgpu::Buffer,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
static GPU_CONTEXT: OnceLock<Option<GpuContext>> = OnceLock::new();

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
pub fn get_gpu_context() -> Option<&'static GpuContext> {
    // Initialize lazily, caching both success and failure so we don't retry
    // (and re-panic) on every frame. If GPU init panics inside a driver
    // backend, fall back to CPU instead of taking down the worker thread.
    GPU_CONTEXT
        .get_or_init(|| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| block_on(GpuContext::new())))
                .unwrap_or_else(|_| {
                    warn!("GPU initialization panicked; falling back to CPU");
                    None
                })
        })
        .as_ref()
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
        // On native, avoid the GLES/EGL backend: wgpu may try to create an
        // EGL context on a worker thread, where `eglMakeCurrent` fails and
        // khronos-egl panics. Vulkan/Metal/DX12 are thread-safe. On WASM we
        // still need the browser/GL backends.
        let backends = if cfg!(target_arch = "wasm32") {
            wgpu::Backends::all()
        } else {
            wgpu::Backends::PRIMARY
        };
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
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
