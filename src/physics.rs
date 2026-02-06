//! Physics module for Film Simulation
//!
//! Handles basic physical quantities and conversions described in the documentation.
//! Section 2: Exposure and Density Mapping.

/// Transmission at zero density: T = 10^(-0) = 1.0
pub const TRANSMISSION_AT_ZERO_DENSITY: f32 = 1.0;

/// Calculates Exposure (E) from Irradiance (I) and Time (t).
/// E = I * t
/// Unit: lux·s
pub fn calculate_exposure(irradiance: f32, time: f32) -> f32 {
    irradiance * time
}

/// Calculates Optical Density (D) from Transmission (T).
/// D = -log10(T)
/// T must be in range (0.0, 1.0]
pub fn transmission_to_density(transmission: f32) -> f32 {
    if transmission <= 0.0 {
        // Handle effectively 0 transmission (infinite density) with a high cap
        return 5.0;
    }
    -transmission.log10()
}

/// Calculates Transmission (T) from Optical Density (D).
/// T = 10^(-D)
#[inline]
pub fn density_to_transmission(density: f32) -> f32 {
    10.0f32.powf(-density)
}

/// Helper to convert sRGB (gamma encoded) to Linear Light (approximate).
/// This is needed to get "Irradiance" from a digital image pixel.
/// Assuming sRGB gamma ~2.2 for simplicity or standard transfer function.
#[inline]
pub fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Helper to convert Linear Light to sRGB.
#[inline]
pub fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Error function approximation (Abramowitz and Stegun 7.1.26)
/// Maximum error: 1.5e-7
#[inline]
pub fn erf(x: f32) -> f32 {
    let a1 = 0.254_829_6;
    let a2 = -0.284_496_72;
    let a3 = 1.421_413_8;
    let a4 = -1.453_152_1;
    let a5 = 1.061_405_4;
    let p = 0.3275911;

    // Save the sign of x
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    // A&S formula 7.1.26
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Shoulder softening model based on space charge limit
/// Smoothly compresses densities above a certain threshold to simulate highlight roll-off.
#[inline]
pub fn shoulder_softening(density: f32, shoulder_point: f32) -> f32 {
    if density > shoulder_point {
        let excess = density - shoulder_point;
        // Formula: D_soft = D - (D-D_s)^2 / (D_s + (D-D_s))
        // This approximates the physical saturation of silver halide crystals.
        density - (excess * excess) / (shoulder_point + excess)
    } else {
        density
    }
}

/// Dye self-absorption correction
/// At high densities, Beer's Law deviates slightly.
#[inline]
pub fn apply_dye_self_absorption(density: f32, transmission: f32) -> f32 {
    if density > 1.5 {
        let correction = 1.0 + (density - 1.5) * 0.02;
        transmission * correction.clamp(0.97, 1.03)
    } else {
        transmission
    }
}
