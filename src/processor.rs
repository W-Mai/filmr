use crate::film::FilmStock;
use crate::physics;
use crate::spectral::{CameraSensitivities, Spectrum};
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;

/// Configuration for the simulation run.
pub struct SimulationConfig {
    pub exposure_time: f32, // t in E = I * t
    pub enable_grain: bool,
    pub output_mode: OutputMode,
    pub white_balance_mode: WhiteBalanceMode,
    pub white_balance_strength: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    Negative, // Transmission of the negative (Dark -> Bright, Bright -> Dark)
    Positive, // Scanned/Inverted Positive (Dark -> Dark, Bright -> Bright)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhiteBalanceMode {
    Auto,
    Gray,
    White,
    Off,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            exposure_time: 1.0,
            enable_grain: true,
            output_mode: OutputMode::Positive, // Default to what users expect
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
        }
    }
}

const SPECTRAL_NORM: f32 = 1.0;

pub fn estimate_exposure_time(input: &RgbImage, film: &FilmStock) -> f32 {
    let camera_sens = CameraSensitivities::srgb();
    let mut film_sens = film.get_spectral_sensitivities();
    let illuminant = Spectrum::new_d65();
    let apply_illuminant = |s: Spectrum| s.multiply(&illuminant);
    
    // CALIBRATION: Ensure consistency with process_image
    let system_white = apply_illuminant(camera_sens.uplift(1.0, 1.0, 1.0));
    film_sens.calibrate_to_white_point(&system_white);
    
    let total = (input.width() * input.height()) as usize;
    let max_samples = 20000usize;
    let step = (total / max_samples).max(1);
    let mut samples = Vec::with_capacity((total / step).max(1));
    for (i, p) in input.pixels().enumerate() {
        if i % step != 0 {
            continue;
        }
        let r = physics::srgb_to_linear(p[0] as f32 / 255.0);
        let g = physics::srgb_to_linear(p[1] as f32 / 255.0);
        let b = physics::srgb_to_linear(p[2] as f32 / 255.0);
        let scene_spectrum = apply_illuminant(camera_sens.uplift(r, g, b));
        let exposure_vals = film_sens.expose(&scene_spectrum);
        let r_in = (exposure_vals[0] * SPECTRAL_NORM).max(0.0);
        let g_in = (exposure_vals[1] * SPECTRAL_NORM).max(0.0);
        let b_in = (exposure_vals[2] * SPECTRAL_NORM).max(0.0);
        if r_in > 0.0 || g_in > 0.0 || b_in > 0.0 {
            samples.push([r_in, g_in, b_in]);
        }
    }
    if samples.is_empty() {
        return 1.0;
    }
    let mut log_sum = 0.0f32;
    let mut count = 0.0f32;
    for s in &samples {
        let lum = (s[0] + s[1] + s[2]) / 3.0;
        if lum > 0.0 {
            log_sum += lum.ln();
            count += 1.0;
        }
    }
    if count == 0.0 {
        return 1.0;
    }
    let log_avg = (log_sum / count).exp();
    let exposure_offset_avg = (film.r_curve.exposure_offset
        + film.g_curve.exposure_offset
        + film.b_curve.exposure_offset)
        / 3.0;
    let iso_norm = (100.0 / film.iso).clamp(0.1, 10.0);
    let base_exposure = exposure_offset_avg / log_avg;
    let t_base = (base_exposure * iso_norm).max(1.0e-6);
    let map_densities = |densities: [f32; 3]| -> (f32, f32, f32) {
        let net_r = (densities[0] - film.r_curve.d_min).max(0.0);
        let net_g = (densities[1] - film.g_curve.d_min).max(0.0);
        let net_b = (densities[2] - film.b_curve.d_min).max(0.0);
        let t_r = physics::density_to_transmission(net_r);
        let t_g = physics::density_to_transmission(net_g);
        let t_b = physics::density_to_transmission(net_b);
        let t_r_max = physics::density_to_transmission(0.0);
        let t_g_max = physics::density_to_transmission(0.0);
        let t_b_max = physics::density_to_transmission(0.0);
        let t_r_min =
            physics::density_to_transmission((film.r_curve.d_max - film.r_curve.d_min).max(0.0));
        let t_g_min =
            physics::density_to_transmission((film.g_curve.d_max - film.g_curve.d_min).max(0.0));
        let t_b_min =
            physics::density_to_transmission((film.b_curve.d_max - film.b_curve.d_min).max(0.0));
        let norm = |t: f32, t_min: f32, t_max: f32| {
            let denom = (t_max - t_min).max(1.0e-6);
            (t_max - t).clamp(0.0, denom) / denom
        };
        (
            norm(t_r, t_r_min, t_r_max),
            norm(t_g, t_g_min, t_g_max),
            norm(t_b, t_b_min, t_b_max),
        )
    };
    let target_mid: f32 = 0.18;
    let target_hi: f32 = 0.70;
    let target_lo: f32 = 0.05;
    let mut t_min: f32 = (t_base / 64.0).max(1.0e-4);
    let mut t_max: f32 = (t_base * 8.0).min(4.0);
    if t_max <= t_min {
        t_max = (t_min * 2.0).min(4.0);
    }
    for _ in 0..8 {
        let t = 0.5 * (t_min + t_max);
        
        // Reciprocity Failure Correction for Estimation
        // E_film = E_actual / (1 + beta * log10(t)^2)
        // We use t as E_actual (assuming I=1).
        let t_eff = if t > 1.0 {
            let factor = 1.0 + film.reciprocity_beta * t.log10().powi(2);
            t / factor
        } else {
            t
        };

        let mut lum = Vec::with_capacity(samples.len());
        for s in &samples {
            let r = (s[0] * t_eff).max(1.0e-6).log10();
            let g = (s[1] * t_eff).max(1.0e-6).log10();
            let b = (s[2] * t_eff).max(1.0e-6).log10();
            let densities = film.map_log_exposure([r, g, b]);
            let (r_lin, g_lin, b_lin) = map_densities(densities);
            lum.push(0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin);
        }
        lum.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = lum.len();
        let p10 = lum[((len - 1) as f32 * 0.1).round() as usize];
        let p50 = lum[((len - 1) as f32 * 0.5).round() as usize];
        let p90 = lum[((len - 1) as f32 * 0.9).round() as usize];
        if p90 > target_hi {
            t_max = t;
            continue;
        }
        if p10 < target_lo {
            t_min = t;
            continue;
        }
        if p50 > target_mid {
            t_max = t;
        } else {
            t_min = t;
        }
    }
    (0.5 * (t_min + t_max)).clamp(0.001, 4.0)
}

