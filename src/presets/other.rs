//! Other film stock presets (various manufacturers)

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Standard Daylight Film (Generic)
pub fn STANDARD_DAYLIGHT() -> FilmStock {
    FilmStock {
        manufacturer: "Generic".to_string(),
        name: "Standard Daylight".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.9,
            gamma: 1.8,
            exposure_offset: 4.32244,
            shoulder_point: 0.8,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.0,
            gamma: 1.8,
            exposure_offset: 4.32244,
            shoulder_point: 0.8,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 3.0,
            gamma: 1.8,
            exposure_offset: 4.32244,
            shoulder_point: 0.8,
        },
        color_matrix: [[1.00, 0.05, 0.02], [0.04, 1.00, 0.04], [0.01, 0.05, 1.00]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0005,
            sigma_read: 0.01,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.5,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 80.0,
        reciprocity: crate::film::ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.0,
        halation_threshold: 0.8,
        halation_sigma: 0.02,
        halation_tint: [1.0, 0.4, 0.2],
    }
}

/// CineStill 800T (Tungsten Balanced Color Negative)
/// Source: CineStill Technical Data
/// ISO: 800
/// RMS: 13 -> Alpha = 0.0169
/// Gamma: 0.65
/// Dmax: 2.9, Dmin: 0.12
/// Resolution: 110 lp/mm
pub fn CINESTILL_800T() -> FilmStock {
    FilmStock {
        manufacturer: "CineStill".to_string(),
        name: "CineStill 800T".to_string(),
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
            [1.04, -0.02, -0.02],
            [-0.02, 1.04, -0.02],
            [-0.02, -0.02, 1.04],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000286,
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
        halation_tint: [1.0, 0.65, 0.45],
    }
}

/// CineStill 50D (Daylight Balanced Color Negative)
/// Source: CineStill Technical Data
/// ISO: 50
/// RMS: 6 -> Alpha = 0.0036
/// Gamma: 0.65
/// Dmax: 2.6, Dmin: 0.12
/// Resolution: 145 lp/mm
pub fn CINESTILL_50D() -> FilmStock {
    FilmStock {
        manufacturer: "CineStill".to_string(),
        name: "CineStill 50D".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 50.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.6,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.6,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.6,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        color_matrix: [
            [1.08, -0.04, -0.04],
            [-0.04, 1.08, -0.04],
            [-0.04, -0.04, 1.08],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000013,
            sigma_read: 0.003,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.25,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 145.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.11,
        halation_threshold: 0.89,
        halation_sigma: 0.011,
        halation_tint: [1.0, 0.65, 0.45],
    }
}

/// Lomography Color Chrome (Slide Film)
/// Source: Lomography Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 1.2
/// Dmax: 3.3, Dmin: 0.12
/// Resolution: 120 lp/mm
pub fn LOMOGRAPHY_COLOR_CHROME() -> FilmStock {
    FilmStock {
        manufacturer: "Lomography".to_string(),
        name: "Lomography Color Chrome".to_string(),
        film_type: FilmType::ColorSlide,
        iso: 200.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.3,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.3,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 3.3,
            gamma: 1.2,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        color_matrix: [
            [1.18, -0.09, -0.09],
            [-0.09, 1.18, -0.09],
            [-0.09, -0.09, 1.18],
        ],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.000100,
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
        halation_strength: 0.13,
        halation_threshold: 0.87,
        halation_sigma: 0.013,
        halation_tint: [0.95, 0.95, 0.95],
    }
}

/// Lomography Lomochrome Purple (Experimental Color Negative)
/// Source: Lomography Technical Data
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.70
/// Dmax: 2.8, Dmin: 0.12
/// Resolution: 110 lp/mm
pub fn LOMOGRAPHY_LOMOCHROME_PURPLE() -> FilmStock {
    FilmStock {
        manufacturer: "Lomography".to_string(),
        name: "Lomography Lomochrome Purple".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.8,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.8,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.8,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        color_matrix: [[0.95, 0.05, 0.00], [0.00, 0.95, 0.05], [0.05, 0.00, 0.95]],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000207,
            sigma_read: 0.006,
            monochrome: false,
            blur_radius: 0.5,
            roughness: 0.5,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 110.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [0.8, 0.5, 1.0],
    }
}

/// Ferrania Solaris 400 (Color Negative)
/// Source: Ferrania Technical Data
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.65
/// Dmax: 2.8, Dmin: 0.12
/// Resolution: 120 lp/mm
pub fn FERRANIA_SOLARIS_400() -> FilmStock {
    FilmStock {
        manufacturer: "Ferrania".to_string(),
        name: "Ferrania Solaris 400".to_string(),
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
            [1.06, -0.03, -0.03],
            [-0.03, 1.06, -0.03],
            [-0.03, -0.03, 1.06],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000207,
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
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [1.0, 0.70, 0.50],
    }
}

/// Ferrania Solaris 100 (Fine Grain Color Negative)
/// Source: Ferrania Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.65
/// Dmax: 2.6, Dmin: 0.12
/// Resolution: 140 lp/mm
pub fn FERRANIA_SOLARIS_100() -> FilmStock {
    FilmStock {
        manufacturer: "Ferrania".to_string(),
        name: "Ferrania Solaris 100".to_string(),
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
            [1.09, -0.05, -0.04],
            [-0.04, 1.09, -0.05],
            [-0.05, -0.04, 1.09],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000041,
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
        halation_tint: [1.0, 0.70, 0.50],
    }
}

/// Orwo UN54 (B&W Negative)
/// Source: Orwo Technical Data
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 120 lp/mm
pub fn ORWO_UN54() -> FilmStock {
    FilmStock {
        manufacturer: "Orwo".to_string(),
        name: "Orwo UN54".to_string(),
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
            alpha: 0.000207,
            sigma_read: 0.006,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.5,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 120.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.17,
        halation_threshold: 0.83,
        halation_sigma: 0.015,
        halation_tint: [0.89, 0.89, 0.89],
    }
}

/// Orwo UN64 (Fine Grain B&W)
/// Source: Orwo Technical Data
/// ISO: 64
/// RMS: 7 -> Alpha = 0.0049
/// Gamma: 0.75
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 150 lp/mm
pub fn ORWO_UN64() -> FilmStock {
    FilmStock {
        manufacturer: "Orwo".to_string(),
        name: "Orwo UN64".to_string(),
        film_type: FilmType::BwNegative,
        iso: 64.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.31,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.31,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.31,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.000024,
            sigma_read: 0.003,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.25,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 150.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.12,
        halation_threshold: 0.88,
        halation_sigma: 0.012,
        halation_tint: [0.94, 0.94, 0.94],
    }
}

/// Get all other manufacturer film stocks
pub fn get_stocks() -> Vec<FilmStock> {
    vec![
        STANDARD_DAYLIGHT(),
        CINESTILL_800T(),
        CINESTILL_50D(),
        LOMOGRAPHY_COLOR_CHROME(),
        LOMOGRAPHY_LOMOCHROME_PURPLE(),
        FERRANIA_SOLARIS_400(),
        FERRANIA_SOLARIS_100(),
        ORWO_UN54(),
        ORWO_UN64(),
    ]
}
