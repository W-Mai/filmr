use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};
use wide::f32x4;

/// Wavelength range for simulation (Visible Spectrum)
pub const LAMBDA_START: usize = 380;
pub const LAMBDA_END: usize = 780; // Extended to 780nm for IR/Red accuracy
pub const LAMBDA_STEP: usize = 5; // 5nm resolution
pub const BINS: usize = (LAMBDA_END - LAMBDA_START) / LAMBDA_STEP + 1;

#[derive(Debug, Clone)]
pub struct Spectrum {
    pub power: [f32; BINS],
}

impl Default for Spectrum {
    fn default() -> Self {
        Self::new()
    }
}

impl Spectrum {
    pub fn new() -> Self {
        Self { power: [0.0; BINS] }
    }

    pub fn new_flat(value: f32) -> Self {
        Self {
            power: [value; BINS],
        }
    }

    pub fn new_gaussian(peak_nm: f32, sigma_nm: f32) -> Self {
        Self::new_gaussian_with_amplitude(peak_nm, sigma_nm, 1.0)
    }

    pub fn new_gaussian_with_amplitude(peak_nm: f32, sigma_nm: f32, amplitude: f32) -> Self {
        let mut s = Self::new();
        for i in 0..BINS {
            let lambda = (LAMBDA_START + i * LAMBDA_STEP) as f32;
            // Gaussian: exp(-0.5 * ((x - mu) / sigma)^2)
            let diff = (lambda - peak_nm) / sigma_nm;
            s.power[i] = amplitude * (-0.5 * diff * diff).exp();
        }
        s
    }

    /// Create a normalized Gaussian spectrum (integral = 1.0).
    /// This ensures energy conservation regardless of spectral width.
    pub fn new_gaussian_normalized(peak_nm: f32, sigma_nm: f32) -> Self {
        let mut s = Self::new();
        for i in 0..BINS {
            let lambda = (LAMBDA_START + i * LAMBDA_STEP) as f32;
            let diff = (lambda - peak_nm) / sigma_nm;
            s.power[i] = (-0.5 * diff * diff).exp();
        }

        // Normalize area
        let area = s.integrate_product(&Spectrum::new_flat(1.0));
        if area > 0.0 {
            let factor = 1.0 / area;
            for v in &mut s.power {
                *v *= factor;
            }
        }
        s
    }

    /// Create a Blackbody spectrum using Planck's Law.
    /// Note: This returns Relative Spectral Power Distribution (RSPD), not absolute radiance.
    pub fn new_blackbody(temperature: f32) -> Self {
        let mut s = Self::new();
        let c1 = 3.741771e-16f32; // 2*h*c^2
        let c2 = 1.4388e-2f32; // h*c/k

        for i in 0..BINS {
            let lambda_nm = (LAMBDA_START + i * LAMBDA_STEP) as f32;
            let lambda_m = lambda_nm * 1.0e-9;

            let x = c2 / (lambda_m * temperature);
            // Prevent overflow for small lambda * T
            let exp_x = if x < 50.0 { x.exp() } else { f32::INFINITY };

            let denom = lambda_m.powi(5) * (exp_x - 1.0);

            if denom.is_finite() && denom > 0.0 {
                s.power[i] = c1 / denom;
            } else {
                s.power[i] = 0.0;
            }
        }
        s.normalize_max();
        s
    }

    pub fn new_d65() -> Self {
        Self::new_blackbody(6504.0)
    }

    pub fn normalize_max(&mut self) {
        let mut max_val = 0.0f32;
        for i in 0..BINS {
            max_val = max_val.max(self.power[i]);
        }
        if max_val > 0.0 {
            for i in 0..BINS {
                self.power[i] /= max_val;
            }
        }
    }

    pub fn multiply(&self, other: &Spectrum) -> Self {
        let mut s = Self::new();
        let mut i = 0;

        // Process 4 elements at a time using SIMD
        while i + 4 <= BINS {
            // Use explicit array construction to avoid slice-to-array copy overhead/panic
            // Compiler should optimize this to a vector load
            let a = f32x4::from([
                self.power[i],
                self.power[i + 1],
                self.power[i + 2],
                self.power[i + 3],
            ]);
            let b = f32x4::from([
                other.power[i],
                other.power[i + 1],
                other.power[i + 2],
                other.power[i + 3],
            ]);
            let res = a * b;

            let res_arr: [f32; 4] = res.into();
            s.power[i..i + 4].copy_from_slice(&res_arr);
            i += 4;
        }

        // Handle remainder
        while i < BINS {
            s.power[i] = self.power[i] * other.power[i];
            i += 1;
        }
        s
    }

