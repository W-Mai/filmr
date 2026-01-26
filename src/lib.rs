pub mod film;
pub mod grain;
pub mod light_leak;
pub mod metrics;
pub mod physics;
pub mod pipeline;
pub mod presets;
pub mod processor;
pub mod spectral;
pub mod utils;

pub use film::FilmStock;
pub use grain::GrainModel;
pub use metrics::FilmMetrics;
pub use processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode,
};
pub use spectral::Spectrum;
