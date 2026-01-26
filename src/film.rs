use crate::grain::GrainModel;
use crate::physics;
use crate::spectral::{FilmSensitivities, FilmSpectralParams};

/// Film Modeling Module
///
/// Handles Characteristic Curves (H-D Curves) and Color Coupling.
/// Section 3 & 5 of the technical document.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SegmentedCurve {
    pub d_min: f32,
    pub d_max: f32,
    pub gamma: f32,
    pub exposure_offset: f32, // E0 in the doc, controls speed
}

impl SegmentedCurve {
    pub fn new(d_min: f32, d_max: f32, gamma: f32, exposure_offset: f32) -> Self {
        Self {
            d_min,
            d_max,
            gamma,
            exposure_offset,
        }
    }

    /// Maps log10(Exposure) to Density.
    /// Implements a simplified sigmoid-like S-curve based on the segmented model logic
    /// but smoothed for better visual results if exact break points aren't provided.
    pub fn map(&self, log_e: f32) -> f32 {
        self.map_erf(log_e)
    }

    /// Implementation using the Error Function (Erf), which corresponds to the
    /// Gaussian distribution of crystal sensitivities. This is the scientifically
    /// accurate model mentioned in the technical documentation.
    ///
    /// D(E) = D_min + (D_max - D_min) * (1 + erf((log E - log E0) / sigma)) / 2
    pub fn map_erf(&self, log_e: f32) -> f32 {
        let log_e0 = self.exposure_offset.log10();
        let range = self.d_max - self.d_min;

        if range <= 0.0 {
            return self.d_min;
        }

        // Relationship between Gamma and Sigma:
        // Gamma is the slope at the inflection point (log_e = log_e0).
        // D'(log_e) = range * (1/sqrt(pi)) * exp(-z^2) * (1/sigma)
        // At z=0, D' = range / (sigma * sqrt(pi))
        // So Gamma = range / (sigma * sqrt(pi))
        // Sigma = range / (Gamma * sqrt(pi))

        let sqrt_pi = 1.772_453_9;
        let sigma = range / (self.gamma * sqrt_pi);

        let z = (log_e - log_e0) / sigma;

        let val = 0.5 * (1.0 + physics::erf(z));
        self.d_min + range * val
    }

