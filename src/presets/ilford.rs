//! Ilford film stock presets

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Ilford HP5 Plus 400 (Professional B&W)
/// Source: Ilford Technical Data
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 120 lp/mm
pub fn HP5_PLUS_400() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "HP5 Plus 400".to_string(),
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
            alpha: 0.0144,
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
        halation_strength: 0.18,
        halation_threshold: 0.83,
        halation_sigma: 0.015,
        halation_tint: [0.88, 0.88, 0.88],
    }
}

/// Ilford FP4 Plus 125 (Fine Grain B&W)
/// Source: Ilford Technical Data
/// ISO: 125
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 0.75
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 140 lp/mm
pub fn FP4_PLUS_125() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "FP4 Plus 125".to_string(),
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

/// Ilford Delta 400 Professional (High Contrast B&W)
/// Source: Ilford Technical Data
/// ISO: 400
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 0.80
/// Dmax: 2.4, Dmin: 0.10
/// Resolution: 130 lp/mm
pub fn DELTA_400_PROFESSIONAL() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "Delta 400 Professional".to_string(),
        film_type: FilmType::BwNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.4,
            gamma: 0.80,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.4,
            gamma: 0.80,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.4,
            gamma: 0.80,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0121,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.45,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 130.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [0.90, 0.90, 0.90],
    }
}

/// Ilford Delta 100 Professional (Fine Grain B&W)
/// Source: Ilford Technical Data
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.85
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 150 lp/mm
pub fn DELTA_100_PROFESSIONAL() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "Delta 100 Professional".to_string(),
        film_type: FilmType::BwNegative,
        iso: 100.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.85,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.85,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.85,
            shoulder_point: 0.8,
            exposure_offset: 0.20,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0064,
            sigma_read: 0.004,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.35,
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

/// Ilford Pan F Plus 50 (Ultra Fine Grain B&W)
/// Source: Ilford Technical Data
/// ISO: 50
/// RMS: 6 -> Alpha = 0.0036
/// Gamma: 0.80
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 170 lp/mm
pub fn PAN_F_PLUS_50() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "Pan F Plus 50".to_string(),
        film_type: FilmType::BwNegative,
        iso: 50.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.80,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.80,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.80,
            shoulder_point: 0.8,
            exposure_offset: 0.40,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0036,
            sigma_read: 0.003,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.25,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 170.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.10,
        halation_threshold: 0.90,
        halation_sigma: 0.010,
        halation_tint: [0.96, 0.96, 0.96],
    }
}

/// Ilford XP2 Super 400 (Chromogenic B&W)
/// Source: Ilford Technical Data
/// ISO: 400
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.65
/// Dmax: 2.5, Dmin: 0.12
/// Resolution: 125 lp/mm
pub fn XP2_SUPER_400() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "XP2 Super 400".to_string(),
        film_type: FilmType::BwNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.5,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        g_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.5,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
        },
        b_curve: SegmentedCurve {
            d_min: 0.12,
            d_max: 2.5,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 0.05,
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
        resolution_lp_mm: 125.0,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.14,
        halation_threshold: 0.86,
        halation_sigma: 0.014,
        halation_tint: [0.92, 0.92, 0.92],
    }
}

/// Ilford SFX 200 (Infrared Sensitive B&W)
/// Source: Ilford Technical Data
/// ISO: 200
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.70
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 130 lp/mm
pub fn SFX_200() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "SFX 200".to_string(),
        film_type: FilmType::BwNegative,
        iso: 200.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.70,
            shoulder_point: 0.8,
            exposure_offset: 0.10,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.3,
            gamma: 0.70,
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
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [0.90, 0.90, 0.90],
    }
}

/// Ilford Ortho Plus 80 (Orthochromatic B&W)
/// Source: Ilford Technical Data
/// ISO: 80
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.75
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 140 lp/mm
pub fn ORTHO_PLUS_80() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "Ortho Plus 80".to_string(),
        film_type: FilmType::BwNegative,
        iso: 80.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.25,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.25,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.2,
            gamma: 0.75,
            shoulder_point: 0.8,
            exposure_offset: 0.25,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.0064,
            sigma_read: 0.004,
            monochrome: true,
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
        halation_tint: [0.93, 0.93, 0.93],
    }
}