/// Helper to apply Box Blur (Approximates Gaussian when repeated)
/// Uses a sliding window (Integral Image / Moving Average) approach for O(1) per pixel independent of radius.
fn apply_box_blur(image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, radius: u32) {
    let width = image.width();
    let height = image.height();
    let r = radius as i32;

    // We process each channel independently
    // Horizontal Pass
    let mut temp = image.clone(); // Need temp buffer for separable pass

    // Use Rayon for parallel processing of rows
    // Horizontal Pass using par_chunks_mut on raw buffer
    let raw_buffer = temp.as_mut();
    raw_buffer
        .par_chunks_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, row_slice)| {
            let y = y as u32;
            // Sliding window sum
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            // Initialize window
            // Left padding (pixel at x=0)
            let first_r = row_slice[0];
            let first_g = row_slice[1];
            let first_b = row_slice[2];

            for _ in 0..=r {
                sum_r += first_r;
                sum_g += first_g;
                sum_b += first_b;
            }
            // Right side of window
            for x in 1..=r {
                let idx = (x.min((width - 1).try_into().unwrap()) as usize) * 3;
                sum_r += row_slice[idx];
                sum_g += row_slice[idx + 1];
                sum_b += row_slice[idx + 2];
            }

            for x in 0..width {
                let count = (2 * r + 1) as f32;
                let current_idx = (x as usize) * 3;

                // Write to `temp` (row_slice)
                row_slice[current_idx] = sum_r / count;
                row_slice[current_idx + 1] = sum_g / count;
                row_slice[current_idx + 2] = sum_b / count;

                // Slide window: subtract left-out, add incoming-in
                let left_x = (x as i32 - r).clamp(0, (width - 1) as i32) as u32;
                let right_x = (x as i32 + r + 1).clamp(0, (width - 1) as i32) as u32;

                // Read from source `image`
                let p_out = image.get_pixel(left_x, y).0;
                let p_in = image.get_pixel(right_x, y).0;

                sum_r += p_in[0] - p_out[0];
                sum_g += p_in[1] - p_out[1];
                sum_b += p_in[2] - p_out[2];
            }
        });

    // Vertical Pass: Transpose -> Horizontal Blur -> Transpose
    // 1. Transpose temp -> image (swapped dimensions)
    let mut transposed: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(height, width);
    // Use raw buffer parallel iteration for transpose
    // temp is width x height, transposed is height x width
    let transposed_raw = transposed.as_mut();

    // Parallelize by rows of the OUTPUT (transposed) image
    transposed_raw
        .par_chunks_mut((height * 3) as usize)
        .enumerate()
        .for_each(|(y_t, row_t)| {
            for x_t in 0..height {
                let src_pixel = temp.get_pixel(y_t as u32, x_t).0;

                let idx = (x_t as usize) * 3;
                row_t[idx] = src_pixel[0];
                row_t[idx + 1] = src_pixel[1];
                row_t[idx + 2] = src_pixel[2];
            }
        });

    // 2. Horizontal Blur on Transposed
    // This is essentially the vertical blur of the original image
    let mut transposed_blurred = transposed.clone();

    let raw_blurred_transposed = transposed_blurred.as_mut();
    // Iterating over rows of transposed image
    raw_blurred_transposed
        .par_chunks_mut((height * 3) as usize)
        .enumerate()
        .for_each(|(y, row_slice)| {
            // y is row index in transposed image
            // width_t is width of transposed image (= original height)
            let width_t = height;

            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            // Initialize window
            let first_r = row_slice[0];
            let first_g = row_slice[1];
            let first_b = row_slice[2];

            for _ in 0..=r {
                sum_r += first_r;
                sum_g += first_g;
                sum_b += first_b;
            }
            for x in 1..=r {
                let idx = (x.min((width_t - 1).try_into().unwrap()) as usize) * 3;
                sum_r += row_slice[idx];
                sum_g += row_slice[idx + 1];
                sum_b += row_slice[idx + 2];
            }

            for x in 0..width_t {
                let count = (2 * r + 1) as f32;
                let current_idx = (x as usize) * 3;

                row_slice[current_idx] = sum_r / count;
                row_slice[current_idx + 1] = sum_g / count;
                row_slice[current_idx + 2] = sum_b / count;

                let left_idx = (x as i32 - r).clamp(0, (width_t - 1) as i32) as u32;
                let right_idx = (x as i32 + r + 1).clamp(0, (width_t - 1) as i32) as u32;

                let p_out = transposed.get_pixel(left_idx, y as u32).0;
                let p_in = transposed.get_pixel(right_idx, y as u32).0;

                sum_r += p_in[0] - p_out[0];
                sum_g += p_in[1] - p_out[1];
                sum_b += p_in[2] - p_out[2];
            }
        });

    // 3. Transpose back: transposed_blurred -> image
    let raw_buffer = image.as_mut();
    // image is width x height
    raw_buffer
        .par_chunks_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, row)| {
            // y is row index in image (0..height)
            for x in 0..width {
                // Target(x, y) = Source(y, x)
                // Source is transposed_blurred
                let src_pixel = transposed_blurred.get_pixel(y as u32, x).0;

                let idx = (x as usize) * 3;
                row[idx] = src_pixel[0];
                row[idx + 1] = src_pixel[1];
                row[idx + 2] = src_pixel[2];
            }
        });
}

