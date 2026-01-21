/// Physics module for Film Simulation
/// 
/// Handles basic physical quantities and conversions described in the technical document.
/// Section 2: Exposure and Density Mapping.

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
pub fn density_to_transmission(density: f32) -> f32 {
    10.0f32.powf(-density)
}

/// Helper to convert sRGB (gamma encoded) to Linear Light (approximate).
/// This is needed to get "Irradiance" from a digital image pixel.
/// Assuming sRGB gamma ~2.2 for simplicity or standard transfer function.
pub fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Helper to convert Linear Light to sRGB.
pub fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
