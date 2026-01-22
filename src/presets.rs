use crate::film::{FilmStock, SegmentedCurve};
use crate::grain::GrainModel;

/// Standard Daylight Film (Generic)
pub const STANDARD_DAYLIGHT: FilmStock = FilmStock {
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 2.9,
        gamma: 1.8,
        exposure_offset: 0.18,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 3.0,
        gamma: 1.8,
        exposure_offset: 0.18,
    },
    b_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.8,
        gamma: 1.7,
        exposure_offset: 0.18,
    },
    color_matrix: [[1.00, 0.05, 0.02], [0.04, 1.00, 0.04], [0.01, 0.05, 1.00]],
    spectral_sensitivity: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    grain_model: GrainModel {
        alpha: 0.05,
        sigma_read: 0.01,
        monochrome: false,
    },
    resolution_lp_mm: 80.0,
    reciprocity_exponent: 0.85,
    halation_strength: 0.0,
    halation_threshold: 0.8,
    halation_sigma: 0.02,
    halation_tint: [1.0, 0.4, 0.2],
};

/// Kodak Tri-X 400 (Classic B&W / News Film)
/// Source: Kodak F-4017 & ISO Standards
/// ISO: 400
/// RMS Granularity: 17 -> Alpha = 10^-4 * 17^2 = 0.0289
/// Gamma: 0.70
/// Dmax: 2.3, Dmin: 0.11
/// Resolution: 100 lp/mm
/// Reciprocity: 1s -> +1 stop (exponent ~0.7)
pub const KODAK_TRI_X_400: FilmStock = FilmStock {
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.3,
        gamma: 0.70,
        exposure_offset: 0.0025,
    },
    g_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.3,
        gamma: 0.70,
        exposure_offset: 0.0025,
    },
    b_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.3,
        gamma: 0.70,
        exposure_offset: 0.0025,
    },
    color_matrix: [[0.30, 0.59, 0.11], [0.30, 0.59, 0.11], [0.30, 0.59, 0.11]],
    spectral_sensitivity: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    grain_model: GrainModel {
        alpha: 0.0289,
        sigma_read: 0.02,
        monochrome: true,
    }, // RMS 17
    resolution_lp_mm: 100.0,
    reciprocity_exponent: 0.70,
    halation_strength: 0.2,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [0.8, 0.8, 0.8],
};

/// Fujifilm Velvia 50 (Landscape Slide Film)
/// Source: Fujifilm Data Sheet & tec3.md
/// ISO: 50
/// RMS: 9 -> Alpha = 10^-4 * 9^2 = 0.0081
/// Gamma: 1.3
/// Dmax: 3.7, Dmin: 0.15
/// Resolution: 160 lp/mm
/// Reciprocity: Very stable until 1/4000s or >1s. Exponent ~0.95.
/// Spectral: Enhanced Red Sensitivity
pub const FUJIFILM_VELVIA_50: FilmStock = FilmStock {
    iso: 50.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 0.02,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 0.02,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 0.02,
    },
    color_matrix: [
        [1.1, -0.05, -0.05],
        [-0.05, 1.1, -0.05],
        [-0.05, -0.05, 1.1],
    ],
    spectral_sensitivity: [[1.1, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    grain_model: GrainModel {
        alpha: 0.0081,
        sigma_read: 0.005,
        monochrome: false,
    }, // RMS 9
    resolution_lp_mm: 160.0,
    reciprocity_exponent: 0.95,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.5, 0.3],
};

/// ILFORD HP5 Plus (Versatile B&W)
/// Source: ILFORD Data Sheet & tec3.md
/// ISO: 400
/// RMS: 16 -> Alpha = 10^-4 * 16^2 = 0.0256
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.09
/// Resolution: 95 lp/mm
/// Reciprocity: Good
/// Spectral: Slightly lower Red response
pub const ILFORD_HP5_PLUS: FilmStock = FilmStock {
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.09,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 0.0025,
    },
    g_curve: SegmentedCurve {
        d_min: 0.09,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 0.0025,
    },
    b_curve: SegmentedCurve {
        d_min: 0.09,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 0.0025,
    },
    color_matrix: [[0.30, 0.59, 0.11], [0.30, 0.59, 0.11], [0.30, 0.59, 0.11]],
    spectral_sensitivity: [[0.9, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    grain_model: GrainModel {
        alpha: 0.0256,
        sigma_read: 0.02,
        monochrome: true,
    }, // RMS 16
    resolution_lp_mm: 95.0,
    reciprocity_exponent: 0.80,
    halation_strength: 0.25,
    halation_threshold: 0.8,
    halation_sigma: 0.02,
    halation_tint: [0.8, 0.8, 0.8],
};
