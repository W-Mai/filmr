//! Camera shake trajectory generation.
//!
//! Models physiological hand tremor (8-12Hz + harmonics) with slow drift.
//! Generates reproducible trajectories from a seed for UI preview + pipeline sync.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// A camera shake trajectory: sequence of (x, y, weight) displacement samples.
pub struct ShakeTrajectory {
    pub points: Vec<(f32, f32, f32)>,
}

impl ShakeTrajectory {
    /// Generate a trajectory from a seed.
    /// Same seed + amplitude + n_samples → identical trajectory.
    pub fn generate(amplitude_px: f32, n_samples: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let exposure_time = 1.0 / 60.0f32;
        let dt = exposure_time / n_samples as f32;
        let tau = std::f32::consts::TAU;

        let freq_x: f32 = 8.0 + rng.gen::<f32>() * 4.0;
        let freq_y: f32 = 8.0 + rng.gen::<f32>() * 4.0;
        let phase_x: f32 = rng.gen::<f32>() * tau;
        let phase_y: f32 = rng.gen::<f32>() * tau;
        let drift_vx = (rng.gen::<f32>() - 0.5) * amplitude_px * 0.1 / exposure_time;
        let drift_vy = (rng.gen::<f32>() - 0.5) * amplitude_px * 0.1 / exposure_time;
        let ax = amplitude_px * (0.3 + rng.gen::<f32>() * 0.7);
        let ay = amplitude_px * (0.3 + rng.gen::<f32>() * 0.7);
        // Generate N random harmonics per axis
        // Each harmonic: (freq_multiplier, amplitude_ratio, phase)
        let n_harmonics = 8;
        let harmonics_x: Vec<(f32, f32, f32)> = (0..n_harmonics)
            .map(|_| {
                let fm: f32 = 0.2 + rng.gen::<f32>() * 3.0; // 0.2x to 3.2x base freq
                let ar: f32 = rng.gen::<f32>().powi(2); // power-law: most are small
                let ph: f32 = rng.gen::<f32>() * tau;
                (fm, ar, ph)
            })
            .collect();
        let harmonics_y: Vec<(f32, f32, f32)> = (0..n_harmonics)
            .map(|_| {
                let fm: f32 = 0.2 + rng.gen::<f32>() * 3.0;
                let ar: f32 = rng.gen::<f32>().powi(2);
                let ph: f32 = rng.gen::<f32>() * tau;
                (fm, ar, ph)
            })
            .collect();
        // Normalize so total amplitude ratio sums to ~1
        let sum_x: f32 = harmonics_x.iter().map(|h| h.1).sum::<f32>() + 1.0; // +1 for base
        let sum_y: f32 = harmonics_y.iter().map(|h| h.1).sum::<f32>() + 1.0;

        let mut points = Vec::with_capacity(n_samples);

        for i in 0..n_samples {
            let t = i as f32 * dt;
            let mut x = ax / sum_x * (tau * freq_x * t + phase_x).sin();
            for &(fm, ar, ph) in &harmonics_x {
                x += ax * ar / sum_x * (tau * freq_x * fm * t + ph).sin();
            }
            // Random jitter: small per-sample noise (muscle micro-twitches)
            x += ax * 0.15 * (rng.gen::<f32>() - 0.5);
            x += drift_vx * t;

            let mut y = ay / sum_y * (tau * freq_y * t + phase_y).sin();
            for &(fm, ar, ph) in &harmonics_y {
                y += ay * ar / sum_y * (tau * freq_y * fm * t + ph).sin();
            }
            y += ay * 0.15 * (rng.gen::<f32>() - 0.5);
            y += drift_vy * t;

            let frac = i as f32 / n_samples as f32;
            let ramp = 0.1;
            let curtain = if frac < ramp {
                frac / ramp
            } else if frac > 1.0 - ramp {
                (1.0 - frac) / ramp
            } else {
                1.0
            };
            points.push((x, y, curtain));
        }

        // Normalize weights
        let total: f32 = points.iter().map(|p| p.2).sum();
        if total > 0.0 {
            for p in points.iter_mut() {
                p.2 /= total;
            }
        }

        // Center around origin
        let mean_x: f32 = points.iter().map(|p| p.0 * p.2).sum();
        let mean_y: f32 = points.iter().map(|p| p.1 * p.2).sum();
        for p in points.iter_mut() {
            p.0 -= mean_x;
            p.1 -= mean_y;
        }

        Self { points }
    }
}
