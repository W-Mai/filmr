pub mod physics;
pub mod film;
pub mod grain;
pub mod processor;

pub use film::FilmStock;
pub use grain::GrainModel;
pub use processor::{process_image, SimulationConfig, OutputMode};
