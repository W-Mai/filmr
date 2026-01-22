pub mod film;
pub mod grain;
pub mod physics;
pub mod presets;
pub mod processor;
pub mod spectral;

pub use film::FilmStock;
pub use grain::GrainModel;
pub use processor::{process_image, OutputMode, SimulationConfig, WhiteBalanceMode};
