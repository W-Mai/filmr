use crate::film::{FilmStock, SegmentedCurve};

/// Standard Daylight Film (Generic)
pub const STANDARD_DAYLIGHT: FilmStock = FilmStock {
    r_curve: SegmentedCurve {
        d_min: 0.12, d_max: 2.9, gamma: 1.8, exposure_offset: 0.18,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10, d_max: 3.0, gamma: 1.8, exposure_offset: 0.18,
    },
    b_curve: SegmentedCurve {
        d_min: 0.11, d_max: 2.8, gamma: 1.7, exposure_offset: 0.18,
    },
    color_matrix: [
        [1.00, 0.05, 0.02],
        [0.04, 1.00, 0.04],
        [0.01, 0.05, 1.00],
    ],
    halation_strength: 0.0,
    halation_threshold: 0.8,
    halation_sigma: 0.02,
    halation_tint: [1.0, 0.4, 0.2],
};

/// Kodak Tri-X 400 (Classic B&W / News Film)
pub const KODAK_TRI_X_400: FilmStock = FilmStock {
    r_curve: SegmentedCurve {
        d_min: 0.10, d_max: 2.3, gamma: 0.70, exposure_offset: 0.0025,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10, d_max: 2.3, gamma: 0.70, exposure_offset: 0.0025,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10, d_max: 2.3, gamma: 0.70, exposure_offset: 0.0025,
    },
    color_matrix: [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ],
    halation_strength: 0.2,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [0.8, 0.8, 0.8],
};

/// Fujifilm Velvia 50 (Landscape Slide Film)
pub const FUJIFILM_VELVIA_50: FilmStock = FilmStock {
    r_curve: SegmentedCurve {
        d_min: 0.15, d_max: 3.8, gamma: 1.3, exposure_offset: 0.02,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15, d_max: 3.8, gamma: 1.3, exposure_offset: 0.02,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15, d_max: 3.8, gamma: 1.3, exposure_offset: 0.02,
    },
    color_matrix: [
        [1.1, -0.05, -0.05],
        [-0.05, 1.1, -0.05],
        [-0.05, -0.05, 1.1],
    ],
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.5, 0.3],
};

/// ILFORD HP5 Plus (Versatile B&W)
pub const ILFORD_HP5_PLUS: FilmStock = FilmStock {
    r_curve: SegmentedCurve {
        d_min: 0.08, d_max: 2.2, gamma: 0.65, exposure_offset: 0.0025,
    },
    g_curve: SegmentedCurve {
        d_min: 0.08, d_max: 2.2, gamma: 0.65, exposure_offset: 0.0025,
    },
    b_curve: SegmentedCurve {
        d_min: 0.08, d_max: 2.2, gamma: 0.65, exposure_offset: 0.0025,
    },
    color_matrix: [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ],
    halation_strength: 0.25,
    halation_threshold: 0.8,
    halation_sigma: 0.02,
    halation_tint: [0.8, 0.8, 0.8],
};
