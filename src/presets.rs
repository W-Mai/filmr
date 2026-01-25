use crate::film::{FilmStock, SegmentedCurve, FilmType};
use crate::grain::GrainModel;
use crate::spectral::FilmSpectralParams;

/// Standard Daylight Film (Generic)
pub const STANDARD_DAYLIGHT: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 2.9,
        gamma: 1.8,
        exposure_offset: 4.32244,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 3.0,
        gamma: 1.8,
        exposure_offset: 4.32244,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 3.0,
        gamma: 1.8,
        exposure_offset: 4.32244,
    },
    color_matrix: [[1.00, 0.05, 0.02], [0.04, 1.00, 0.04], [0.01, 0.05, 1.00]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.05,
        sigma_read: 0.01,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 80.0,
    reciprocity_beta: 0.05,
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
    film_type: FilmType::BwNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.3,
        gamma: 0.70,
        exposure_offset: 48.87788,
    },
    g_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.3,
        gamma: 0.70,
        exposure_offset: 48.87788,
    },
    b_curve: SegmentedCurve {
        d_min: 0.11,
        d_max: 2.3,
        gamma: 0.70,
        exposure_offset: 48.87788,
    },
    color_matrix: [[0.30, 0.59, 0.11], [0.30, 0.59, 0.11], [0.30, 0.59, 0.11]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0289,
        sigma_read: 0.02,
        monochrome: true,
        blur_radius:  1.2,
    }, // RMS 17
    resolution_lp_mm: 100.0,
    reciprocity_beta: 0.05,
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
    film_type: FilmType::ColorSlide,
    iso: 50.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 49.22617,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 49.22617,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 49.22617,
    },
    color_matrix: [
        [1.1, -0.05, -0.05],
        [-0.05, 1.1, -0.05],
        [-0.05, -0.05, 1.1],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0081,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius:  0.4,
    }, // RMS 9
    resolution_lp_mm: 160.0,
    reciprocity_beta: 0.05,
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
    film_type: FilmType::BwNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.09,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 34.22952,
    },
    g_curve: SegmentedCurve {
        d_min: 0.09,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 34.22952,
    },
    b_curve: SegmentedCurve {
        d_min: 0.09,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 34.22952,
    },
    color_matrix: [[0.30, 0.59, 0.11], [0.30, 0.59, 0.11], [0.30, 0.59, 0.11]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0256,
        sigma_read: 0.02,
        monochrome: true,
        blur_radius: 0.5,
    }, // RMS 16
    resolution_lp_mm: 95.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.25,
    halation_threshold: 0.8,
    halation_sigma: 0.02,
    halation_tint: [0.8, 0.8, 0.8],
};

/// Kodak Portra 400 (Professional Color Negative)
/// Source: Kodak E-7053
/// ISO: 400
/// PGI: 35 -> Est. RMS ~11 -> Alpha = 0.0121
/// Gamma: 0.65 (Negative)
/// Dmax: 2.9, Dmin: 0.15
/// Resolution: 115 lp/mm
/// Spectral: Neutral + 15% Saturation
pub const KODAK_PORTRA_400: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 2.9,
        gamma: 0.65,
        exposure_offset: 625.046_9,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 2.9,
        gamma: 0.65,
        exposure_offset: 625.046_9,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 2.9,
        gamma: 0.65,
        exposure_offset: 625.046_9,
    },
    color_matrix: [
        [1.15, -0.05, -0.1],
        [-0.05, 1.15, -0.1],
        [-0.1, -0.05, 1.15],
    ],
    // Simulating slight spectral overlap (Red sees some Green, Green sees some Blue)
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0121,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 115.0,
    reciprocity_beta: 0.05, // Typical color negative
    halation_strength: 0.15,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [1.0, 0.6, 0.4], // Warm glow
};

/// Kodak Ektar 100 (Fine Grain Color Negative)
/// Source: Kodak E-7043
/// ISO: 100
/// PGI: 25 -> Est. RMS ~8 -> Alpha = 0.0064
/// Gamma: 0.75 (High Saturation Negative)
/// Dmax: 3.2, Dmin: 0.15
/// Resolution: 150 lp/mm
/// Spectral: High Saturation (+25%)
pub const KODAK_EKTAR_100: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.2,
        gamma: 0.85,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.2,
        gamma: 0.85,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.2,
        gamma: 0.85,
        exposure_offset: 0.20,
    },
    color_matrix: [
        [1.25, -0.10, -0.15],
        [-0.10, 1.25, -0.15],
        [-0.15, -0.10, 1.25],
    ],
    // Ektar has high color separation (enhanced saturation)
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.003,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 150.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.5, 0.3],
};