/// Helper to apply Gaussian blur (Approx) using 3 Box Blurs
fn apply_gaussian_blur(image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, sigma: f32) {
    // 3 passes of Box Blur approximates Gaussian Blur very well (Central Limit Theorem)
    // w = sqrt(12 * sigma^2 / n + 1)
    // radius = (w - 1) / 2
    let n = 3.0;
    let w = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let radius = ((w - 1.0) / 2.0).floor() as u32;
    let radius = radius.max(1);

    for _ in 0..3 {
        apply_box_blur(image, radius);
    }
}

/// Main processor function.
/// Takes an input image and film parameters, returns the simulated image.
pub fn process_image(input: &RgbImage, film: &FilmStock, config: &SimulationConfig) -> RgbImage {
    let width = input.width();
    let height = input.height();

    // 1. Pre-process: Linearize Input
    // We need a float buffer for intermediate calculations (Exposure, Halation)
    let mut linear_image: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

    // Parallelize linear conversion
    linear_image
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(i, pixel)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            let in_pixel = input.get_pixel(x, y);

            pixel[0] = physics::srgb_to_linear(in_pixel[0] as f32 / 255.0);
            pixel[1] = physics::srgb_to_linear(in_pixel[1] as f32 / 255.0);
            pixel[2] = physics::srgb_to_linear(in_pixel[2] as f32 / 255.0);
        });

    // 2. Simulate Halation (Diffusion/Reflection)
    // If enabled (strength > 0)
    if film.halation_strength > 0.0 {
        // Create a map of high-intensity areas that will scatter
        // Thresholding: Only bright lights cause significant halation
        let threshold = film.halation_threshold; // Linear light threshold
        let mut halation_map = linear_image.clone();

        // Apply threshold
        halation_map.pixels_mut().for_each(|p| {
            // Luminance approx
            let lum = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
            if lum < threshold {
                p.0 = [0.0, 0.0, 0.0];
            } else {
                p.0 = [
                    (p[0] - threshold).max(0.0),
                    (p[1] - threshold).max(0.0),
                    (p[2] - threshold).max(0.0),
                ];
            }
        });

        // Blur the map to simulate scattering in the base
        // Sigma depends on base thickness.
        let blur_sigma = width as f32 * film.halation_sigma; // % of width dispersion
        apply_gaussian_blur(&mut halation_map, blur_sigma);

        // Add back to linear image
        let tint = film.halation_tint; // Tint color

        linear_image
            .pixels_mut()
            .zip(halation_map.pixels())
            .for_each(|(dest, src)| {
                let strength = film.halation_strength;
                dest[0] += src[0] * tint[0] * strength;
                dest[1] += src[1] * tint[1] * strength;
                dest[2] += src[2] * tint[2] * strength;
            });
    }

    // 3. MTF Blur (Optical Softness)
    // Simulate light diffusion in the emulsion before it hits the crystals.
    // Cutoff frequency fc approx resolution / 2.
    // Gaussian sigma approx 1 / (2 * pi * fc) in pixels?
    // Let's assume resolution_lp_mm maps to pixels.
    // Assuming 35mm film width = 36mm.
    let pixels_per_mm = width as f32 / 36.0;
    // resolution_lp_mm is limit. Sigma should be related to this.
    // A loose approximation: Blur radius = 0.5 * (1/Res) * PixelsPerMM
    // e.g. Res=100 lp/mm -> 0.01 mm per pair. 0.005 mm per line.
    // 0.005 * pixels_per_mm.
    // If Res=100, Width=6000 (24MP). PixelsPerMM = 166.
    // Blur Radius = 0.005 * 166 = 0.83 pixels.
    let mtf_sigma = (0.5 / film.resolution_lp_mm) * pixels_per_mm;
    if mtf_sigma > 0.5 {
         apply_gaussian_blur(&mut linear_image, mtf_sigma);
    }

    // 4. Process Exposure -> Density
    let camera_sens = CameraSensitivities::srgb();
    let mut film_sens = film.get_spectral_sensitivities();
    let illuminant = Spectrum::new_d65();
    let apply_illuminant = |s: Spectrum| s.multiply(&illuminant);

    // CALIBRATION: Ensure energy conservation
    // Construct the "System White Point" (sRGB (1,1,1) uplifted and illuminated by D65).
    // This ensures that a neutral white input results in equal exposure to all film layers.
    let system_white = apply_illuminant(camera_sens.uplift(1.0, 1.0, 1.0));
    film_sens.calibrate_to_white_point(&system_white);

    // Calculate Reciprocity Failure Factor
    // E_film = E_actual / (1 + beta * log10(t)^2)
    let reciprocity_factor = if config.exposure_time > 1.0 {
        1.0 + film.reciprocity_beta * config.exposure_time.log10().powi(2)
    } else {
        1.0
    };
    let t_eff = config.exposure_time / reciprocity_factor;

    // Calculate White Balance Gain (Physical Layer / Latent Image Domain)
    // We analyze the "Latent Image" (exposure values after spectral integration)
    // and balance them before they hit the non-linear film curves.
    let wb_gains = match config.white_balance_mode {
        WhiteBalanceMode::Auto => {
            // Gray World on Latent Image
            let step = (width * height / 1000).max(1);
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut count = 0.0;

            for i in (0..(width * height)).step_by(step as usize) {
                let x = i % width;
                let y = i / width;
                let p = linear_image.get_pixel(x, y).0;
                
                // Full spectral path for sample
                // Note: We don't apply SPECTRAL_NORM here as we just want ratios
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
        },
        WhiteBalanceMode::Gray | WhiteBalanceMode::White => [1.0, 1.0, 1.0],
        WhiteBalanceMode::Off => [1.0, 1.0, 1.0],
    };

    // Buffer for Density (D)
    let mut density_image: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

    density_image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        let lin_pixel = linear_image.get_pixel(x, y).0;

        // Spectral Uplift
        let scene_spectrum = apply_illuminant(camera_sens.uplift(lin_pixel[0], lin_pixel[1], lin_pixel[2]));
        let exposure_vals = film_sens.expose(&scene_spectrum);

        // Apply White Balance (Physical Layer)
        // We modify the exposure values directly.
        let r_balanced = exposure_vals[0] * wb_gains[0];
        let g_balanced = exposure_vals[1] * wb_gains[1];
        let b_balanced = exposure_vals[2] * wb_gains[2];

        // Calculate Log Exposure
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

        // Map to Density (H-D Curve + Color Matrix)
        let densities = film.map_log_exposure(log_e);
        pixel[0] = densities[0];
        pixel[1] = densities[1];
        pixel[2] = densities[2];
    });

    // 5. Grain Simulation (Spatially Correlated)
    if config.enable_grain {
        let mut grain_noise: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);
        
        // Generate white noise based on density
        grain_noise.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            let densities = density_image.get_pixel(x, y).0;
            let mut rng = rand::thread_rng();

            if film.grain_model.monochrome {
                let d = densities[1]; // Use Green for reference
                let noise = film.grain_model.sample_noise(d, &mut rng);
                pixel[0] = noise;
                pixel[1] = noise;
                pixel[2] = noise;
            } else {
                // Organic Color Noise:
                // Real color film grain is correlated because dye clouds form around silver.
                // Pure independent RGB noise looks like digital sensor noise.
                // We generate independent noise but blend it towards luminance to reduce chroma noise.
                let n_r = film.grain_model.sample_noise(densities[0], &mut rng);
                let n_g = film.grain_model.sample_noise(densities[1], &mut rng);
                let n_b = film.grain_model.sample_noise(densities[2], &mut rng);
                
                let n_lum = (n_r + n_g + n_b) / 3.0;
                let chroma_scale = 0.3; // Reduce chroma noise to 30%
                
                pixel[0] = n_lum + (n_r - n_lum) * chroma_scale;
                pixel[1] = n_lum + (n_g - n_lum) * chroma_scale;
                pixel[2] = n_lum + (n_b - n_lum) * chroma_scale;
            }
        });

        // Apply blur to noise (simulate grain size)
        // blur_radius in GrainModel is relative to pixel? Or absolute?
        // Assuming blur_radius is in pixels.
        if film.grain_model.blur_radius > 0.0 {
             apply_gaussian_blur(&mut grain_noise, film.grain_model.blur_radius);
        }

        // Add noise to density
        density_image.pixels_mut().zip(grain_noise.pixels()).for_each(|(d, n)| {
            d[0] = (d[0] + n[0]).max(0.0);
            d[1] = (d[1] + n[1]).max(0.0);
            d[2] = (d[2] + n[2]).max(0.0);
        });
    }

    // 6. Map Density to Output
    let map_densities = |densities: [f32; 3]| -> (f32, f32, f32) {
        let net_r = (densities[0] - film.r_curve.d_min).max(0.0);
        let net_g = (densities[1] - film.g_curve.d_min).max(0.0);
        let net_b = (densities[2] - film.b_curve.d_min).max(0.0);
        match config.output_mode {
            OutputMode::Negative => {
                let t_r = physics::density_to_transmission(net_r);
                let t_g = physics::density_to_transmission(net_g);
                let t_b = physics::density_to_transmission(net_b);
                (t_r.clamp(0.0, 1.0), t_g.clamp(0.0, 1.0), t_b.clamp(0.0, 1.0))
            }
            OutputMode::Positive => {
                let t_r = physics::density_to_transmission(net_r);
                let t_g = physics::density_to_transmission(net_g);
                let t_b = physics::density_to_transmission(net_b);
                let t_r_max = physics::density_to_transmission(0.0);
                let t_g_max = physics::density_to_transmission(0.0);
                let t_b_max = physics::density_to_transmission(0.0);
                let t_r_min = physics::density_to_transmission((film.r_curve.d_max - film.r_curve.d_min).max(0.0));
                let t_g_min = physics::density_to_transmission((film.g_curve.d_max - film.g_curve.d_min).max(0.0));
                let t_b_min = physics::density_to_transmission((film.b_curve.d_max - film.b_curve.d_min).max(0.0));
                let norm = |t: f32, t_min: f32, t_max: f32| {
                    let denom = (t_max - t_min).max(1e-6);
                    let n = (t_max - t).clamp(0.0, denom) / denom;
                    // Apply Paper Gamma (Contrast Boost)
                    // Standard Grade 2-3 paper has gamma ~2.0 relative to negative density range
                    // This simulates the printing process where high-contrast paper restores scene contrast.
                    n.powf(2.0)
                };
                (norm(t_r, t_r_min, t_r_max), norm(t_g, t_g_min, t_g_max), norm(t_b, t_b_min, t_b_max))
            }
        }
    };

    let mut pixels: Vec<u8> = vec![0; (width * height * 3) as usize];
    
    // Previous WB logic removed as we now do it in spectral domain.
    // If output-side adjustments are needed (e.g. creative tinting), they can be added here.
    // For now, we assume the spectral WB + H-D Curve provides the desired look.

    pixels.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        let d = density_image.get_pixel(x, y).0;
        
        let (r_lin, g_lin, b_lin) = map_densities(d);
        
        // No further WB here
        let r_out = physics::linear_to_srgb(r_lin.clamp(0.0, 1.0));
        let g_out = physics::linear_to_srgb(g_lin.clamp(0.0, 1.0));
        let b_out = physics::linear_to_srgb(b_lin.clamp(0.0, 1.0));

        chunk[0] = (r_out * 255.0).round() as u8;
        chunk[1] = (g_out * 255.0).round() as u8;
        chunk[2] = (b_out * 255.0).round() as u8;
    });

    RgbImage::from_raw(width, height, pixels).unwrap()
}
