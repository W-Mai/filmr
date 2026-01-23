pub mod film;
pub mod grain;
pub mod physics;
pub mod presets;
pub mod processor;
pub mod spectral;

pub use film::FilmStock;
pub use grain::GrainModel;
pub use processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode,
};