/// Kodak T-Max 3200 (High Speed B&W)
/// Source: Kodak F-4016
/// ISO: 3200 (EI 800-6400)
/// RMS: 18 -> Alpha = 0.0324
/// Gamma: 0.75
/// Dmax: 2.1, Dmin: 0.15
/// Resolution: 80 lp/mm
/// Reciprocity: Good for high speed
pub const KODAK_T_MAX_3200: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 3200.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 2.1,
        gamma: 0.75,
        exposure_offset: 0.00625, // Very fast
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 2.1,
        gamma: 0.75,
        exposure_offset: 0.00625,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 2.1,
        gamma: 0.75,
        exposure_offset: 0.00625,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    // Standard Panchromatic response
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0324,
        sigma_read: 0.03,
        monochrome: true,
        blur_radius:  1.5,
    },
    resolution_lp_mm: 80.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.3,
    halation_threshold: 0.7,
    halation_sigma: 0.025,
    halation_tint: [0.9, 0.9, 0.9],
};

/// Ilford Delta 100 (Professional B&W)
/// Source: Ilford 2021 Data Sheet
/// ISO: 100
/// RMS: 7 -> Alpha = 0.0049
/// Gamma: 0.70
/// Dmax: 2.2, Dmin: 0.08
/// Resolution: 160 lp/mm
/// Crystal: Core-Shell
pub const ILFORD_DELTA_100: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.2,
        gamma: 0.70,
        exposure_offset: 0.20,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    // Standard Panchromatic response
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0049,
        sigma_read: 0.005,
        monochrome: true,
        blur_radius:  0.4,
    },
    resolution_lp_mm: 160.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [0.85, 0.85, 0.85],
};

/// Fujifilm Pro 400H (Professional Color Negative)
/// Source: Fujifilm 2020 Data Sheet
/// ISO: 400
/// RMS: 12 -> Alpha = 0.0144
/// Gamma: 0.65 (Wide Latitude)
/// Dmax: 2.8, Dmin: 0.10
/// Resolution: 125 lp/mm
/// Spectral: Fourth Color Layer Sim (Cyan-ish)
pub const FUJIFILM_PRO_400H: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.8,
        gamma: 0.65,
        exposure_offset: 0.05,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.8,
        gamma: 0.65,
        exposure_offset: 0.05,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.8,
        gamma: 0.65,
        exposure_offset: 0.05,
    },
    color_matrix: [
        [1.05, 0.0, -0.05],
        [-0.05, 1.1, -0.05], // Slightly better greens
        [-0.05, 0.0, 1.05],
    ],
    // Fuji colors: distinct Green/Blue handling
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0144,
        sigma_read: 0.008,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.12,
    halation_threshold: 0.88,
    halation_sigma: 0.018,
    halation_tint: [0.8, 1.0, 0.9], // Cooler halation tint characteristic of Fuji?
};

/// Fujifilm Velvia 100F (High Saturation Slide Film)
/// Source: Fujifilm 2020 Data Sheet
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.2
/// Dmax: 3.8, Dmin: 0.15
/// Resolution: 160 lp/mm
pub const FUJIFILM_VELVIA_100F: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.8,
        gamma: 1.2,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.8,
        gamma: 1.2,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.8,
        gamma: 1.2,
        exposure_offset: 0.20,
    },
    color_matrix: [
        [1.15, -0.05, -0.1],
        [-0.05, 1.15, -0.1],
        [-0.1, -0.05, 1.15],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius:  0.4,
    },
    resolution_lp_mm: 160.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.5, 0.4],
};

/// Fujifilm Velvia 100 (Vivid Color Slide Film)
/// Source: Fujifilm 2018 Technical Bulletin
/// ISO: 100
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 1.3
/// Dmax: 3.7, Dmin: 0.16
/// Resolution: 160 lp/mm
pub const FUJIFILM_VELVIA_100: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.16,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.16,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.16,
        d_max: 3.7,
        gamma: 1.3,
        exposure_offset: 0.20,
    },
    color_matrix: [[1.2, -0.1, -0.1], [-0.1, 1.2, -0.1], [-0.1, -0.1, 1.2]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0081,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius:  0.4,
    },
    resolution_lp_mm: 160.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.4, 0.3],
};