    /// A smoother implementation using interpolation, closer to real film.
    pub fn map_smooth(&self, log_e: f32) -> f32 {
        let log_e0 = self.exposure_offset.log10();
        let x = log_e - log_e0;

        // A sigmoid that goes from D_min to D_max with slope gamma at origin
        // y = D_min + (D_max - D_min) * (1 / (1 + exp(-k * x)))
        // Derivative y' = range * k * sigmoid * (1-sigmoid). At x=0, sigmoid=0.5.
        // y'(0) = range * k * 0.25 = gamma
        // k = 4 * gamma / range

        let range = self.d_max - self.d_min;
        if range <= 0.0 {
            return self.d_min;
        }

        let k = 4.0 * self.gamma / range;

        let sigmoid = 1.0 / (1.0 + (-k * x).exp());
        self.d_min + range * sigmoid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FilmType {
    ColorNegative,
    ColorSlide,
    BwNegative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FilmStock {
    /// Film Type (affects processing pipeline)
    pub film_type: FilmType,

    /// ISO Sensitivity (e.g. 400.0, 50.0).
    /// Used for metadata and reciprocity calculations.
    pub iso: f32,

    /// Response of the Red-sensitive layer (Bottom Layer -> Cyan Dye)
    pub r_curve: SegmentedCurve,
    /// Response of the Green-sensitive layer (Middle Layer -> Magenta Dye)
    pub g_curve: SegmentedCurve,
    /// Response of the Blue-sensitive layer (Top Layer -> Yellow Dye)
    pub b_curve: SegmentedCurve,

    // 3x3 Matrix for crosstalk. Rows: R_out, G_out, B_out. Cols: R_in, G_in, B_in.
    // D_out = Matrix * D_in
    pub color_matrix: [[f32; 3]; 3],

    /// Spectral Sensitivity Parameters.
    /// Used to generate the spectral response curves at runtime.
    pub spectral_params: FilmSpectralParams,

    /// Grain parameters derived from RMS Granularity.
    pub grain_model: GrainModel,

    /// Resolution limit in line pairs per mm (lp/mm).
    /// Used to simulate optical softness before grain.
    pub resolution_lp_mm: f32,

    /// Reciprocity Failure coefficient (beta).
    /// E_effective = E * (1 + beta * log10(t/t0)^2)
    /// Typically 0.03-0.10.
    pub reciprocity_beta: f32,

    /// Halation strength.
    /// Simulates light reflecting off the film base back into the emulsion.
    /// Primarily affects the Red layer (bottom layer) and spreads out (blur).
    pub halation_strength: f32,

    /// Linear light threshold for halation (0.0 to 1.0).
    /// Only highlights above this threshold trigger halation.
    pub halation_threshold: f32,

    /// Blur radius for halation as a fraction of image width (e.g. 0.02).
    /// Controls the spread of the glow.
    pub halation_sigma: f32,

    /// Tint color for the halation glow (RGB).
    /// Usually reddish-orange [1.0, 0.4, 0.2] due to base reflection.
    pub halation_tint: [f32; 3],
}

impl FilmStock {
    /// Create a custom film stock
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        film_type: FilmType,
        iso: f32,
        r_curve: SegmentedCurve,
        g_curve: SegmentedCurve,
        b_curve: SegmentedCurve,
        color_matrix: [[f32; 3]; 3],
        spectral_params: FilmSpectralParams,
        grain_model: GrainModel,
        resolution_lp_mm: f32,
        reciprocity_beta: f32,
        halation_strength: f32,
        halation_threshold: f32,
        halation_sigma: f32,
        halation_tint: [f32; 3],
    ) -> Self {
        Self {
            film_type,
            iso,
            r_curve,
            g_curve,
            b_curve,
            spectral_params,
            color_matrix,
            grain_model,
            resolution_lp_mm,
            reciprocity_beta,
            halation_strength,
            halation_threshold,
            halation_sigma,
            halation_tint,
        }
    }

    /// Generate spectral sensitivities from parameters
    pub fn get_spectral_sensitivities(&self) -> FilmSensitivities {
        FilmSensitivities::from_params(self.spectral_params)
    }

    /// Helper to modify halation strength (common operation)
    pub fn with_halation(mut self, strength: f32) -> Self {
        self.halation_strength = strength;
        self
    }

    /// Save the film stock to a JSON file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Load a film stock from a JSON file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let stock = serde_json::from_reader(reader)?;
        Ok(stock)
    }

    /// Apply the film simulation to RGB log-exposures
    pub fn map_log_exposure(&self, log_e: [f32; 3]) -> [f32; 3] {
        // 1. Map each channel through its curve (Simulates Section 3)
        let d_r = self.r_curve.map_smooth(log_e[0]);
        let d_g = self.g_curve.map_smooth(log_e[1]);
        let d_b = self.b_curve.map_smooth(log_e[2]);

        let net_r = (d_r - self.r_curve.d_min).max(0.0);
        let net_g = (d_g - self.g_curve.d_min).max(0.0);
        let net_b = (d_b - self.b_curve.d_min).max(0.0);

        // 2. Apply Color Matrix (Simulates Section 5 - Layer Coupling)
        // [Dr']   [ M00 M01 M02 ] [ Dr ]
        // [Dg'] = [ M10 M11 M12 ] [ Dg ]
        // [Db']   [ M20 M21 M22 ] [ Db ]

        let d_r_out = self.color_matrix[0][0] * net_r
            + self.color_matrix[0][1] * net_g
            + self.color_matrix[0][2] * net_b;
        let d_g_out = self.color_matrix[1][0] * net_r
            + self.color_matrix[1][1] * net_g
            + self.color_matrix[1][2] * net_b;
        let d_b_out = self.color_matrix[2][0] * net_r
            + self.color_matrix[2][1] * net_g
            + self.color_matrix[2][2] * net_b;

        [
            d_r_out + self.r_curve.d_min,
            d_g_out + self.g_curve.d_min,
            d_b_out + self.b_curve.d_min,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmented_curve_monotonicity() {
        let curve = SegmentedCurve::new(0.1, 2.5, 0.8, 1.0);
        let mut prev_d = curve.map(-5.0); // Very low exposure

        // Test a range of log exposures
        for i in -50..50 {
            let log_e = i as f32 / 10.0;
            let d = curve.map(log_e);

            assert!(
                d >= prev_d,
                "Curve must be monotonic increasing. At log_e={}, d={}, prev_d={}",
                log_e,
                d,
                prev_d
            );
            assert!(
                d >= curve.d_min - 1e-6,
                "Density {} below d_min {}",
                d,
                curve.d_min
            );
            assert!(
                d <= curve.d_max + 1e-6,
                "Density {} above d_max {}",
                d,
                curve.d_max
            );

            prev_d = d;
        }
    }

    #[test]
    fn test_segmented_curve_gamma() {
        // Gamma should be the slope at exposure_offset (log_e0)
        let gamma = 1.5;
        let offset = 10.0;
        let curve = SegmentedCurve::new(0.0, 3.0, gamma, offset);

        let log_e0 = offset.log10();
        let epsilon = 0.001;

        let _d_center = curve.map(log_e0);
        let d_plus = curve.map(log_e0 + epsilon);
        let d_minus = curve.map(log_e0 - epsilon);

        let slope = (d_plus - d_minus) / (2.0 * epsilon);

        // The slope in the ERF model should be close to gamma.
        // Let's check how close.
        let diff = (slope - gamma).abs();
        assert!(
            diff < 0.05,
            "Slope at midpoint {} should be close to gamma {}, diff {}",
            slope,
            gamma,
            diff
        );
    }

    #[test]
    fn test_segmented_curve_limits() {
        let curve = SegmentedCurve::new(0.2, 2.8, 1.0, 1.0);

        // Test asymptotic limits
        let d_low = curve.map(-10.0);
        let d_high = curve.map(10.0);

        assert!(
            (d_low - curve.d_min).abs() < 0.01,
            "Should approach d_min at low exposure"
        );
        assert!(
            (d_high - curve.d_max).abs() < 0.01,
            "Should approach d_max at high exposure"
        );
    }

    #[test]
    fn test_film_stock_creation() {
        let curve = SegmentedCurve::new(0.0, 2.0, 1.0, 1.0);
        let stock = FilmStock::new(
            FilmType::ColorNegative,
            100.0,
            curve,
            curve,
            curve,
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            FilmSpectralParams::new_panchromatic(),
            GrainModel::medium_grain(),
            100.0,
            0.1,
            0.0,
            0.0,
            0.0,
            [0.0, 0.0, 0.0],
        );

        assert_eq!(stock.iso, 100.0);
        assert_eq!(stock.film_type, FilmType::ColorNegative);
    }
}
