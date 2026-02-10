#[cfg(feature = "compute-gpu")]
use crate::gpu::GpuContext;
#[cfg(feature = "compute-gpu")]
use std::sync::OnceLock;

mod common;
pub mod develop;
pub mod gaussian;
pub mod grain;
pub mod halation;
pub mod light_leak;
pub mod linearize;

pub use develop::DevelopPipeline;
pub use gaussian::GaussianPipeline;
pub use grain::{GrainPipeline, GrainUniforms};
pub use halation::HalationPipeline;
pub use light_leak::LightLeakPipeline;
pub use linearize::LinearizePipeline;

#[cfg(feature = "compute-gpu")]
pub use common::read_gpu_buffer;

#[cfg(all(feature = "compute-gpu", not(target_arch = "wasm32")))]
type PipelineWrapper<T> = T;

#[cfg(all(feature = "compute-gpu", target_arch = "wasm32"))]
type PipelineWrapper<T> = common::SendSyncWrapper<T>;

#[cfg(feature = "compute-gpu")]
static LINEARIZE_PIPELINE: OnceLock<PipelineWrapper<LinearizePipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static LIGHT_LEAK_PIPELINE: OnceLock<PipelineWrapper<LightLeakPipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static HALATION_PIPELINE: OnceLock<PipelineWrapper<HalationPipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static GAUSSIAN_PIPELINE: OnceLock<PipelineWrapper<GaussianPipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static GRAIN_PIPELINE: OnceLock<PipelineWrapper<GrainPipeline>> = OnceLock::new();
#[cfg(feature = "compute-gpu")]
static DEVELOP_PIPELINE: OnceLock<PipelineWrapper<DevelopPipeline>> = OnceLock::new();

#[cfg(feature = "compute-gpu")]
pub fn get_linearize_pipeline(context: &GpuContext) -> &'static LinearizePipeline {
    LINEARIZE_PIPELINE.get_or_init(|| common::wrap_pipeline(LinearizePipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_light_leak_pipeline(context: &GpuContext) -> &'static LightLeakPipeline {
    LIGHT_LEAK_PIPELINE.get_or_init(|| common::wrap_pipeline(LightLeakPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_halation_pipeline(context: &GpuContext) -> &'static HalationPipeline {
    HALATION_PIPELINE.get_or_init(|| common::wrap_pipeline(HalationPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_gaussian_pipeline(context: &GpuContext) -> &'static GaussianPipeline {
    GAUSSIAN_PIPELINE.get_or_init(|| common::wrap_pipeline(GaussianPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_grain_pipeline(context: &GpuContext) -> &'static GrainPipeline {
    GRAIN_PIPELINE.get_or_init(|| common::wrap_pipeline(GrainPipeline::new(context)))
}

#[cfg(feature = "compute-gpu")]
pub fn get_develop_pipeline(context: &GpuContext) -> &'static DevelopPipeline {
    DEVELOP_PIPELINE.get_or_init(|| common::wrap_pipeline(DevelopPipeline::new(context)))
}