/// Fujifilm Provia 100F (Professional Slide Film)
/// Source: Fujifilm 2020 Data Sheet
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 1.0
/// Dmax: 3.2, Dmin: 0.12
/// Resolution: 135 lp/mm
pub const FUJIFILM_PROVIA_100F: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.2,
        gamma: 1.0,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.2,
        gamma: 1.0,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.2,
        gamma: 1.0,
        exposure_offset: 0.20,
    },
    color_matrix: [[1.05, 0.0, -0.05], [0.0, 1.05, -0.05], [-0.05, 0.0, 1.05]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 135.0,
    reciprocity_beta: 0.05, // Very good reciprocity
    halation_strength: 0.08,
    halation_threshold: 0.9,
    halation_sigma: 0.012,
    halation_tint: [0.9, 0.8, 0.8],
};

/// Fujifilm Astia 100F (Soft Portrait Slide Film)
/// Source: Fujifilm 2016 Data Sheet
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.7
/// Dmax: 3.0, Dmin: 0.12
/// Resolution: 135 lp/mm
pub const FUJIFILM_ASTIA_100F: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.0,
        gamma: 0.9,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.0,
        gamma: 0.9,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.0,
        gamma: 0.9,
        exposure_offset: 0.20,
    },
    color_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], // Very neutral
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.004,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 135.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.05,
    halation_threshold: 0.92,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.9, 0.9],
};

/// Fujifilm Provia 400X (High Speed Slide Film)
/// Source: Fujifilm 2013 Data Sheet
/// ISO: 400
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 0.95
/// Dmax: 3.4, Dmin: 0.14
/// Resolution: 125 lp/mm
pub const FUJIFILM_PROVIA_400X: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.14,
        d_max: 3.4,
        gamma: 1.1,
        exposure_offset: 0.005,
    },
    g_curve: SegmentedCurve {
        d_min: 0.14,
        d_max: 3.4,
        gamma: 1.1,
        exposure_offset: 0.005,
    },
    b_curve: SegmentedCurve {
        d_min: 0.14,
        d_max: 3.4,
        gamma: 1.1,
        exposure_offset: 0.005,
    },
    color_matrix: [
        [1.1, -0.05, -0.05],
        [-0.05, 1.1, -0.05],
        [-0.05, -0.05, 1.1],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0121,
        sigma_read: 0.006,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.12,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [0.9, 0.8, 0.8],
};

/// Fujifilm TREBI 400
/// Source: Fujifilm 2005 Data Sheet
/// ISO: 400
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 1.0
/// Dmax: 3.3, Dmin: 0.15
/// Resolution: 125 lp/mm
pub const FUJIFILM_TREBI_400: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.3,
        gamma: 1.0,
        exposure_offset: 0.005,
    },
    g_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.3,
        gamma: 1.0,
        exposure_offset: 0.005,
    },
    b_curve: SegmentedCurve {
        d_min: 0.15,
        d_max: 3.3,
        gamma: 1.0,
        exposure_offset: 0.005,
    },
    color_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0121,
        sigma_read: 0.006,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.88,
    halation_sigma: 0.015,
    halation_tint: [1.0, 0.9, 0.8],
};

/// Fujifilm Pro 160NS (Professional Color Negative)
/// Source: Fujifilm 2020 Data Sheet
/// ISO: 160
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.65
/// Dmax: 2.6, Dmin: 0.08
/// Resolution: 135 lp/mm
pub const FUJIFILM_PRO_160NS: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 160.0,
    r_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.6,
        gamma: 0.65,
        exposure_offset: 0.125,
    },
    g_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.6,
        gamma: 0.65,
        exposure_offset: 0.125,
    },
    b_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.6,
        gamma: 0.65,
        exposure_offset: 0.125,
    },
    color_matrix: [
        [1.05, -0.02, -0.03],
        [-0.02, 1.05, -0.03],
        [-0.03, -0.02, 1.05],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.004,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 135.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [0.9, 0.9, 1.0], // Fuji signature
};

