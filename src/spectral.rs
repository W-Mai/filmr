use std::ops::{Add, Mul};

/// Wavelength range for simulation (Visible Spectrum)
pub const LAMBDA_START: usize = 380;
pub const LAMBDA_END: usize = 780; // Extended to 780nm for IR/Red accuracy
pub const LAMBDA_STEP: usize = 5; // 5nm resolution
pub const BINS: usize = (LAMBDA_END - LAMBDA_START) / LAMBDA_STEP + 1;

#[derive(Debug, Clone, Copy)]
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

    /// Integrate the product of two spectra (inner product)
    /// Used for calculating response: Integral(Light * Sensitivity)
    pub fn integrate_product(&self, other: &Spectrum) -> f32 {
        let mut sum = 0.0;
        for i in 0..BINS {
            sum += self.power[i] * other.power[i];
        }
        // Normalize by step size?
        // Exposure = Power * Time. The units of power are arbitrary here,
        // but physically it's Integral(P(lambda) d_lambda).
        sum * (LAMBDA_STEP as f32)
    }
}

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
        // Area ~= peak * sigma.
        // Using sRGB-ish sigmas: B=25, G=30, R=30.
        // Peak_B * 25 = Peak_G * 30 = Peak_R * 30
        // Set Peak_R = 1.0, Peak_G = 1.0
        // Peak_B = 30/25 * 1.0 = 1.2

        Self {
            r_curve: Spectrum::new_gaussian_with_amplitude(610.0, 30.0, 1.0),
            g_curve: Spectrum::new_gaussian_with_amplitude(540.0, 30.0, 1.0),
            b_curve: Spectrum::new_gaussian_with_amplitude(465.0, 25.0, 1.2), // Peak shifted to 465 to match blue better
        }
    }

    pub fn srgb() -> Self {
        Self::srgb_balanced()
    }

    /// Reconstruct estimated scene spectrum from RGB pixel
    /// L(lambda) = R * S_r + G * S_g + B * S_b
    /// Note: This is a simplification (Principle of Superposition).
    pub fn uplift(&self, r: f32, g: f32, b: f32) -> Spectrum {
        self.r_curve * r + self.g_curve * g + self.b_curve * b
    }
}

/// Spectral sensitivities of the film layers (Red, Green, Blue sensitive)
#[derive(Debug, Clone, Copy)]
pub struct FilmSensitivities {
    pub r_sensitivity: Spectrum, // Cyan forming layer (Red sensitive)
    pub g_sensitivity: Spectrum, // Magenta forming layer (Green sensitive)
    pub b_sensitivity: Spectrum, // Yellow forming layer (Blue sensitive)
    pub r_factor: f32,           // Relative sensitivity factors
    pub g_factor: f32,
    pub b_factor: f32,
}

/// Parameters to generate spectral sensitivities
#[derive(Debug, Clone, Copy)]
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
            r_peak: 650.0,
            r_width: 60.0, // Wide peak for red
            g_peak: 545.0,
            g_width: 50.0, // Shifted and wide
            b_peak: 465.0,
            b_width: 55.0, // Matched to blue
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
                Spectrum::new_gaussian(params.r_peak, params.r_width)
            } else {
                Spectrum::new_zero()
            },
            g_sensitivity: if params.g_peak > 0.0 {
                Spectrum::new_gaussian(params.g_peak, params.g_width)
            } else {
                Spectrum::new_zero()
            },
            b_sensitivity: if params.b_peak > 0.0 {
                Spectrum::new_gaussian(params.b_peak, params.b_width)
            } else {
                Spectrum::new_zero()
            },
            r_factor: 1.0,
            g_factor: 1.0,
            b_factor: 1.0,
        };

        // Auto-balance logic if it looks like a standard panchromatic film
        if params.r_peak > 600.0 && params.b_peak > 400.0 {
            s.r_factor = 1.70; // Boost Red to match Green
            s.g_factor = 1.0;
            s.b_factor = 1.40; // Boost Blue significantly to fix yellow tint
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
}

impl Spectrum {
    pub fn new_zero() -> Self {
        Self { power: [0.0; BINS] }
    }
}
