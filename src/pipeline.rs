use crate::film::{FilmStock, FilmType};
use crate::physics;
use crate::processor::{OutputMode, SimulationConfig, WhiteBalanceMode};
use crate::spectral::{CameraSensitivities, Spectrum};
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

        let camera_sens = CameraSensitivities::srgb();
        let mut film_sens = film.get_spectral_sensitivities();
        let illuminant = Spectrum::new_d65();
        let apply_illuminant = |s: Spectrum| s.multiply(&illuminant);

        let system_white = apply_illuminant(camera_sens.uplift(1.0, 1.0, 1.0));
        film_sens.calibrate_to_white_point(&system_white);

        // Precompute Spectral Matrix (3x3)
        // Maps Linear RGB -> Film Layer Exposure directly
        // This avoids per-pixel full spectrum integration (~600 FLOPS -> 15 FLOPS)
        let r_cam = camera_sens.r_curve.multiply(&illuminant);
        let g_cam = camera_sens.g_curve.multiply(&illuminant);
        let b_cam = camera_sens.b_curve.multiply(&illuminant);

        let m00 = film_sens.r_sensitivity.integrate_product(&r_cam) * film_sens.r_factor;
        let m01 = film_sens.r_sensitivity.integrate_product(&g_cam) * film_sens.r_factor;
        let m02 = film_sens.r_sensitivity.integrate_product(&b_cam) * film_sens.r_factor;

        let m10 = film_sens.g_sensitivity.integrate_product(&r_cam) * film_sens.g_factor;
        let m11 = film_sens.g_sensitivity.integrate_product(&g_cam) * film_sens.g_factor;
        let m12 = film_sens.g_sensitivity.integrate_product(&b_cam) * film_sens.g_factor;

        let m20 = film_sens.b_sensitivity.integrate_product(&r_cam) * film_sens.b_factor;
        let m21 = film_sens.b_sensitivity.integrate_product(&g_cam) * film_sens.b_factor;
        let m22 = film_sens.b_sensitivity.integrate_product(&b_cam) * film_sens.b_factor;

        // Helper to apply matrix
        let apply_matrix = |r: f32, g: f32, b: f32| -> [f32; 3] {
            [
                r * m00 + g * m01 + b * m02,
                r * m10 + g * m11 + b * m12,
                r * m20 + g * m21 + b * m22,
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

        // Scale grain parameters based on resolution
        // Reference: 2048px width (approx 2K scan)
        const REFERENCE_WIDTH: f32 = 2048.0;
        let scale_factor = width as f32 / REFERENCE_WIDTH;

        // Calculate System PSF sigma (Optical Resolution Limit)
        // System Resolution (lp/mm) -> Sigma (pixels)
        // 1 cycle = 2 pixels (Nyquist), but Gaussian sigma ~ 0.5 / freq
        let pixels_per_mm = width as f32 / 36.0f32; // Assuming 35mm width

        // Model the full optical chain: Film + Lens + Scanner
        // If we only use film.resolution, we assume perfect lens/scanner, which yields too sharp grain.
        // Assume a "Standard System" limit of ~40 lp/mm (Typical flatbed scanner or kit lens)
        // This avoids the "Digital Noise" look (too sharp) and the "Blurry Mess" look (too soft)
        let system_limit_lp_mm = 40.0f32;
        let effective_lp_mm = (1.0f32 / film.resolution_lp_mm.powi(2)
            + 1.0f32 / system_limit_lp_mm.powi(2))
        .sqrt()
        .recip();

        let system_sigma = (0.5f32 / effective_lp_mm) * pixels_per_mm;

        // Effective Blur = Intrinsic Grain Blur (scaled) + System Optical Blur
        // Convolving two Gaussians: sigma_total = sqrt(sigma1^2 + sigma2^2)
        let intrinsic_blur = film.grain_model.blur_radius * scale_factor;
        let effective_blur = (intrinsic_blur.powi(2) + system_sigma.powi(2)).sqrt();

        // Highlight coarseness scales the "coarse" grain layer
        // Coarse layer is also subject to system blur
        // Coarse clumps are physically ~2.5x the size of individual crystals
        let intrinsic_coarse_blur = intrinsic_blur * 5.0f32;
        let coarse_blur = (intrinsic_coarse_blur.powi(2) + system_sigma.powi(2)).sqrt();

        // Scale noise amplitude to maintain perceived granularity density
        // Var = alpha * D^1.5 + sigma^2
        // We want std_dev to scale linearly with resolution scale (to counter averaging)
        // So Variance scales with square of resolution scale
        let mut fine_model = film.grain_model;
        let mut coarse_model = film.grain_model;

        // Selwyn's Law Compensation / Contrast Reduction
        // Standard dampening to account for averaging
        // Dampening factor adjusted to 0.25 * sigma for a balanced look
        let dampening = 1.0f32 / (1.0f32 + 0.35f32 * system_sigma);

        // Fine Grain (Shadows/Mids): Heavily dampened by system blur
        fine_model.alpha *= scale_factor * scale_factor * dampening;
        fine_model.sigma_read *= scale_factor * dampening.sqrt();
        fine_model.shadow_noise *= scale_factor * scale_factor * dampening;

        // Coarse Grain (Highlights): Less dampened to retain structure
        // Large clumps survive the blur better, so we use a gentler dampening
        // or just scale with resolution without the heavy Selwyn penalty
        let coarse_dampening = 1.0f32 / (1.0f32 + 0.1f32 * system_sigma);
        coarse_model.alpha *= scale_factor * scale_factor * coarse_dampening;
        coarse_model.sigma_read *= scale_factor * coarse_dampening.sqrt();
        // Coarse layer doesn't really have shot noise (that's fine scale), so zero it out
        coarse_model.shadow_noise = 0.0;

        let mut grain_noise: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);
        let mut coarse_noise: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

        // Function to generate noise pixel
        let generate_pixel_noise = |d: [f32; 3],
                                    model: &crate::grain::GrainModel,
                                    rng: &mut rand::rngs::ThreadRng|
         -> [f32; 3] {
            if model.monochrome {
                let noise = model.sample_noise(d[1], rng);
                [noise, noise, noise]
            } else {
                let d_lum = 0.2126 * d[0] + 0.7152 * d[1] + 0.0722 * d[2];
                let n_shared = model.sample_noise(d_lum, rng);
                let n_r = model.sample_noise(d[0], rng);
                let n_g = model.sample_noise(d[1], rng);
                let n_b = model.sample_noise(d[2], rng);
                let corr = model.color_correlation;
                [
                    corr * n_shared + (1.0 - corr) * n_r,
                    corr * n_shared + (1.0 - corr) * n_g,
                    corr * n_shared + (1.0 - corr) * n_b,
                ]
            }
        };

        // 1. Generate Fine Noise (Base + Shot Noise)
        grain_noise
            .par_chunks_mut(3)
            .enumerate()
            .for_each(|(i, pixel)| {
                let x = (i as u32) % width;
                let y = (i as u32) / width;
                let densities = image.get_pixel(x, y).0;
                let mut rng = rand::thread_rng();
                let noise = generate_pixel_noise(densities, &fine_model, &mut rng);
                pixel[0] = noise[0];
                pixel[1] = noise[1];
                pixel[2] = noise[2];
            });

        // 2. Generate Coarse Noise (Clumps)
        // Only if highlight coarseness > 0
        if film.grain_model.highlight_coarseness > 0.0 {
            coarse_noise
                .par_chunks_mut(3)
                .enumerate()
                .for_each(|(i, pixel)| {
                    let x = (i as u32) % width;
                    let y = (i as u32) / width;
                    let densities = image.get_pixel(x, y).0;
                    let mut rng = rand::thread_rng();
                    let noise = generate_pixel_noise(densities, &coarse_model, &mut rng);
                    pixel[0] = noise[0];
                    pixel[1] = noise[1];
                    pixel[2] = noise[2];
                });
        }

        // Blur passes
        if effective_blur > 0.0 {
            utils::apply_gaussian_blur(&mut grain_noise, effective_blur);
        }
        if film.grain_model.highlight_coarseness > 0.0 && coarse_blur > 0.0 {
            utils::apply_gaussian_blur(&mut coarse_noise, coarse_blur);
        }

        // Mix
        image
            .pixels_mut()
            .zip(grain_noise.pixels())
            .zip(coarse_noise.pixels())
            .for_each(|((d, n_fine), n_coarse)| {
                // Calculate blend factor based on density
                // High density -> More clumps
                // We use Green channel as Luma proxy
                let luma_d = d[1];

                // Sigmoid Mixing Function for Sharp Transition
                // "Big Family" (Clumps) only shows up at high density
                // Center at D=1.0, Slope=4.0
                let sigmoid = |x: f32, center: f32, slope: f32| -> f32 {
                    1.0 / (1.0 + (-slope * (x - center)).exp())
                };

                // Clump Intensity:
                // Base factor from presets
                // Multiplied by Sigmoid(D)
                // Dmax is typically ~2.5-3.0. We want clumps to start appearing around D=0.8 and max out at D=1.5+
                let clump_mix = sigmoid(luma_d, 1.2, 5.0) * film.grain_model.highlight_coarseness;

                // Shoulder Compression (D-Log E)
                // If D is extremely high (> 2.5), contrast might drop (User's exception)
                // But for now we stick to the main physics: High D = Visible Clumps

                let final_noise_r = n_fine[0] + n_coarse[0] * clump_mix;
                let final_noise_g = n_fine[1] + n_coarse[1] * clump_mix;
                let final_noise_b = n_fine[2] + n_coarse[2] * clump_mix;

                d[0] = (d[0] + final_noise_r).max(0.0);
                d[1] = (d[1] + final_noise_g).max(0.0);
                d[2] = (d[2] + final_noise_b).max(0.0);
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
                let t_r_max = physics::density_to_transmission(0.0);
                let t_g_max = physics::density_to_transmission(0.0);
                let t_b_max = physics::density_to_transmission(0.0);
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
                    norm(t_r, t_r_min, t_r_max),
                    norm(t_g, t_g_min, t_g_max),
                    norm(t_b, t_b_min, t_b_max),
                )
            }
        }
    };

    let mut pixels: Vec<u8> = vec![0; (width * height * 3) as usize];

    pixels.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        let d = image.get_pixel(x, y).0;

        let (mut r_lin, mut g_lin, mut b_lin) = map_densities(d);

        // Apply Saturation
        if config.saturation != 1.0 {
            let lum = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
            r_lin = lum + (r_lin - lum) * config.saturation;
            g_lin = lum + (g_lin - lum) * config.saturation;
            b_lin = lum + (b_lin - lum) * config.saturation;
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