/// Fujifilm Pro 160NC (Professional Color Negative - Neutral)
/// Source: Fujifilm 2016 Data Sheet
/// ISO: 160
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.60
/// Dmax: 2.5, Dmin: 0.08
/// Resolution: 125 lp/mm
pub const FUJIFILM_PRO_160NC: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 160.0,
    r_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.5,
        gamma: 0.60,
        exposure_offset: 0.125,
    },
    g_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.5,
        gamma: 0.60,
        exposure_offset: 0.125,
    },
    b_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.5,
        gamma: 0.60,
        exposure_offset: 0.125,
    },
    color_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.004,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.08,
    halation_threshold: 0.92,
    halation_sigma: 0.01,
    halation_tint: [0.95, 0.95, 0.95],
};

/// Fujifilm Superia 200 (Consumer Color Negative)
/// Source: Fujifilm 2018 Data Sheet
/// ISO: 200
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.10
/// Resolution: 125 lp/mm
pub const FUJIFILM_SUPERIA_200: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 200.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.7,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.7,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.7,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    color_matrix: [
        [1.1, -0.05, -0.05],
        [-0.05, 1.1, -0.05],
        [-0.05, -0.05, 1.1],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0121,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.15,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [1.0, 0.8, 0.8],
};

/// Fujifilm Superia X-tra 800
/// Source: Fujifilm 2019 Data Sheet
/// ISO: 800
/// RMS: 13 -> Alpha = 0.0169
/// Gamma: 0.65
/// Dmax: 2.9, Dmin: 0.12
/// Resolution: 110 lp/mm
pub const FUJIFILM_SUPERIA_X_TRA_800: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 800.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 2.9,
        gamma: 0.65,
        exposure_offset: 0.025,
    },
    g_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 2.9,
        gamma: 0.65,
        exposure_offset: 0.025,
    },
    b_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 2.9,
        gamma: 0.65,
        exposure_offset: 0.025,
    },
    color_matrix: [[1.05, -0.05, 0.0], [-0.05, 1.05, 0.0], [0.0, 0.0, 1.05]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0169,
        sigma_read: 0.008,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 110.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.2,
    halation_threshold: 0.8,
    halation_sigma: 0.02,
    halation_tint: [1.0, 0.7, 0.7],
};

/// Kodak T-Max 400 (Professional B&W)
/// Source: Kodak F-4016
/// ISO: 400
/// RMS: 10 -> Alpha = 0.0100
/// Gamma: 0.85
/// Dmax: 2.4, Dmin: 0.10
/// Resolution: 125 lp/mm
pub const KODAK_T_MAX_400: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.4,
        gamma: 0.85,
        exposure_offset: 0.05,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.4,
        gamma: 0.85,
        exposure_offset: 0.05,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.4,
        gamma: 0.85,
        exposure_offset: 0.05,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0100,
        sigma_read: 0.005,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.15,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [0.9, 0.9, 0.9],
};

/// Kodak T-Max 100 (Fine Grain B&W)
/// Source: Kodak F-4016
/// ISO: 100
/// RMS: 8 -> Alpha = 0.0064
/// Gamma: 0.80
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 200 lp/mm
pub const KODAK_T_MAX_100: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.3,
        gamma: 0.80,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.3,
        gamma: 0.80,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.3,
        gamma: 0.80,
        exposure_offset: 0.20,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0064,
        sigma_read: 0.004,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 200.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [0.95, 0.95, 0.95],
};

/// Kodak Plus-X 125 (General Purpose B&W)
/// Source: Kodak F-4017
/// ISO: 125
/// RMS: 13 -> Alpha = 0.0169
/// Gamma: 0.65
/// Dmax: 2.1, Dmin: 0.10
/// Resolution: 125 lp/mm
pub const KODAK_PLUS_X_125: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 125.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.1,
        gamma: 0.65,
        exposure_offset: 0.16,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.1,
        gamma: 0.65,
        exposure_offset: 0.16,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.1,
        gamma: 0.65,
        exposure_offset: 0.16,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0169,
        sigma_read: 0.006,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.18,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [0.85, 0.85, 0.85],
};

/// Ilford FP4 Plus (Fine Grain B&W)
/// Source: Ilford 2021 Data Sheet
/// ISO: 125
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 0.65
/// Dmax: 2.0, Dmin: 0.08
/// Resolution: 135 lp/mm
pub const ILFORD_FP4_PLUS: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 125.0,
    r_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.0,
        gamma: 0.65,
        exposure_offset: 0.16,
    },
    g_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.0,
        gamma: 0.65,
        exposure_offset: 0.16,
    },
    b_curve: SegmentedCurve {
        d_min: 0.08,
        d_max: 2.0,
        gamma: 0.65,
        exposure_offset: 0.16,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0121,
        sigma_read: 0.005,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 135.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.15,
    halation_threshold: 0.85,
    halation_sigma: 0.015,
    halation_tint: [0.9, 0.9, 0.9],
};

