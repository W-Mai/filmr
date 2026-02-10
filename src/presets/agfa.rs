//! Agfa film stock presets

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Agfa Vista 400 (Consumer Color Negative)
/// Source: Agfa Technical Data
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.65
/// Dmax: 2.8, Dmin: 0.12
/// Resolution: 115 lp/mm
pub fn VISTA_400() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "Vista 400".to_string(),
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
            [1.03, -0.02, -0.01],
            [-0.01, 1.03, -0.02],
            [-0.02, -0.01, 1.03],
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
        resolution_lp_mm: 115.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.17,
        halation_threshold: 0.83,
        halation_sigma: 0.015,
        halation_tint: [1.0, 0.68, 0.48],
    }
}

/// Agfa Vista 200 (General Purpose Color Negative)
/// Source: Agfa Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.12
/// Resolution: 125 lp/mm
pub fn VISTA_200() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "Vista 200".to_string(),
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
        resolution_lp_mm: 125.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [1.0, 0.68, 0.48],
    }
}

/// Agfa Vista 100 (Fine Grain Color Negative)
/// Source: Agfa Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.65
/// Dmax: 2.6, Dmin: 0.12
/// Resolution: 135 lp/mm
pub fn VISTA_100() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "Vista 100".to_string(),
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
            [1.08, -0.04, -0.04],
            [-0.04, 1.08, -0.04],
            [-0.04, -0.04, 1.08],
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
        resolution_lp_mm: 135.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.13,
        halation_threshold: 0.87,
        halation_sigma: 0.013,
        halation_tint: [1.0, 0.68, 0.48],
    }
}

/// Agfa APX 400 (Professional B&W)
/// Source: Agfa Technical Data
/// ISO: 400
/// RMS: 13 -> Alpha = 0.0169
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 110 lp/mm
pub fn APX_400() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "APX 400".to_string(),
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
            alpha: 0.0169,
            sigma_read: 0.007,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.55,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 110.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.19,
        halation_threshold: 0.81,
        halation_sigma: 0.016,
        halation_tint: [0.86, 0.86, 0.86],
    }
}

/// Agfa APX 100 (Fine Grain B&W)
/// Source: Agfa Technical Data
/// ISO: 100
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 0.75
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 135 lp/mm
pub fn APX_100() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "APX 100".to_string(),
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
        resolution_lp_mm: 135.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.14,
        halation_threshold: 0.86,
        halation_sigma: 0.014,
        halation_tint: [0.91, 0.91, 0.91],
    }
}

/// Agfa Precisa 100 (Professional Slide Film)
/// Source: Agfa Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.3
/// Dmax: 3.5, Dmin: 0.12
/// Resolution: 145 lp/mm
pub fn PRECISA_100() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "Precisa 100".to_string(),
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
            [1.12, -0.06, -0.06],
            [-0.06, 1.12, -0.06],
            [-0.06, -0.06, 1.12],
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
        halation_strength: 0.11,
        halation_threshold: 0.89,
        halation_sigma: 0.011,
        halation_tint: [0.96, 0.96, 0.96],
    }
}

/// Agfa Scala 200 (B&W Slide Film)
/// Source: Agfa Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 1.2
/// Dmax: 3.2, Dmin: 0.10
/// Resolution: 130 lp/mm
pub fn SCALA_200() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "Scala 200".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 200.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.2,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.2,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.2,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0100,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 130.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.12,
        halation_threshold: 0.88,
        halation_sigma: 0.012,
        halation_tint: [0.93, 0.93, 0.93],
    }
}

/// Agfa Optima 200 (Consumer Color Negative)
/// Source: Agfa Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.12
/// Resolution: 120 lp/mm
pub fn OPTIMA_200() -> FilmStock {
    FilmStock {
        manufacturer: "Agfa".to_string(),
        name: "Optima 200".to_string(),
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
            [1.04, -0.02, -0.02],
            [-0.02, 1.04, -0.02],
            [-0.02, -0.02, 1.04],
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
        resolution_lp_mm: 120.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [1.0, 0.68, 0.48],
    }
}

/// Get all Agfa film stocks
pub fn get_stocks() -> Vec<FilmStock> {
    vec![
        VISTA_200(),
        VISTA_400(),
        VISTA_100(),
        APX_400(),
        APX_100(),
        PRECISA_100(),
        SCALA_200(),
        OPTIMA_200(),
    ]
}
