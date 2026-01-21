/// Grain and Noise Simulation Module
///
/// Section 7: Grain Statistics Model.
/// Var(D) = alpha * D + sigma_read^2
use rand::Rng;
use rand_distr::{Distribution, Normal};

#[derive(Debug, Clone, Copy)]
pub struct GrainModel {
    pub alpha: f32,       // Shot noise coefficient (scales with density)
    pub sigma_read: f32,  // Base noise (fog/scanner noise)
}

impl GrainModel {
    pub fn new(alpha: f32, sigma_read: f32) -> Self {
        Self { alpha, sigma_read }
    }

    /// Default parameters for a medium-grained film
    pub fn medium_grain() -> Self {
        Self {
            alpha: 0.05,
            sigma_read: 0.01,
        }
    }

    /// Adds grain noise to a density value D.
    /// Returns the noisy density.
    pub fn add_grain<R: Rng>(&self, d: f32, rng: &mut R) -> f32 {
        // Var(D) = alpha * D + sigma_read^2
        let variance = self.alpha * d + self.sigma_read.powi(2);
        let std_dev = variance.sqrt().max(0.0);

        if std_dev > 0.0 {
            let normal = Normal::new(0.0, std_dev).unwrap();
            let noise = normal.sample(rng);
            (d + noise).max(0.0)
        } else {
            d
        }
    }
}