    /// Integrate the product of two spectra (inner product)
    /// Used for calculating response: Integral(Light * Sensitivity)
    pub fn integrate_product(&self, other: &Spectrum) -> f32 {
        // Optimization: Trapezoidal rule
        // Sum middle points (weight 1.0) then add endpoints (weight 0.5)

        let mut sum_simd = f32x4::splat(0.0);
        let mut i = 1; // Start from 1 to skip first element
        let limit = BINS - 1; // Stop before last element

        while i + 4 <= limit {
            let a = f32x4::from([
                self.power[i],
                self.power[i + 1],
                self.power[i + 2],
                self.power[i + 3],
            ]);
            let b = f32x4::from([
                other.power[i],
                other.power[i + 1],
                other.power[i + 2],
                other.power[i + 3],
            ]);
            sum_simd += a * b;
            i += 4;
        }

        let mut sum = sum_simd.reduce_add();

        // Handle remainder of the middle section
        while i < limit {
            sum += self.power[i] * other.power[i];
            i += 1;
        }

        // Add endpoints with 0.5 weight
        let first = self.power[0] * other.power[0];
        let last = self.power[BINS - 1] * other.power[BINS - 1];

        (sum + 0.5 * (first + last)) * (LAMBDA_STEP as f32)
    }
}

/// NOTE:
/// Add means linear superposition of radiance.
impl Add for Spectrum {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut res = Self::new();
        for i in 0..BINS {
            res.power[i] = self.power[i] + rhs.power[i];
        }
        res
    }
}

/// Support reference addition to avoid moves/clones
impl Add<&Spectrum> for &Spectrum {
    type Output = Spectrum;
    fn add(self, rhs: &Spectrum) -> Spectrum {
        let mut res = Spectrum::new();
        for i in 0..BINS {
            res.power[i] = self.power[i] + rhs.power[i];
        }
        res
    }
}

/// NOTE:
/// Mul means scalar exposure / gain.
impl Mul<f32> for Spectrum {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        let mut res = Self::new();
        for i in 0..BINS {
            res.power[i] = self.power[i] * rhs;
        }
        res
    }
}

/// Support reference multiplication to avoid moves/clones
impl Mul<f32> for &Spectrum {
    type Output = Spectrum;
    fn mul(self, rhs: f32) -> Spectrum {
        let mut res = Spectrum::new();
        for i in 0..BINS {
            res.power[i] = self.power[i] * rhs;
        }
        res
    }
}

/// Approximate sRGB Camera Sensitivities (Standard Observer-ish)
/// Used to reconstruct the incident light spectrum from RGB values.
pub struct CameraSensitivities {
    pub r_curve: Spectrum,
    pub g_curve: Spectrum,
    pub b_curve: Spectrum,
}

impl CameraSensitivities {
    pub fn srgb_balanced() -> Self {
        // Approximate sRGB / Rec.709 primaries peaks
        // Blue: ~450nm, Green: ~540nm, Red: ~610nm
        //
        // Target: Equal Area under curve for white balance.
        // Using normalized Gaussians ensures exactly equal area (energy).

        Self {
            r_curve: Spectrum::new_gaussian_normalized(610.0, 30.0),
            g_curve: Spectrum::new_gaussian_normalized(540.0, 30.0),
            b_curve: Spectrum::new_gaussian_normalized(465.0, 30.0), // Peak shifted to 465
        }
    }

    pub fn srgb() -> Self {
        Self::srgb_balanced()
    }

    /// Reconstruct estimated scene spectrum from RGB pixel
    /// L(lambda) = R * S_r + G * S_g + B * S_b
    /// Note: This is a simplification (Principle of Superposition).
    pub fn uplift(&self, r: f32, g: f32, b: f32) -> Spectrum {
        &self.r_curve * r + &self.g_curve * g + &self.b_curve * b
    }
}

/// Spectral sensitivities of the film layers (Red, Green, Blue sensitive)
#[derive(Debug, Clone)]
pub struct FilmSensitivities {
    pub r_sensitivity: Spectrum, // Cyan forming layer (Red sensitive)
    pub g_sensitivity: Spectrum, // Magenta forming layer (Green sensitive)
    pub b_sensitivity: Spectrum, // Yellow forming layer (Blue sensitive)
    pub r_factor: f32,           // Relative sensitivity factors
    pub g_factor: f32,
    pub b_factor: f32,
}

