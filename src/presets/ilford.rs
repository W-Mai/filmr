//! Ilford film stock presets

#![allow(non_snake_case)]

use crate::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use crate::film_layer::*;
use crate::grain::GrainModel;
use crate::spectral::{FilmSpectralParams, BINS};

/// Ilford HP5 Plus 400 (Professional B&W)
/// Source: Ilford 2021 Data Sheet
/// ISO: 400
/// RMS: 16 -> Alpha = 0.000256
/// Gamma: 0.65
/// Dmax: 2.1, Dmin: 0.10
/// Resolution: 95 lp/mm
pub fn HP5_PLUS_400() -> FilmStock {
    FilmStock {
        manufacturer: "Ilford".to_string(),
        name: "HP5 Plus 400".to_string(),
        film_type: FilmType::BwNegative,
        iso: 400.0,
        r_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.1,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 34.22952,
        },
        g_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.1,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 34.22952,
        },
        b_curve: SegmentedCurve {
            d_min: 0.10,
            d_max: 2.1,
            gamma: 0.65,
            shoulder_point: 0.8,
            exposure_offset: 34.22952,
        },
        color_matrix: [[0.33, 0.33, 0.33], [0.33, 0.33, 0.33], [0.33, 0.33, 0.33]],
        spectral_params: FilmSpectralParams::new_panchromatic(),
        grain_model: GrainModel {
            alpha: 0.000256,
            sigma_read: 0.006,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.5,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 95.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.18,
        halation_threshold: 0.83,
        halation_sigma: 0.015,
        halation_tint: [0.88, 0.88, 0.88],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 9.0,
                    refractive_index: 1.54,
                    absorption: gaussian_absorption(540.0, 95.0, 0.08),
                    scattering: 0.040,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
    }
}

/// Ilford HP5 Plus 400 - Artistic (Classic documentary look)
/// Based on HP5 Plus with enhanced grain and contrast
pub fn HP5_PLUS_400_ARTISTIC() -> FilmStock {
    let mut stock = HP5_PLUS_400();

    // Increased contrast (classic pushed HP5 look)
    stock.r_curve.gamma = 0.75;
    stock.g_curve.gamma = 0.75;
    stock.b_curve.gamma = 0.75;

    // More prominent grain
    stock.grain_model.alpha *= 1.7;
    stock.grain_model.roughness = 0.65;

    // Enhanced halation
    stock.halation_strength = 0.25;
    stock.halation_sigma = 0.018;

    stock.name = "HP5 Plus 400 Artistic".to_string();
    stock
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
            alpha: 0.000066,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 140.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [0.92, 0.92, 0.92],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 7.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 85.0, 0.10),
                    scattering: 0.025,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
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
            alpha: 0.000146,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.45,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 130.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [0.90, 0.90, 0.90],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 7.5,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 90.0, 0.09),
                    scattering: 0.020,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
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
            alpha: 0.000041,
            sigma_read: 0.004,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.35,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 150.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.12,
        halation_threshold: 0.88,
        halation_sigma: 0.012,
        halation_tint: [0.94, 0.94, 0.94],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 6.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 80.0, 0.11),
                    scattering: 0.015,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
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
            alpha: 0.000013,
            sigma_read: 0.003,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.25,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 170.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.10,
        halation_threshold: 0.90,
        halation_sigma: 0.010,
        halation_tint: [0.96, 0.96, 0.96],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 5.0,
                    refractive_index: 1.52,
                    absorption: gaussian_absorption(540.0, 75.0, 0.13),
                    scattering: 0.012,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
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
            alpha: 0.000100,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 125.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.14,
        halation_threshold: 0.86,
        halation_sigma: 0.014,
        halation_tint: [0.92, 0.92, 0.92],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 8.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 90.0, 0.09),
                    scattering: 0.030,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
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
            alpha: 0.000100,
            sigma_read: 0.005,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.4,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 130.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.16,
        halation_threshold: 0.84,
        halation_sigma: 0.015,
        halation_tint: [0.90, 0.90, 0.90],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 7.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 100.0, 0.09),
                    scattering: 0.025,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
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
            alpha: 0.000041,
            sigma_read: 0.004,
            monochrome: true,
            blur_radius: 0.5,
            roughness: 0.35,
            color_correlation: 0.8,
            shadow_noise: 0.001,
            highlight_coarseness: 0.05,
        },
        resolution_lp_mm: 140.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.13,
        halation_threshold: 0.87,
        halation_sigma: 0.013,
        halation_tint: [0.93, 0.93, 0.93],
        layer_stack: Some(FilmLayerStack {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: [0.0; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 6.5,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 60.0, 0.12),
                    scattering: 0.020,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 110.0, 0.45),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 127.0,
                    refractive_index: 1.65,
                    absorption: [0.001; BINS],
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }),
    }
}

/// Get all Ilford film stocks
pub fn get_stocks() -> Vec<FilmStock> {
    vec![
        HP5_PLUS_400(),
        HP5_PLUS_400_ARTISTIC(),
        FP4_PLUS_125(),
        PAN_F_PLUS_50(),
        DELTA_100_PROFESSIONAL(),
        DELTA_400_PROFESSIONAL(),
        ORTHO_PLUS_80(),
        XP2_SUPER_400(),
        SFX_200(),
    ]
}
