#[cfg(feature = "compute-gpu")]
use tracing::info;

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
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

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
pub fn get_gpu_context() -> Option<&'static GpuContext> {
    None
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
