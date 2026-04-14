use crate::film::{FilmStock, FilmType};
use crate::physics;
use crate::processor::{OutputMode, SimulationConfig, WhiteBalanceMode};
use crate::utils;
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use tracing::{debug, info, instrument};
use wide::f32x4;

/// Context shared across all pipeline stages.
/// Contains read-only references to film data and configuration.
pub struct PipelineContext<'a> {
    pub film: &'a FilmStock,
    pub config: &'a SimulationConfig,
}

/// A stage in the image processing pipeline.
/// Modifies the image buffer in place.
pub trait PipelineStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext);
}

/// # Linearize Stage (Initializer)
///
/// Converts sRGB input image to Linear RGB f32 format.
/// Uses a Look-Up Table (LUT) for performance optimization.
#[instrument(skip(input))]
pub fn create_linear_image(input: &RgbImage) -> ImageBuffer<Rgb<f32>, Vec<f32>> {
    debug!("Converting input image to linear space");
    let width = input.width();
    let height = input.height();
    let mut linear_image: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

    // Precompute sRGB to Linear LUT for 8-bit input
    // This provides a significant speedup (instruction level parallelism via LUT)
    let lut: Vec<f32> = (0..=255)
        .map(|i| physics::srgb_to_linear(i as f32 / 255.0))
        .collect();

    linear_image
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(i, pixel)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            let in_pixel = input.get_pixel(x, y);

            // Use LUT for fast conversion
            pixel[0] = lut[in_pixel[0] as usize];
            pixel[1] = lut[in_pixel[1] as usize];
            pixel[2] = lut[in_pixel[2] as usize];
        });
    linear_image
}

/// # Vignetting Stage
///
/// Simulates lens light falloff using cos⁴(θ) model.
/// Applied in exposure space (before develop) so it affects H-D curve naturally.
pub struct VignettingStage;

impl PipelineStage for VignettingStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let strength = context.film.vignette_strength;
        if strength <= 0.0 {
            return;
        }
        info!("Applying vignetting (strength: {:.2})", strength);
        let w = image.width() as f32;
        let h = image.height() as f32;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let r_max = (cx * cx + cy * cy).sqrt();
        // Equivalent focal length: assume 50mm on 36mm frame → f/diagonal ratio
        let f_equiv = r_max * 1.2; // ~50mm equivalent

        let img_w = image.width();
        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i as u32 % img_w) as f32 + 0.5;
            let y = (i as u32 / img_w) as f32 + 0.5;
            let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            let theta = (r / f_equiv).atan();
            let falloff = theta.cos().powi(4);
            // Blend between no vignetting (1.0) and full cos⁴
            let factor = 1.0 - strength * (1.0 - falloff);
            pixel[0] *= factor;
            pixel[1] *= factor;
            pixel[2] *= factor;
        });
    }
}

/// # Halation Stage
///
/// Simulates light reflecting off the film base back into the emulsion.
/// Creates a reddish-orange glow around highlights.
pub struct HalationStage;