/// Parameters to generate spectral sensitivities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FilmSpectralParams {
    pub r_peak: f32,
    pub r_width: f32,
    pub g_peak: f32,
    pub g_width: f32,
    pub b_peak: f32,
    pub b_width: f32,
}

impl FilmSpectralParams {
    /// Create standard panchromatic response
    pub const fn new_panchromatic() -> Self {
        Self {
            r_peak: 630.0, // Closer to 610 to pick up Red efficiency
            r_width: 20.0, // Very narrow to avoid crosstalk
            g_peak: 540.0, // Centered on Green
            g_width: 20.0, // Narrow
            b_peak: 460.0, // Centered on Blue
            b_width: 20.0, // Narrow
        }
    }

    /// Create standard Color Negative response (Broad, overlapping)
    /// Modeled after Kodak Portra / Vision3
    pub const fn new_color_negative_standard() -> Self {
        Self {
            r_peak: 640.0, // Deep Red
            r_width: 45.0, // Broad
            g_peak: 550.0, // Green
            g_width: 40.0, // Broad, overlaps with Blue at 480-500
            b_peak: 450.0, // Blue
            b_width: 45.0, // Broad
        }
    }

    /// Create orthochromatic response (insensitive to red)
    pub const fn new_orthochromatic() -> Self {
        Self {
            r_peak: 0.0,
            r_width: 0.0, // Special case 0 = no sensitivity
            g_peak: 540.0,
            g_width: 40.0,
            b_peak: 440.0,
            b_width: 40.0,
        }
    }

    /// Create infrared response (extended red)
    pub const fn new_infrared() -> Self {
        Self {
            r_peak: 720.0,
            r_width: 60.0,
            g_peak: 540.0,
            g_width: 40.0,
            b_peak: 440.0,
            b_width: 40.0,
        }
    }
}

impl FilmSensitivities {
    pub fn from_params(params: FilmSpectralParams) -> Self {
        // Standard Panchromatic Balance defaults
        // These can be overridden if we had them in params, but for now hardcode
        // the balancing logic for the common case.
        // We assume most films using "panchromatic" params want neutral balance.

        let mut s = Self {
            r_sensitivity: if params.r_peak > 0.0 {
                Spectrum::new_gaussian_normalized(params.r_peak, params.r_width)
            } else {
                Spectrum::new()
            },
            g_sensitivity: if params.g_peak > 0.0 {
                Spectrum::new_gaussian_normalized(params.g_peak, params.g_width)
            } else {
                Spectrum::new()
            },
            b_sensitivity: if params.b_peak > 0.0 {
                Spectrum::new_gaussian_normalized(params.b_peak, params.b_width)
            } else {
                Spectrum::new()
            },
            r_factor: 1.0,
            g_factor: 1.0,
            b_factor: 1.0,
        };

        if params.r_peak > 0.0 || params.g_peak > 0.0 || params.b_peak > 0.0 {
            let reference = Spectrum::new_d65();
            let r_resp = s.r_sensitivity.integrate_product(&reference);
            let g_resp = s.g_sensitivity.integrate_product(&reference);
            let b_resp = s.b_sensitivity.integrate_product(&reference);
            let epsilon = 1e-6;
            s.r_factor = 1.0 / r_resp.max(epsilon);
            s.g_factor = 1.0 / g_resp.max(epsilon);
            s.b_factor = 1.0 / b_resp.max(epsilon);
        }

        s
    }

    /// Calculate exposure for the three layers given an incident light spectrum
    pub fn expose(&self, light: &Spectrum) -> [f32; 3] {
        [
            self.r_sensitivity.integrate_product(light) * self.r_factor,
            self.g_sensitivity.integrate_product(light) * self.g_factor,
            self.b_sensitivity.integrate_product(light) * self.b_factor,
        ]
    }

    /// Calibrate sensitivity factors to a specific white point spectrum.
    /// Ensures that exposing this spectrum results in [1.0, 1.0, 1.0].
    pub fn calibrate_to_white_point(&mut self, white_point: &Spectrum) {
        let r_resp = self.r_sensitivity.integrate_product(white_point);
        let g_resp = self.g_sensitivity.integrate_product(white_point);
        let b_resp = self.b_sensitivity.integrate_product(white_point);
        let epsilon = 1e-6;
        self.r_factor = 1.0 / r_resp.max(epsilon);
        self.g_factor = 1.0 / g_resp.max(epsilon);
        self.b_factor = 1.0 / b_resp.max(epsilon);
    }
}
