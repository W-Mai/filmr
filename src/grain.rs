/// Grain and Noise Simulation Module
///
/// Section 7: Grain Statistics Model.
/// Var(D) = alpha * D + sigma_read^2
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GrainModel {
    pub alpha: f32,       // Shot noise coefficient (scales with density)
    pub sigma_read: f32,  // Base noise (fog/scanner noise)
    pub monochrome: bool, // Whether the grain affects all channels equally (B&W)
    pub blur_radius: f32, // Spatial correlation radius (simulates grain size)
    pub roughness: f32,   // Frequency modulation (0.0 = Smooth, 1.0 = Rough)
}

impl GrainModel {
    pub fn new(
        alpha: f32,
        sigma_read: f32,
        monochrome: bool,
        blur_radius: f32,
        roughness: f32,
    ) -> Self {
        Self {
            alpha,
            sigma_read,
            monochrome,
            blur_radius,
            roughness,
        }
    }

    /// Default parameters for a medium-grained film
    pub fn medium_grain() -> Self {
        Self {
            alpha: 0.05,
            sigma_read: 0.01,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.5,
        }
    }

    /// Generates a noise sample for a given density
    pub fn sample_noise<R: Rng>(&self, d: f32, rng: &mut R) -> f32 {
        // Organic Grain: Use D^1.5 to suppress noise in low-density areas (shadows in positive)
        // and concentrate it in high-density areas (highlights), matching physical silver distribution.

        // Roughness modulation:
        // High roughness increases variance in mid-tones
        // Adjusted Variance = Base_Variance * (1.0 + roughness * sin(pi * d))
        let base_variance = self.alpha * d.powf(1.5) + self.sigma_read.powi(2);

        // Ensure d is reasonable for modulation
        let modulation = 1.0 + self.roughness * (std::f32::consts::PI * d.clamp(0.0, 1.0)).sin();

        let variance = base_variance * modulation;
        let std_dev = variance.sqrt().max(0.0);

        if std_dev > 0.0 {
            let normal = Normal::new(0.0, std_dev).unwrap();
            normal.sample(rng)
        } else {
            0.0
        }
    }

    /// Adds grain noise to a density value D.
    /// Returns the noisy density.
    pub fn add_grain<R: Rng>(&self, d: f32, rng: &mut R) -> f32 {
        let noise = self.sample_noise(d, rng);
        (d + noise).max(0.0)
    }
}
