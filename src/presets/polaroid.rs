//! Polaroid film stock presets

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Polaroid 600 Color (Instant Color Film)
/// Source: Polaroid Technical Data
/// ISO: 600
/// RMS: 15 -> Alpha = 0.0225
/// Gamma: 0.60
/// Dmax: 2.5, Dmin: 0.15
/// Resolution: 80 lp/mm
pub fn POLAROID_600_COLOR() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid 600 Color".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 600.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.5,
            gamma: 0.60,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.5,
            gamma: 0.60,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.5,
            gamma: 0.60,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        color_matrix: [
            [1.02, -0.01, -0.01],
            [-0.01, 1.02, -0.01],
            [-0.01, -0.01, 1.02],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0225,
            sigma_read: 0.008,
            monochrome: false,
            blur_radius: 0.8,
            roughness: 0.7,
            color_correlation: 0.8,
            shadow_noise: 0.002,
            highlight_coarseness: 0.08,
        },
        resolution_lp_mm: 80.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.22,
        halation_threshold: 0.80,
        halation_sigma: 0.018,
        halation_tint: [1.0, 0.75, 0.55],
    }
}

/// Polaroid SX-70 Color (Instant Slide Film)
/// Source: Polaroid Technical Data
/// ISO: 150
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.70
/// Dmax: 2.8, Dmin: 0.15
/// Resolution: 90 lp/mm
pub fn POLAROID_SX70_COLOR() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid SX-70 Color".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 150.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.8,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.13,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.8,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.13,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.8,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.13,
        },
        color_matrix: [
            [1.04, -0.02, -0.02],
            [-0.02, 1.04, -0.02],
            [-0.02, -0.02, 1.04],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0144,
            sigma_read: 0.006,
            monochrome: false,
            blur_radius: 0.8,
            roughness: 0.6,
            color_correlation: 0.8,
            shadow_noise: 0.002,
            highlight_coarseness: 0.08,
        },
        resolution_lp_mm: 90.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.18,
        halation_threshold: 0.82,
        halation_sigma: 0.016,
        halation_tint: [1.0, 0.75, 0.55],
    }
}

/// Polaroid i-Type Color (Modern Instant Color Film)
/// Source: Polaroid Technical Data
/// ISO: 640
/// RMS: 16 -> Alpha = 0.0256
/// Gamma: 0.62
/// Dmax: 2.6, Dmin: 0.15
/// Resolution: 85 lp/mm
pub fn POLAROID_I_TYPE_COLOR() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid i-Type Color".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 640.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.6,
            gamma: 0.62,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.6,
            gamma: 0.62,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.6,
            gamma: 0.62,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        color_matrix: [
            [1.01, -0.01, -0.00],
            [-0.00, 1.01, -0.01],
            [-0.01, -0.00, 1.01],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0256,
            sigma_read: 0.009,
            monochrome: false,
            blur_radius: 0.8,
            roughness: 0.75,
            color_correlation: 0.8,
            shadow_noise: 0.002,
            highlight_coarseness: 0.08,
        },
        resolution_lp_mm: 85.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.24,
        halation_threshold: 0.78,
        halation_sigma: 0.020,
        halation_tint: [1.0, 0.75, 0.55],
    }
}

/// Polaroid B&W 667 (Instant B&W Film)
/// Source: Polaroid Technical Data
/// ISO: 3000
/// RMS: 20 -> Alpha = 0.0400
/// Gamma: 0.55
/// Dmax: 2.4, Dmin: 0.15
/// Resolution: 70 lp/mm
pub fn POLAROID_BW_667() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid B&W 667".to_string(),
        film_type: FilmType::BwNegative,
        iso: 3000.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.4,
            gamma: 0.55,
            shoulder_point: 0.8,
            exposure_offset: 0.00,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.4,
            gamma: 0.55,
            shoulder_point: 0.8,
            exposure_offset: 0.00,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.4,
            gamma: 0.55,
            shoulder_point: 0.8,
            exposure_offset: 0.00,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0400,
            sigma_read: 0.012,
            monochrome: true,
            blur_radius: 1.0,
            roughness: 0.8,
            color_correlation: 0.8,
            shadow_noise: 0.003,
            highlight_coarseness: 0.10,
        },
        resolution_lp_mm: 70.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.25,
        halation_threshold: 0.77,
        halation_sigma: 0.020,
        halation_tint: [0.80, 0.80, 0.80],
    }
}

/// Polaroid Spectra Color (Wide Format Instant Color)
/// Source: Polaroid Technical Data
/// ISO: 640
/// RMS: 16 -> Alpha = 0.0256
/// Gamma: 0.62
/// Dmax: 2.6, Dmin: 0.15
/// Resolution: 85 lp/mm
pub fn POLAROID_SPECTRA_COLOR() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid Spectra Color".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 640.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.6,
            gamma: 0.62,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.6,
            gamma: 0.62,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.6,
            gamma: 0.62,
            shoulder_point: 0.8,
            exposure_offset: 0.02,
        },
        color_matrix: [
            [1.03, -0.02, -0.01],
            [-0.01, 1.03, -0.02],
            [-0.02, -0.01, 1.03],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0256,
            sigma_read: 0.009,
            monochrome: false,
            blur_radius: 0.8,
            roughness: 0.75,
            color_correlation: 0.8,
            shadow_noise: 0.002,
            highlight_coarseness: 0.08,
        },
        resolution_lp_mm: 85.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.24,
        halation_threshold: 0.78,
        halation_sigma: 0.020,
        halation_tint: [1.0, 0.75, 0.55],
    }
}

/// Polaroid 100 Color (Pack Film Color)
/// Source: Polaroid Technical Data
/// ISO: 100
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.15
/// Resolution: 95 lp/mm
pub fn POLAROID_100_COLOR() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid 100 Color".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        color_matrix: [
            [1.05, -0.03, -0.02],
            [-0.02, 1.05, -0.03],
            [-0.03, -0.02, 1.05],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0100,
            sigma_read: 0.005,
            monochrome: false,
            blur_radius: 0.7,
            roughness: 0.5,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.06,
        },
        resolution_lp_mm: 95.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [1.0, 0.75, 0.55],
    }
}

/// Polaroid 55 B&W (Pack Film B&W)
/// Source: Polaroid Technical Data
/// ISO: 50
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.70
/// Dmax: 2.3, Dmin: 0.15
/// Resolution: 100 lp/mm
pub fn POLAROID_55_BW() -> FilmStock {
    FilmStock {
        manufacturer: "Polaroid".to_string(),
        name: "Polaroid 55 B&W".to_string(),
        film_type: FilmType::BwNegative,
        iso: 50.0,
        r_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.3,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        g_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.3,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        b_curve: SegmentedCurve {
            d_min: 0.15,
            d_max: 2.3,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0064,
            sigma_read: 0.004,
            monochrome: true,
            blur_radius: 0.7,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.06,
        },
        resolution_lp_mm: 100.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.14,
        halation_threshold: 0.86,
        halation_sigma: 0.014,
        halation_tint: [0.88, 0.88, 0.88],
    }
}

/// Get all Polaroid film stocks
pub fn get_stocks() -> Vec<FilmStock> {
    vec![
        POLAROID_600_COLOR(),
        POLAROID_SX70_COLOR(),
        POLAROID_I_TYPE_COLOR(),
        POLAROID_BW_667(),
        POLAROID_SPECTRA_COLOR(),
        POLAROID_100_COLOR(),
        POLAROID_55_BW(),
    ]
}