impl PipelineStage for HalationStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        if film.halation_strength <= 0.0 {
            debug!("Halation disabled (strength <= 0)");
            return;
        }
        info!("Applying Halation effect");

        let width = image.width();
        let threshold = film.halation_threshold;
        let mut halation_map = image.clone();

        // Apply threshold
        halation_map.par_chunks_mut(3).for_each(|p| {
            let lum = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
            if lum < threshold {
                p[0] = 0.0;
                p[1] = 0.0;
                p[2] = 0.0;
            } else {
                p[0] = (p[0] - threshold).max(0.0);
                p[1] = (p[1] - threshold).max(0.0);
                p[2] = (p[2] - threshold).max(0.0);
            }
        });

        let blur_sigma = width as f32 * film.halation_sigma;
        utils::apply_gaussian_blur(&mut halation_map, blur_sigma);

        let tint = film.halation_tint;
        let strength = film.halation_strength;

        let factor_r = tint[0] * strength;
        let factor_g = tint[1] * strength;
        let factor_b = tint[2] * strength;

        // SIMD constants for RGBRGB... pattern
        let v0 = f32x4::from([factor_r, factor_g, factor_b, factor_r]);
        let v1 = f32x4::from([factor_g, factor_b, factor_r, factor_g]);
        let v2 = f32x4::from([factor_b, factor_r, factor_g, factor_b]);

        // Process 4 pixels (12 floats) at a time to align with SIMD lanes
        image
            .par_chunks_mut(12)
            .zip(halation_map.par_chunks(12))
            .for_each(|(dest, src)| {
                if dest.len() == 12 {
                    // SIMD Path
                    let d0_arr: [f32; 4] = dest[0..4].try_into().unwrap();
                    let s0_arr: [f32; 4] = src[0..4].try_into().unwrap();
                    let d0 = f32x4::from(d0_arr);
                    let s0 = f32x4::from(s0_arr);
                    let r0 = d0 + s0 * v0;
                    dest[0..4].copy_from_slice(&<[f32; 4]>::from(r0));

                    let d1_arr: [f32; 4] = dest[4..8].try_into().unwrap();
                    let s1_arr: [f32; 4] = src[4..8].try_into().unwrap();
                    let d1 = f32x4::from(d1_arr);
                    let s1 = f32x4::from(s1_arr);
                    let r1 = d1 + s1 * v1;
                    dest[4..8].copy_from_slice(&<[f32; 4]>::from(r1));

                    let d2_arr: [f32; 4] = dest[8..12].try_into().unwrap();
                    let s2_arr: [f32; 4] = src[8..12].try_into().unwrap();
                    let d2 = f32x4::from(d2_arr);
                    let s2 = f32x4::from(s2_arr);
                    let r2 = d2 + s2 * v2;
                    dest[8..12].copy_from_slice(&<[f32; 4]>::from(r2));
                } else {
                    // Scalar Fallback
                    for (d, s) in dest.chunks_mut(3).zip(src.chunks(3)) {
                        d[0] += s[0] * factor_r;
                        d[1] += s[1] * factor_g;
                        d[2] += s[2] * factor_b;
                    }
                }
            });
    }
}

/// # MTF (Modulation Transfer Function) Stage
///
/// Simulates optical softness based on the film's resolving power (lp/mm).
/// Applied before grain to simulate the physical blurring of the image.
pub struct MtfStage;

impl PipelineStage for MtfStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        let width = image.width();

        // Assuming 35mm film width = 36mm.
        let pixels_per_mm = width as f32 / 36.0;
        let mtf_sigma = (0.5 / film.resolution_lp_mm) * pixels_per_mm;

        if mtf_sigma > 0.5 {
            info!("Applying MTF blur (sigma: {:.2})", mtf_sigma);
            utils::apply_gaussian_blur(image, mtf_sigma);
        } else {
            debug!("MTF blur skipped (sigma too small: {:.2})", mtf_sigma);
        }
    }
}

/// # Develop Stage
///
/// The core physical simulation:
/// - Spectral Sensitivity (RGB -> Spectrum -> Exposure)
/// - Reciprocity Failure (Exposure Adjustment)
/// - White Balance (Exposure Gain)
/// - H-D Curves (Exposure -> Density)
pub struct DevelopStage;

const SPECTRAL_NORM: f32 = 1.0;

