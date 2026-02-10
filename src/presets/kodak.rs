//! Kodak film stock presets

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Kodak Portra 400 (Professional Color Negative)
/// Source: Kodak Technical Data
/// ISO: 400
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 0.65
/// Dmax: 2.8, Dmin: 0.12
/// Resolution: 125 lp/mm
pub fn KODAK_PORTRA_400() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Portra 400".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.8,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.8,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.8,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        color_matrix: [
            [1.07, -0.04, -0.03],
            [-0.03, 1.07, -0.04],
            [-0.04, -0.03, 1.07],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0121,
            sigma_read: 0.005,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.45,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 125.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [1.0, 0.70, 0.50],
    }
}

/// Kodak Portra 160 (Fine Grain Color Negative)
/// Source: Kodak Technical Data
/// ISO: 160
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.12
/// Resolution: 140 lp/mm
pub fn KODAK_PORTRA_160() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Portra 160".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 160.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.13,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.13,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.13,
        },
        color_matrix: [
            [1.09, -0.05, -0.04],
            [-0.04, 1.09, -0.05],
            [-0.05, -0.04, 1.09],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0081,
            sigma_read: 0.004,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.35,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 140.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.13,
        halation_threshold: 0.87,
        halation_sigma: 0.013,
        halation_tint: [1.0, 0.70, 0.50],
    }
}

/// Kodak Portra 800 (High Speed Color Negative)
/// Source: Kodak Technical Data
/// ISO: 800
/// RMS: 13 -> Alpha = 0.0169
/// Gamma: 0.65
/// Dmax: 2.9, Dmin: 0.12
/// Resolution: 110 lp/mm
pub fn KODAK_PORTRA_800() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Portra 800".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 800.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.9,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.03,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.9,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.03,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.9,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.03,
        },
        color_matrix: [
            [1.05, -0.03, -0.02],
            [-0.02, 1.05, -0.03],
            [-0.03, -0.02, 1.05],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0169,
            sigma_read: 0.007,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.55,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 110.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.17,
        halation_threshold: 0.83,
        halation_sigma: 0.015,
        halation_tint: [1.0, 0.70, 0.50],
    }
}

/// Kodak Tri-X 400 (Professional B&W)
/// Source: Kodak Technical Data
/// ISO: 400
/// RMS: 14 -> Alpha = 0.0196
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 115 lp/mm
pub fn KODAK_TRI_X_400() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Tri-X 400".to_string(),
        film_type: FilmType::BwNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0196,
            sigma_read: 0.007,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.6,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 115.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.20,
        halation_threshold: 0.82,
        halation_sigma: 0.016,
        halation_tint: [0.85, 0.85, 0.85],
    }
}

/// Kodak Plus-X 125 (Fine Grain B&W)
/// Source: Kodak Technical Data
/// ISO: 125
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 0.75
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 140 lp/mm
pub fn KODAK_PLUS_X_125() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Plus-X 125".to_string(),
        film_type: FilmType::BwNegative,
        iso: 125.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.16,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.16,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.16,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0081,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 140.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [0.92, 0.92, 0.92],
    }
}

/// Kodak Ektachrome 100 (Professional Slide Film)
/// Source: Kodak Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.3
/// Dmax: 3.5, Dmin: 0.12
/// Resolution: 150 lp/mm
pub fn KODAK_EKTACHROME_100() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Ektachrome 100".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.5,
            gamma: 1.3,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.5,
            gamma: 1.3,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.5,
            gamma: 1.3,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        color_matrix: [
            [1.13, -0.07, -0.06],
            [-0.06, 1.13, -0.07],
            [-0.07, -0.06, 1.13],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0064,
            sigma_read: 0.004,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.3,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 150.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.10,
        halation_threshold: 0.90,
        halation_sigma: 0.010,
        halation_tint: [0.95, 0.95, 0.95],
    }
}

/// Kodak Ektachrome 100 VS (Vivid Saturation Slide Film)
/// Source: Kodak Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.35
/// Dmax: 3.5, Dmin: 0.12
/// Resolution: 150 lp/mm
pub fn KODAK_EKTACHROME_100VS() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Ektachrome 100 VS".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.5,
            gamma: 1.35,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.5,
            gamma: 1.35,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.5,
            gamma: 1.35,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        color_matrix: [
            [1.18, -0.09, -0.09],
            [-0.09, 1.18, -0.09],
            [-0.09, -0.09, 1.18],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0064,
            sigma_read: 0.004,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.3,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 150.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.10,
        halation_threshold: 0.90,
        halation_sigma: 0.010,
        halation_tint: [0.95, 0.95, 0.95],
    }
}