/// Ilford Delta 400 (Professional B&W)
/// Source: Ilford 2021 Data Sheet
/// ISO: 400
/// RMS: 11 -> Alpha = 0.0121
/// Gamma: 0.75
/// Dmax: 2.3, Dmin: 0.10
/// Resolution: 125 lp/mm
pub const ILFORD_DELTA_400: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 400.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.3,
        gamma: 0.75,
        exposure_offset: 0.05,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.3,
        gamma: 0.75,
        exposure_offset: 0.05,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.3,
        gamma: 0.75,
        exposure_offset: 0.05,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0121,
        sigma_read: 0.006,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.12,
    halation_threshold: 0.88,
    halation_sigma: 0.015,
    halation_tint: [0.88, 0.88, 0.88],
};

/// Ilford Pan F Plus (Slow B&W)
/// Source: Ilford 2021 Data Sheet
/// ISO: 50
/// RMS: 5 -> Alpha = 0.0025
/// Gamma: 0.60
/// Dmax: 1.9, Dmin: 0.05
/// Resolution: 180 lp/mm
pub const ILFORD_PAN_F_PLUS: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 50.0,
    r_curve: SegmentedCurve {
        d_min: 0.05,
        d_max: 1.9,
        gamma: 0.60,
        exposure_offset: 0.40,
    },
    g_curve: SegmentedCurve {
        d_min: 0.05,
        d_max: 1.9,
        gamma: 0.60,
        exposure_offset: 0.40,
    },
    b_curve: SegmentedCurve {
        d_min: 0.05,
        d_max: 1.9,
        gamma: 0.60,
        exposure_offset: 0.40,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0025,
        sigma_read: 0.003,
        monochrome: true,
        blur_radius:  0.3,
    },
    resolution_lp_mm: 180.0,
    reciprocity_beta: 0.05, // Needs precise exposure
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [0.95, 0.95, 0.95],
};

/// Ilford SFX 200 (Extended Red Sensitivity)
/// Source: Ilford 2021 Data Sheet
/// ISO: 200
/// RMS: 14 -> Alpha = 0.0196
/// Gamma: 0.65
/// Dmax: 2.0, Dmin: 0.10
/// Resolution: 100 lp/mm
pub const ILFORD_SFX_200: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 200.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.0,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.0,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.0,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    // Enhanced Red Sensitivity (Infrared-like)
    spectral_params: FilmSpectralParams::new_infrared(),
    grain_model: GrainModel {
        alpha: 0.0196,
        sigma_read: 0.008,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 100.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.25, // IR films often have strong halation
    halation_threshold: 0.8,
    halation_sigma: 0.03,
    halation_tint: [0.9, 0.7, 0.7],
};

/// Kodak Portra 160 (Professional Color Negative)
/// Source: Kodak E-7053
/// ISO: 160
/// PGI: 28 -> Est. RMS ~9 -> Alpha = 0.0081
/// Gamma: 0.65
/// Dmax: 2.8, Dmin: 0.10
/// Resolution: 125 lp/mm
pub const KODAK_PORTRA_160: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 160.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.8,
        gamma: 0.75,
        exposure_offset: 0.125,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.8,
        gamma: 0.75,
        exposure_offset: 0.125,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.8,
        gamma: 0.75,
        exposure_offset: 0.125,
    },
    color_matrix: [
        [1.1, -0.05, -0.05],
        [-0.05, 1.1, -0.05],
        [-0.05, -0.05, 1.1],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0081,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 125.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.12,
    halation_threshold: 0.88,
    halation_sigma: 0.012,
    halation_tint: [1.0, 0.6, 0.5],
};

/// Kodak Gold 200 (Consumer Color Negative)
/// Source: Kodak E-7041
/// ISO: 200
/// PGI: 40 -> Est. RMS ~12 -> Alpha = 0.0144
/// Gamma: 0.65
/// Dmax: 2.7, Dmin: 0.10
/// Resolution: 110 lp/mm
pub const KODAK_GOLD_200: FilmStock = FilmStock {
    film_type: FilmType::ColorNegative,
    iso: 200.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.7,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.7,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.7,
        gamma: 0.65,
        exposure_offset: 0.10,
    },
    color_matrix: [[1.15, -0.1, 0.0], [-0.05, 1.1, -0.05], [0.0, -0.05, 1.05]], // Warm tone
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0144,
        sigma_read: 0.006,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 110.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.18,
    halation_threshold: 0.82,
    halation_sigma: 0.015,
    halation_tint: [1.0, 0.7, 0.4],
};