impl PipelineStage for DevelopStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        info!("Developing film (Spectral -> Exposure -> Density)");
        let film = context.film;
        let config = context.config;
        let width = image.width();
        let height = image.height();

        // Precompute Spectral Matrix (3x3)
        // Maps Linear RGB -> Film Layer Exposure directly
        let spectral_matrix = film.compute_spectral_matrix();

        let apply_matrix = |r: f32, g: f32, b: f32| -> [f32; 3] {
            [
                r * spectral_matrix[0][0] + g * spectral_matrix[0][1] + b * spectral_matrix[0][2],
                r * spectral_matrix[1][0] + g * spectral_matrix[1][1] + b * spectral_matrix[1][2],
                r * spectral_matrix[2][0] + g * spectral_matrix[2][1] + b * spectral_matrix[2][2],
            ]
        };

        let reciprocity_factor = if config.exposure_time > 1.0 {
            1.0 + film.reciprocity.beta * config.exposure_time.log10().powi(2)
        } else {
            1.0
        };
        let t_eff = config.exposure_time / reciprocity_factor;

        // White Balance Calculation
        let wb_gains = match config.white_balance_mode {
            WhiteBalanceMode::Auto => {
                let step = (width * height / 1000).max(1);
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut count = 0.0;

                for i in (0..(width * height)).step_by(step as usize) {
                    let x = i % width;
                    let y = i / width;
                    let p = image.get_pixel(x, y).0;

                    let exposure_vals = apply_matrix(p[0], p[1], p[2]);

                    sum_r += exposure_vals[0];
                    sum_g += exposure_vals[1];
                    sum_b += exposure_vals[2];
                    count += 1.0;
                }

                if count > 0.0 {
                    let avg_r = sum_r / count;
                    let avg_g = sum_g / count;
                    let avg_b = sum_b / count;
                    let lum = (avg_r + avg_g + avg_b) / 3.0;
                    let eps = 1e-9;

                    let gain_r = lum / avg_r.max(eps);
                    let gain_g = lum / avg_g.max(eps);
                    let gain_b = lum / avg_b.max(eps);

                    let s = config.white_balance_strength.clamp(0.0, 1.0);
                    let base_gains = [
                        1.0 + (gain_r - 1.0) * s,
                        1.0 + (gain_g - 1.0) * s,
                        1.0 + (gain_b - 1.0) * s,
                    ];

                    // Apply Warmth (Shift R/B)
                    let warmth = config.warmth.clamp(-1.0, 1.0);
                    [
                        base_gains[0] * (1.0 + warmth * 0.1),
                        base_gains[1],
                        base_gains[2] * (1.0 - warmth * 0.1),
                    ]
                } else {
                    [1.0, 1.0, 1.0]
                }
            }
            _ => {
                // Manual/Off mode still supports Warmth
                let warmth = config.warmth.clamp(-1.0, 1.0);
                [1.0 + warmth * 0.1, 1.0, 1.0 - warmth * 0.1]
            }
        };

        // Transform in place: Linear -> Density
        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let _x = (i as u32) % width;
            let _y = (i as u32) / width;
            // Current pixel is Linear RGB
            let lin_pixel = [pixel[0], pixel[1], pixel[2]];

            let exposure_vals = apply_matrix(lin_pixel[0], lin_pixel[1], lin_pixel[2]);

            let r_balanced = exposure_vals[0] * wb_gains[0];
            let g_balanced = exposure_vals[1] * wb_gains[1];
            let b_balanced = exposure_vals[2] * wb_gains[2];

            let r_in = (r_balanced * SPECTRAL_NORM).max(0.0);
            let g_in = (g_balanced * SPECTRAL_NORM).max(0.0);
            let b_in = (b_balanced * SPECTRAL_NORM).max(0.0);

            let r_exp = physics::calculate_exposure(r_in, t_eff);
            let g_exp = physics::calculate_exposure(g_in, t_eff);
            let b_exp = physics::calculate_exposure(b_in, t_eff);

            let epsilon = 1e-6;
            let log_e = [
                r_exp.max(epsilon).log10(),
                g_exp.max(epsilon).log10(),
                b_exp.max(epsilon).log10(),
            ];

            let densities = film.map_log_exposure(log_e);
            pixel[0] = densities[0];
            pixel[1] = densities[1];
            pixel[2] = densities[2];
        });
    }
}

/// # Grain Stage
///
/// Adds film grain noise based on density.
/// Supports both monochrome and color grain models.
pub struct GrainStage;

