use crate::film::{FilmStock, FilmType};
use crate::physics;
use crate::processor::{OutputMode, SimulationConfig, WhiteBalanceMode};
use crate::spectral::{CameraSensitivities, Spectrum};
use crate::utils;
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;

pub struct PipelineContext<'a> {
    pub film: &'a FilmStock,
    pub config: &'a SimulationConfig,
}

pub trait PipelineStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext);
}

// 1. Linearize (Not a stage per se, but an initializer)
pub fn create_linear_image(input: &RgbImage) -> ImageBuffer<Rgb<f32>, Vec<f32>> {
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

// 2. Halation Stage
pub struct HalationStage;

impl PipelineStage for HalationStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        if film.halation_strength <= 0.0 {
            return;
        }

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

        image
            .par_chunks_mut(3)
            .zip(halation_map.par_chunks(3))
            .for_each(|(dest, src)| {
                dest[0] += src[0] * tint[0] * strength;
                dest[1] += src[1] * tint[1] * strength;
                dest[2] += src[2] * tint[2] * strength;
            });
    }
}

// 3. MTF Stage
pub struct MtfStage;

impl PipelineStage for MtfStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        let width = image.width();

        // Assuming 35mm film width = 36mm.
        let pixels_per_mm = width as f32 / 36.0;
        let mtf_sigma = (0.5 / film.resolution_lp_mm) * pixels_per_mm;

        if mtf_sigma > 0.5 {
            utils::apply_gaussian_blur(image, mtf_sigma);
        }
    }
}

// 4. Develop Stage (Spectral -> Log Exposure -> Density)
pub struct DevelopStage;

const SPECTRAL_NORM: f32 = 1.0;

impl PipelineStage for DevelopStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
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

                    let scene_spectrum = apply_illuminant(camera_sens.uplift(p[0], p[1], p[2]));
                    let exposure_vals = film_sens.expose(&scene_spectrum);

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
                    [
                        1.0 + (gain_r - 1.0) * s,
                        1.0 + (gain_g - 1.0) * s,
                        1.0 + (gain_b - 1.0) * s,
                    ]
                } else {
                    [1.0, 1.0, 1.0]
                }
            }
            _ => [1.0, 1.0, 1.0],
        };

        // Transform in place: Linear -> Density
        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let _x = (i as u32) % width;
            let _y = (i as u32) / width;
            // Current pixel is Linear RGB
            let lin_pixel = [pixel[0], pixel[1], pixel[2]];

            let scene_spectrum =
                apply_illuminant(camera_sens.uplift(lin_pixel[0], lin_pixel[1], lin_pixel[2]));
            let exposure_vals = film_sens.expose(&scene_spectrum);

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

// 5. Grain Stage
pub struct GrainStage;

impl PipelineStage for GrainStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        if !context.config.enable_grain {
            return;
        }
        let film = context.film;
        let width = image.width();
        let height = image.height();

        let mut grain_noise: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

        grain_noise
            .par_chunks_mut(3)
            .enumerate()
            .for_each(|(i, pixel)| {
                let x = (i as u32) % width;
                let y = (i as u32) / width;
                let densities = image.get_pixel(x, y).0;
                let mut rng = rand::thread_rng();

                if film.grain_model.monochrome {
                    let d = densities[1];
                    let noise = film.grain_model.sample_noise(d, &mut rng);
                    pixel[0] = noise;
                    pixel[1] = noise;
                    pixel[2] = noise;
                } else {
                    let n_r = film.grain_model.sample_noise(densities[0], &mut rng);
                    let n_g = film.grain_model.sample_noise(densities[1], &mut rng);
                    let n_b = film.grain_model.sample_noise(densities[2], &mut rng);

                    let n_lum = (n_r + n_g + n_b) / 3.0;
                    let chroma_scale = 0.3;

                    pixel[0] = n_lum + (n_r - n_lum) * chroma_scale;
                    pixel[1] = n_lum + (n_g - n_lum) * chroma_scale;
                    pixel[2] = n_lum + (n_b - n_lum) * chroma_scale;
                }
            });

        if film.grain_model.blur_radius > 0.0 {
            utils::apply_gaussian_blur(&mut grain_noise, film.grain_model.blur_radius);
        }

        image
            .pixels_mut()
            .zip(grain_noise.pixels())
            .for_each(|(d, n)| {
                d[0] = (d[0] + n[0]).max(0.0);
                d[1] = (d[1] + n[1]).max(0.0);
                d[2] = (d[2] + n[2]).max(0.0);
            });
    }
}

// 6. Output (Final Conversion)
pub fn create_output_image(
    image: &ImageBuffer<Rgb<f32>, Vec<f32>>,
    context: &PipelineContext,
) -> RgbImage {
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

        let (r_lin, g_lin, b_lin) = map_densities(d);

        let r_out = physics::linear_to_srgb(r_lin.clamp(0.0, 1.0));
        let g_out = physics::linear_to_srgb(g_lin.clamp(0.0, 1.0));
        let b_out = physics::linear_to_srgb(b_lin.clamp(0.0, 1.0));

        chunk[0] = (r_out * 255.0).round() as u8;
        chunk[1] = (g_out * 255.0).round() as u8;
        chunk[2] = (b_out * 255.0).round() as u8;
    });

    RgbImage::from_raw(width, height, pixels).unwrap()
}
