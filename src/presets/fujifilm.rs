//! Fujifilm film stock presets

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Fujifilm Superia 400 (Consumer Color Negative)
/// Source: Fujifilm Technical Data
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.65
/// Dmax: 2.8, Dmin: 0.12
/// Resolution: 120 lp/mm
pub fn SUPERIA_400() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Superia 400".to_string(),
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
            [1.05, -0.03, -0.02],
            [-0.02, 1.05, -0.03],
            [-0.03, -0.02, 1.05],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.0144,
            sigma_read: 0.006,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.5,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 120.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [1.0, 0.65, 0.45],
    }
}

/// Fujifilm Superia 200 (General Purpose Color Negative)
/// Source: Fujifilm Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.12
/// Resolution: 130 lp/mm
pub fn SUPERIA_200() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Superia 200".to_string(),
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
            [1.08, -0.04, -0.04],
            [-0.04, 1.08, -0.04],
            [-0.04, -0.04, 1.08],
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
        halation_tint: [1.0, 0.65, 0.45],
    }
}

/// Fujifilm Superia 100 (Fine Grain Color Negative)
/// Source: Fujifilm Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.65
/// Dmax: 2.6, Dmin: 0.12
/// Resolution: 140 lp/mm
pub fn SUPERIA_100() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Superia 100".to_string(),
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
        resolution_lp_mm: 140.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.12,
        halation_threshold: 0.88,
        halation_sigma: 0.012,
        halation_tint: [1.0, 0.65, 0.45],
    }
}

/// Fujifilm Neopan 400 (Professional B&W)
/// Source: Fujifilm Technical Data
/// ISO: 400
/// RMS: 14 -> Alpha = 0.0196
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 110 lp/mm
pub fn NEOPAN_400() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Neopan 400".to_string(),
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
        resolution_lp_mm: 110.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.20,
        halation_threshold: 0.82,
        halation_sigma: 0.016,
        halation_tint: [0.85, 0.85, 0.85],
    }
}

/// Fujifilm Neopan 100 (Fine Grain B&W)
/// Source: Fujifilm Technical Data
/// ISO: 100
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 0.75
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 140 lp/mm
pub fn NEOPAN_100() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Neopan 100".to_string(),
        film_type: FilmType::BwNegative,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
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
        halation_tint: [0.90, 0.90, 0.90],
    }
}

/// Fujifilm Provia 100F (Professional Slide Film)
/// Source: Fujifilm Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.3
/// Dmax: 3.5, Dmin: 0.12
/// Resolution: 150 lp/mm
pub fn PROVIA_100F() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Provia 100F".to_string(),
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
            [1.15, -0.08, -0.07],
            [-0.07, 1.15, -0.08],
            [-0.08, -0.07, 1.15],
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

/// Fujifilm Velvia 50 (High Saturation Slide Film)
/// Source: Fujifilm Technical Data
/// ISO: 50
/// RMS: 6 -> Alpha = 0.0036
/// Gamma: 1.4
/// Dmax: 3.6, Dmin: 0.10
/// Resolution: 160 lp/mm
pub fn VELVIA_50() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Velvia 50".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 50.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.6,
            gamma: 1.4,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.6,
            gamma: 1.4,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.6,
            gamma: 1.4,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        color_matrix: [
            [1.30, -0.15, -0.15],
            [-0.15, 1.30, -0.15],
            [-0.15, -0.15, 1.30],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0036,
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
        halation_tint: [1.0, 0.4, 0.4],
    }
}

/// Fujifilm Astia 100F (Soft Tone Slide Film)
/// Source: Fujifilm Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.2
/// Dmax: 3.4, Dmin: 0.12
/// Resolution: 145 lp/mm
pub fn ASTIA_100F() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Astia 100F".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.4,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.4,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.4,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        color_matrix: [
            [1.05, -0.03, -0.02],
            [-0.02, 1.05, -0.03],
            [-0.03, -0.02, 1.05],
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
        resolution_lp_mm: 145.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.09,
        halation_threshold: 0.91,
        halation_sigma: 0.009,
        halation_tint: [0.98, 0.98, 1.0],
    }
}