impl PipelineStage for GrainStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        if !context.config.enable_grain {
            debug!("Grain disabled");
            return;
        }
        info!("Applying Grain noise");
        let film = context.film;
        let width = image.width();
        let height = image.height();
        let gm = &film.grain_model;

        // Physical grain size in pixels (blur_radius in mm-equivalent units)
        let pixels_per_mm = width as f32 / 36.0;
        let grain_sigma = gm.blur_radius * 0.05 * pixels_per_mm; // 0.5 → 25µm → ~1-2px
        let grain_sigma = grain_sigma.max(0.8); // minimum 0.8px for visible structure

        // Generate 3 independent noise textures (one per emulsion layer)
        // + 1 shared luminance texture for channel correlation
        let gen_texture = || -> Vec<f32> {
            let mut tex = vec![0.0f32; (width * height) as usize];
            tex.par_chunks_mut(1).for_each(|p| {
                let mut rng = rand::thread_rng();
                p[0] = rand_distr::Distribution::sample(
                    &rand_distr::Normal::new(0.0f32, 1.0f32).unwrap(),
                    &mut rng,
                );
            });
            tex
        };

        let mut tex_shared = gen_texture();
        let mut tex_r = gen_texture();
        let mut tex_g = gen_texture();
        let mut tex_b = gen_texture();

        // Blur all textures to create spatial grain structure
        // Reuse existing Gaussian blur on temporary ImageBuffers
        let blur_texture = |tex: &mut Vec<f32>, sigma: f32| {
            if sigma < 0.5 {
                return;
            }
            // Wrap as single-channel image (abuse Rgb with same value)
            let mut img: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);
            img.chunks_mut(3).enumerate().for_each(|(i, pixel)| {
                pixel[0] = tex[i];
                pixel[1] = tex[i];
                pixel[2] = tex[i];
            });
            utils::apply_gaussian_blur(&mut img, sigma);
            img.chunks(3).enumerate().for_each(|(i, pixel)| {
                tex[i] = pixel[0];
            });
        };

        blur_texture(&mut tex_shared, grain_sigma);
        blur_texture(&mut tex_r, grain_sigma);
        blur_texture(&mut tex_g, grain_sigma);
        blur_texture(&mut tex_b, grain_sigma);

        // Apply grain: multiplicative modulation of density
        // D_final = D × (1 + strength × texture)
        // strength scales with √D (Selwyn) and preset alpha
        let corr = gm.color_correlation;
        let alpha = gm.alpha;
        let boost = 25.0f32; // visual compensation

        let mono = gm.monochrome;

        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let shared = tex_shared[i];
            let (nr, ng, nb) = if mono {
                (shared, shared, shared)
            } else {
                (
                    corr * shared + (1.0 - corr) * tex_r[i],
                    corr * shared + (1.0 - corr) * tex_g[i],
                    corr * shared + (1.0 - corr) * tex_b[i],
                )
            };

            // Grain strength proportional to √D (Selwyn)
            let str_r = (alpha * pixel[0].max(0.0).sqrt() * boost).sqrt();
            let str_g = (alpha * pixel[1].max(0.0).sqrt() * boost).sqrt();
            let str_b = (alpha * pixel[2].max(0.0).sqrt() * boost).sqrt();

            // Multiplicative: density modulated by grain texture
            pixel[0] = (pixel[0] * (1.0 + str_r * nr)).max(0.0);
            pixel[1] = (pixel[1] * (1.0 + str_g * ng)).max(0.0);
            pixel[2] = (pixel[2] * (1.0 + str_b * nb)).max(0.0);
        });
    }
}

