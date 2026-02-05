/// Grain and Noise Simulation Module
///
/// Section 7: Grain Statistics Model.
/// Var(D) = alpha * D + sigma_read^2
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GrainModel {
    pub alpha: f32,                // Shot noise coefficient (scales with density)
    pub sigma_read: f32,           // Base noise (fog/scanner noise)
    pub monochrome: bool,          // Whether the grain affects all channels equally (B&W)
    pub blur_radius: f32,          // Spatial correlation radius (simulates grain size)
    pub roughness: f32,            // Frequency modulation (0.0 = Smooth, 1.0 = Rough)
    pub color_correlation: f32, // How strongly the RGB channels are correlated (0.0 = Independent, 1.0 = Monochrome)
    pub shadow_noise: f32,      // Photon shot noise strength (Poisson noise in shadows)
    pub highlight_coarseness: f32, // Factor to increase grain size (clumping) in highlights
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
            color_correlation: 0.8, // Default high correlation for natural look
            shadow_noise: 0.001,    // Default small amount of shot noise
            highlight_coarseness: 0.10, // Moderate highlight clumping
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
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.10,
        }
    }

    /// Generates a noise sample for a given density
    pub fn sample_noise<R: Rng>(&self, d: f32, rng: &mut R) -> f32 {
        // Organic Grain: Use D^1.5 to suppress noise in low-density areas (shadows in positive)
        // and concentrate it in high-density areas (highlights), matching physical silver distribution.

        // Photon Shot Noise (Shadows): Decays exponentially with density.
        // Physics: Delta_D ~ 1/sqrt(E) ~ 1/sqrt(10^D) ~ 10^(-0.5*D) ~ exp(-1.15*D)
        // We use exp(-2.0 * D) to ensure it vanishes quickly in mid-tones, keeping shadows clean but textured.
        let shot_variance = if self.shadow_noise > 0.0 {
            self.shadow_noise * (-2.0 * d.max(0.0)).exp()
        } else {
            0.0
        };

        // Roughness modulation:
        // High roughness increases variance in mid-tones
        // Adjusted Variance = Base_Variance * (1.0 + roughness * sin(pi * d))
        let grain_variance = self.alpha * d.max(0.0).powf(1.5);
        let base_variance = grain_variance + self.sigma_read.powi(2) + shot_variance;

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
