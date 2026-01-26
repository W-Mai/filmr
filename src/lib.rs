pub mod film;
pub mod grain;
pub mod physics;
pub mod presets;
pub mod processor;
pub mod spectral;
pub mod metrics;
pub mod pipeline;
pub mod utils;

pub use film::FilmStock;
pub use grain::GrainModel;
pub use processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode,
};
pub use metrics::FilmMetrics;