/// # Output Stage (Final Conversion)
///
/// Converts Density to final output color space.
/// - Negative Mode: Simulates transmission light through the negative.
/// - Positive Mode: Simulates scan/inversion for display.
/// - Applies Color Matrix (Crosstalk) and CMY -> RGB conversion.
#[instrument(skip(image, context))]
pub fn create_output_image(
    image: &ImageBuffer<Rgb<f32>, Vec<f32>>,
    context: &PipelineContext,
) -> RgbImage {
    info!("Converting to final output image");
    let width = image.width();
    let height = image.height();
    let film = context.film;
    let config = context.config;

    let map_densities = |densities: [f32; 3]| -> (f32, f32, f32) {
        let net_r = (densities[0] - film.r_curve.d_min).max(0.0);
        let net_g = (densities[1] - film.g_curve.d_min).max(0.0);
        let net_b = (densities[2] - film.b_curve.d_min).max(0.0);
        match config.output_mode {
            OutputMode::Negative => {
                let t_r = physics::apply_dye_self_absorption(
                    net_r,
                    physics::density_to_transmission(net_r),
                );
                let t_g = physics::apply_dye_self_absorption(
                    net_g,
                    physics::density_to_transmission(net_g),
                );
                let t_b = physics::apply_dye_self_absorption(
                    net_b,
                    physics::density_to_transmission(net_b),
                );
                (
                    t_r.clamp(0.0, 1.0),
                    t_g.clamp(0.0, 1.0),
                    t_b.clamp(0.0, 1.0),
                )
            }
            OutputMode::Positive => {
                let t_r = physics::apply_dye_self_absorption(
                    net_r,
                    physics::density_to_transmission(net_r),
                );
                let t_g = physics::apply_dye_self_absorption(
                    net_g,
                    physics::density_to_transmission(net_g),
                );
                let t_b = physics::apply_dye_self_absorption(
                    net_b,
                    physics::density_to_transmission(net_b),
                );
                let t_max = physics::TRANSMISSION_AT_ZERO_DENSITY;
                let t_r_min = physics::density_to_transmission(
                    (film.r_curve.d_max - film.r_curve.d_min).max(0.0),
                );
                let t_g_min = physics::density_to_transmission(
                    (film.g_curve.d_max - film.g_curve.d_min).max(0.0),
                );
                let t_b_min = physics::density_to_transmission(
                    (film.b_curve.d_max - film.b_curve.d_min).max(0.0),
                );
                let norm = |t: f32, t_min: f32, t_max: f32| {
                    let denom = (t_max - t_min).max(1e-6);
                    let n = (t_max - t).clamp(0.0, denom) / denom;
                    let paper_gamma = match film.film_type {
                        FilmType::ColorSlide => 1.5,
                        _ => 2.0,
                    };
                    n.powf(paper_gamma)
                };
                (
                    norm(t_r, t_r_min, t_max),
                    norm(t_g, t_g_min, t_max),
                    norm(t_b, t_b_min, t_max),
                )
            }
        }
    };

    // Extract dye spectra from layer_stack (if available) for spectral output path
    let dye_spectra = film.layer_stack.as_ref().and_then(|stack| {
        use crate::film_layer::{EmulsionChannel, LayerKind};
        let mut yellow = None;
        let mut magenta = None;
        let mut cyan = None;
        for layer in &stack.layers {
            if let LayerKind::Emulsion { channel } = layer.kind {
                if let Some(ref dye) = layer.dye_spectrum {
                    match channel {
                        EmulsionChannel::Blue => yellow = Some(*dye),
                        EmulsionChannel::Green => magenta = Some(*dye),
                        EmulsionChannel::Red => cyan = Some(*dye),
                    }
                }
            }
        }
        match (yellow, magenta, cyan) {
            (Some(y), Some(m), Some(c)) => Some((y, m, c)),
            _ => None,
        }
    });

    // Precompute D65 × CIE XYZ for spectral output (if dye spectra available)
    let spectral_output = dye_spectra.map(|(y_dye, m_dye, c_dye)| {
        use crate::cie_data::{CIE_X, CIE_Y, CIE_Z, D65_SPD, XYZ_TO_SRGB};
        use crate::spectral::{BINS, LAMBDA_STEP};
        // Precompute D65 × CMF
        let mut d65_x = [0.0f32; BINS];
        let mut d65_y = [0.0f32; BINS];
        let mut d65_z = [0.0f32; BINS];
        for i in 0..BINS {
            d65_x[i] = D65_SPD[i] * CIE_X[i] * LAMBDA_STEP as f32;
            d65_y[i] = D65_SPD[i] * CIE_Y[i] * LAMBDA_STEP as f32;
            d65_z[i] = D65_SPD[i] * CIE_Z[i] * LAMBDA_STEP as f32;
        }
        // White point normalization: Y of D65 should = 1.0
        let y_sum: f32 = d65_y.iter().sum();
        let y_norm = if y_sum > 0.0 { 1.0 / y_sum } else { 1.0 };
        (
            y_dye,
            m_dye,
            c_dye,
            d65_x,
            d65_y,
            d65_z,
            y_norm,
            XYZ_TO_SRGB,
        )
    });

    let mut pixels: Vec<u8> = vec![0; (width * height * 3) as usize];

    pixels.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        let d = image.get_pixel(x, y).0;

        let (mut r_lin, mut g_lin, mut b_lin) = if let Some(ref sp) = spectral_output {
            // Full-spectrum dye output path
            let (y_dye, m_dye, c_dye, d65_x, d65_y, d65_z, y_norm, xyz_to_srgb) = sp;
            let net = [
                (d[0] - film.r_curve.d_min).max(0.0), // cyan density (from red layer)
                (d[1] - film.g_curve.d_min).max(0.0), // magenta density (from green layer)
                (d[2] - film.b_curve.d_min).max(0.0), // yellow density (from blue layer)
            ];

            // Per-wavelength: T(λ) = 10^(-Dc×cyan(λ)) × 10^(-Dm×magenta(λ)) × 10^(-Dy×yellow(λ))
            let mut xyz = [0.0f32; 3];
            for i in 0..crate::spectral::BINS {
                let od = net[0] * c_dye[i] + net[1] * m_dye[i] + net[2] * y_dye[i];
                let t = 10.0f32.powf(-od);
                xyz[0] += t * d65_x[i];
                xyz[1] += t * d65_y[i];
                xyz[2] += t * d65_z[i];
            }
            xyz[0] *= y_norm;
            xyz[1] *= y_norm;
            xyz[2] *= y_norm;

            // XYZ → linear sRGB
            let mut r = xyz_to_srgb[0][0] * xyz[0]
                + xyz_to_srgb[0][1] * xyz[1]
                + xyz_to_srgb[0][2] * xyz[2];
            let mut g = xyz_to_srgb[1][0] * xyz[0]
                + xyz_to_srgb[1][1] * xyz[1]
                + xyz_to_srgb[1][2] * xyz[2];
            let mut b = xyz_to_srgb[2][0] * xyz[0]
                + xyz_to_srgb[2][1] * xyz[1]
                + xyz_to_srgb[2][2] * xyz[2];

            // Negative film: invert (high density = bright in print)
            if film.film_type == FilmType::ColorNegative || film.film_type == FilmType::BwNegative {
                r = 1.0 - r;
                g = 1.0 - g;
                b = 1.0 - b;
            }

            (r, g, b)
        } else {
            // Legacy per-channel output path
            map_densities(d)
        };

        // Apply Saturation
        if config.saturation != 1.0 {
            let lum = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
            r_lin = lum + (r_lin - lum) * config.saturation;
            g_lin = lum + (g_lin - lum) * config.saturation;
            b_lin = lum + (b_lin - lum) * config.saturation;
        }

        // Vignetting in output linear space
        let v_str = film.vignette_strength;
        if v_str > 0.0 {
            let px = i as u32 % width;
            let py = i as u32 / width;
            let dx = (px as f32 + 0.5) / width as f32 - 0.5;
            let dy = (py as f32 + 0.5) / height as f32 - 0.5;
            let r2 = dx * dx + dy * dy;
            // Simplified cos⁴: cos²(θ) ≈ 1/(1 + r²/f²), cos⁴ ≈ 1/(1+r²/f²)²
            // f² chosen so corner falloff ≈ 20-30% at strength=1.0
            let f2 = 0.2; // stronger falloff for visible vignette
            let cos4 = 1.0 / (1.0 + r2 / f2).powi(2);
            let factor = 1.0 - v_str * (1.0 - cos4);
            r_lin *= factor;
            g_lin *= factor;
            b_lin *= factor;
        }

        let r_out = physics::linear_to_srgb(r_lin.clamp(0.0, 1.0));
        let g_out = physics::linear_to_srgb(g_lin.clamp(0.0, 1.0));
        let b_out = physics::linear_to_srgb(b_lin.clamp(0.0, 1.0));

        chunk[0] = (r_out * 255.0).round() as u8;
        chunk[1] = (g_out * 255.0).round() as u8;
        chunk[2] = (b_out * 255.0).round() as u8;
    });

    RgbImage::from_raw(width, height, pixels).unwrap()
}