/// Kodachrome 25 (Classic Slide Film)
/// Source: Kodak 1998 Archive
/// ISO: 25
/// RMS: 5 -> Alpha = 0.0025
/// Gamma: 1.4 (High Contrast)
/// Dmax: 3.6, Dmin: 0.10
/// Resolution: 200 lp/mm
pub const KODACHROME_25: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 25.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 3.6,
        gamma: 1.4,
        exposure_offset: 0.80,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 3.6,
        gamma: 1.4,
        exposure_offset: 0.80,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 3.6,
        gamma: 1.4,
        exposure_offset: 0.80,
    },
    color_matrix: [[1.2, -0.1, -0.1], [-0.1, 1.2, -0.1], [-0.1, -0.1, 1.2]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0025,
        sigma_read: 0.003,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 200.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.05,
    halation_threshold: 0.95,
    halation_sigma: 0.005,
    halation_tint: [0.9, 0.9, 0.9],
};

pub fn get_all_stocks() -> Vec<(&'static str, FilmStock)> {
    vec![
        ("Standard Daylight", STANDARD_DAYLIGHT),
        ("Kodak Tri-X 400", KODAK_TRI_X_400),
        ("Fujifilm Velvia 50", FUJIFILM_VELVIA_50),
        ("Ilford HP5 Plus", ILFORD_HP5_PLUS),
        ("Kodak Portra 400", KODAK_PORTRA_400),
        ("Kodak Ektar 100", KODAK_EKTAR_100),
        ("Kodak T-Max 3200", KODAK_T_MAX_3200),
        ("Ilford Delta 100", ILFORD_DELTA_100),
        ("Fujifilm Pro 400H", FUJIFILM_PRO_400H),
        ("Fujifilm Velvia 100F", FUJIFILM_VELVIA_100F),
        ("Fujifilm Velvia 100", FUJIFILM_VELVIA_100),
        ("Fujifilm Provia 100F", FUJIFILM_PROVIA_100F),
        ("Fujifilm Astia 100F", FUJIFILM_ASTIA_100F),
        ("Fujifilm Provia 400X", FUJIFILM_PROVIA_400X),
        ("Fujifilm Trebi 400", FUJIFILM_TREBI_400),
        ("Fujifilm Pro 160NS", FUJIFILM_PRO_160NS),
        ("Fujifilm Pro 160NC", FUJIFILM_PRO_160NC),
        ("Fujifilm Superia 200", FUJIFILM_SUPERIA_200),
        ("Fujifilm Superia X-tra 800", FUJIFILM_SUPERIA_X_TRA_800),
        ("Kodak T-Max 400", KODAK_T_MAX_400),
        ("Kodak T-Max 100", KODAK_T_MAX_100),
        ("Kodak Plus-X 125", KODAK_PLUS_X_125),
        ("Ilford FP4 Plus", ILFORD_FP4_PLUS),
        ("Ilford Delta 400", ILFORD_DELTA_400),
        ("Ilford Pan F Plus", ILFORD_PAN_F_PLUS),
        ("Ilford SFX 200", ILFORD_SFX_200),
        ("Kodak Portra 160", KODAK_PORTRA_160),
        ("Kodak Gold 200", KODAK_GOLD_200),
        ("Kodachrome 25", KODACHROME_25),
        ("Kodachrome 64", KODACHROME_64),
        ("Kodak Ektachrome 100VS", KODAK_EKTACHROME_100VS),
        ("Fujifilm Neopan Acros 100", FUJIFILM_NEOPAN_ACROS_100),
        ("Polaroid SX-70", POLAROID_SX_70),
    ]
}

/// Kodachrome 64
/// Source: Kodak 2008 Archive
/// ISO: 64
/// RMS: 7 -> Alpha = 0.0049
/// Gamma: 1.3
/// Dmax: 3.5, Dmin: 0.12
/// Resolution: 160 lp/mm
pub const KODACHROME_64: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 64.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.5,
        gamma: 1.3,
        exposure_offset: 0.31,
    },
    g_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.5,
        gamma: 1.3,
        exposure_offset: 0.31,
    },
    b_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.5,
        gamma: 1.3,
        exposure_offset: 0.31,
    },
    color_matrix: [
        [1.15, -0.08, -0.07],
        [-0.07, 1.15, -0.08],
        [-0.08, -0.07, 1.15],
    ],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0049,
        sigma_read: 0.004,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 160.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.08,
    halation_threshold: 0.9,
    halation_sigma: 0.008,
    halation_tint: [0.9, 0.9, 0.8],
};