/// Kodak Kodachrome 64 (Classic Slide Film)
/// Source: Kodak Technical Data
/// ISO: 64
/// RMS: 7 -> Alpha = 0.0049
/// Gamma: 1.4
/// Dmax: 3.6, Dmin: 0.10
/// Resolution: 160 lp/mm
pub fn KODAK_KODACHROME_64() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Kodachrome 64".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 64.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.6,
            gamma: 1.4,
            shoulder_point: 0.8,
            exposure_offset: 0.31,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.6,
            gamma: 1.4,
            shoulder_point: 0.8,
            exposure_offset: 0.31,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.6,
            gamma: 1.4,
            shoulder_point: 0.8,
            exposure_offset: 0.31,
        },
        color_matrix: [
            [1.25, -0.13, -0.12],
            [-0.12, 1.25, -0.13],
            [-0.13, -0.12, 1.25],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0049,
            sigma_read: 0.003,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.2,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 160.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.08,
        halation_threshold: 0.92,
        halation_sigma: 0.008,
        halation_tint: [1.0, 0.35, 0.35],
    }
}

/// Kodak Gold 200 (Consumer Color Negative)
/// Source: Kodak Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.12
/// Resolution: 130 lp/mm
pub fn KODAK_GOLD_200() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Gold 200".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 200.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.7,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        color_matrix: [
            [1.06, -0.03, -0.03],
            [-0.03, 1.06, -0.03],
            [-0.03, -0.03, 1.06],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0100,
            sigma_read: 0.005,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 130.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.14,
        halation_threshold: 0.86,
        halation_sigma: 0.014,
        halation_tint: [1.0, 0.72, 0.52],
    }
}

/// Kodak Ektar 100 (Fine Grain Color Negative)
/// Source: Kodak Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.65
/// Dmax: 2.6, Dmin: 0.12
/// Resolution: 145 lp/mm
pub fn KODAK_EKTAR_100() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Ektar 100".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.6,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.6,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.6,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        color_matrix: [
            [1.10, -0.05, -0.05],
            [-0.05, 1.10, -0.05],
            [-0.05, -0.05, 1.10],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0064,
            sigma_read: 0.004,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.3,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 145.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.12,
        halation_threshold: 0.88,
        halation_sigma: 0.012,
        halation_tint: [1.0, 0.72, 0.52],
    }
}

/// Kodak Kodachrome 25 (Classic Slide Film)
/// Source: Kodak Technical Data
/// ISO: 25
/// RMS: 5 -> Alpha = 0.0025
/// Gamma: 1.5
/// Dmax: 3.8, Dmin: 0.08
/// Resolution: 200 lp/mm
pub fn KODAK_KODACHROME_25() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Kodachrome 25".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 25.0,
        r_curve: SegmentedCurve {
            d_min: 0.08,
            d_max: 3.8,
            gamma: 1.5,
            shoulder_point: 0.8,
            exposure_offset: 0.60,
        },
        g_curve: SegmentedCurve {
            d_min: 0.08,
            d_max: 3.8,
            gamma: 1.5,
            shoulder_point: 0.8,
            exposure_offset: 0.60,
        },
        b_curve: SegmentedCurve {
            d_min: 0.08,
            d_max: 3.8,
            gamma: 1.5,
            shoulder_point: 0.8,
            exposure_offset: 0.60,
        },
        color_matrix: [
            [1.30, -0.15, -0.15],
            [-0.15, 1.30, -0.15],
            [-0.15, -0.15, 1.30],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0025,
            sigma_read: 0.002,
            monochrome: false,
            blur_radius: 0.4,
            roughness: 0.15,
            color_correlation: 0.85,
            shadow_noise: 0.0005,
            highlight_coarseness: 0.03,
        },
        resolution_lp_mm: 200.0,
        reciprocity: ReciprocityFailure { beta: 0.08 },
        halation_strength: 0.06,
        halation_threshold: 0.94,
        halation_sigma: 0.006,
        halation_tint: [1.0, 0.30, 0.30],
    }
}

/// Get all Kodak film stocks
pub fn get_stocks() -> Vec<FilmStock> {
    vec![
        KODAK_PORTRA_400(),
        KODAK_PORTRA_160(),
        KODAK_PORTRA_800(),
        KODAK_TRI_X_400(),
        KODAK_PLUS_X_125(),
        KODAK_EKTACHROME_100(),
        KODAK_EKTACHROME_100VS(),
        KODAK_KODACHROME_64(),
        KODAK_GOLD_200(),
        KODAK_EKTAR_100(),
        KODAK_KODACHROME_25(),
    ]
}