/// Kodak Ektachrome 100VS
/// Source: Kodak 2010 Archive
/// ISO: 100
/// RMS: 9 -> Alpha = 0.0081
/// Gamma: 1.25
/// Dmax: 3.6, Dmin: 0.12
/// Resolution: 140 lp/mm
pub const KODAK_EKTACHROME_100VS: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.6,
        gamma: 1.25,
        exposure_offset: 0.20,
    },
    g_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.6,
        gamma: 1.25,
        exposure_offset: 0.20,
    },
    b_curve: SegmentedCurve {
        d_min: 0.12,
        d_max: 3.6,
        gamma: 1.25,
        exposure_offset: 0.20,
    },
    color_matrix: [
        [1.25, -0.1, -0.15],
        [-0.1, 1.25, -0.15],
        [-0.15, -0.1, 1.25],
    ], // Vivid Saturation
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0081,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 140.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [1.0, 0.5, 0.5],
};

/// Fujifilm Neopan ACROS 100
/// Source: Fujifilm 2019 Statement
/// ISO: 100
/// RMS: 7 -> Alpha = 0.0049
/// Gamma: 0.75
/// Dmax: 2.2, Dmin: 0.10
/// Resolution: 160 lp/mm
pub const FUJIFILM_NEOPAN_ACROS_100: FilmStock = FilmStock {
    film_type: FilmType::BwNegative,
    iso: 100.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.2,
        gamma: 0.75,
        exposure_offset: 0.01,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.2,
        gamma: 0.75,
        exposure_offset: 0.01,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.2,
        gamma: 0.75,
        exposure_offset: 0.01,
    },
    color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.0049,
        sigma_read: 0.004,
        monochrome: true,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 160.0,
    reciprocity_beta: 0.05, // Excellent reciprocity
    halation_strength: 0.1,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [0.9, 0.9, 0.9],
};

/// Polaroid SX-70 (Instant Film)
/// Source: Polaroid 2005 Tech Doc
/// ISO: 150
/// Res: 50 lp/mm
/// Dmax: 2.0
pub const POLAROID_SX_70: FilmStock = FilmStock {
    film_type: FilmType::ColorSlide,
    iso: 150.0,
    r_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.0,
        gamma: 0.8,
        exposure_offset: 0.008,
    },
    g_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.0,
        gamma: 0.8,
        exposure_offset: 0.008,
    },
    b_curve: SegmentedCurve {
        d_min: 0.10,
        d_max: 2.0,
        gamma: 0.8,
        exposure_offset: 0.008,
    },
    color_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], // Muted colors
    spectral_params: FilmSpectralParams::new_panchromatic(),
    grain_model: GrainModel {
        alpha: 0.01,
        sigma_read: 0.005,
        monochrome: false,
        blur_radius: 0.5,
    },
    resolution_lp_mm: 50.0,
    reciprocity_beta: 0.05,
    halation_strength: 0.05,
    halation_threshold: 0.9,
    halation_sigma: 0.01,
    halation_tint: [0.95, 0.95, 0.9],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_validity() {
        // 通过 get_all_stocks 获取所有预设，避免手写列表
        let stocks = get_all_stocks();
        // 提取 FilmStock 部分用于验证
        let presets: Vec<FilmStock> = stocks.into_iter().map(|(_, s)| s).collect();

        for (i, preset) in presets.iter().enumerate() {
            assert!(preset.iso > 0.0, "Preset {} ISO must be positive", i);
            assert!(
                preset.r_curve.gamma > 0.0,
                "Preset {} R gamma must be positive",
                i
            );
            assert!(
                preset.g_curve.gamma > 0.0,
                "Preset {} G gamma must be positive",
                i
            );
            assert!(
                preset.b_curve.gamma > 0.0,
                "Preset {} B gamma must be positive",
                i
            );
            assert!(
                preset.grain_model.alpha >= 0.0,
                "Preset {} Grain alpha must be non-negative",
                i
            );
        }
    }
}
