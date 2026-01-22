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

                // Store original value to subtract later?
                // We read from `image` and write to `temp`.

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
            // y_t is row index in transposed (0..height), which corresponds to x in original
            // row_t contains pixels for that row

            for x_t in 0..height {
                // x_t is col index in transposed (0..width), which corresponds to y in original
                // We want Transposed(x_t, y_t) <- Temp(y_t, x_t)
                // Wait.
                // Transposed(col, row) = Original(row, col)
                // Target pixel at (col=x_t, row=y_t) in Transposed comes from Original(x=y_t, y=x_t)

                // y_t is fixed for this row. It is the ROW of Transposed.
                // So it is the X coordinate of Original.
                // x_t iterates columns of Transposed. It is the Y coordinate of Original.

                // Check bounds:
                // Original image `temp` has width `width` and height `height`.
                // We access temp.get_pixel(y_t, x_t).
                // y_t must be < width. But y_t iterates 0..height?
                // NO.
                // transposed has width `height` and height `width`.
                // So row_t has length `height * 3`.
                // y_t iterates 0..width.

                // Let's recheck dimensions.
                // transposed = ImageBuffer::new(height, width); -> Width=height, Height=width.
                // So transposed.width() = height, transposed.height() = width.
                // par_chunks_mut((transposed.width() * 3))
                // So chunk size is height * 3.
                // y_t iterates 0..transposed.height() = 0..width.

                // Inside loop: x_t iterates 0..transposed.width() = 0..height.
                // We want Transposed(x_t, y_t).
                // Source is Temp(y_t, x_t).
                // Temp(x, y). x=y_t (0..width), y=x_t (0..height).
                // Correct.

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
                // Keep the bright pixels, maybe boost them?
                // The reflection is proportional to intensity.
                // Subtract threshold to make smooth transition?
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
        // Halation is usually reddish because red light penetrates deepest and reflects back.
        // We tint the scattered light.
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

    // Create output buffer
    let mut pixels: Vec<u8> = vec![0; (width * height * 3) as usize];

    // 3. Process Exposure -> Density -> Output
    let camera_sens = CameraSensitivities::srgb();
    let film_sens = film.get_spectral_sensitivities();
    let illuminant = Spectrum::new_d65();
    let apply_illuminant = |s: Spectrum| s.multiply(&illuminant);
    // Normalization factor to keep exposure values in a reasonable range
    // relative to the old RGB implementation.
    // tuned to approx 1/Integral(Gaussian^2) where width ~30nm
    const SPECTRAL_NORM: f32 = 0.008;
    let map_densities = |densities: [f32; 3]| -> (f32, f32, f32) {
        let net_r = (densities[0] - film.r_curve.d_min).max(0.0);
        let net_g = (densities[1] - film.g_curve.d_min).max(0.0);
        let net_b = (densities[2] - film.b_curve.d_min).max(0.0);
        match config.output_mode {
            OutputMode::Negative => {
                let t_r = physics::density_to_transmission(net_r);
                let t_g = physics::density_to_transmission(net_g);
                let t_b = physics::density_to_transmission(net_b);
                (
                    t_r.clamp(0.0, 1.0),
                    t_g.clamp(0.0, 1.0),
                    t_b.clamp(0.0, 1.0),
                )
            }
            OutputMode::Positive => {
                let t_r = physics::density_to_transmission(net_r);
                let t_g = physics::density_to_transmission(net_g);
                let t_b = physics::density_to_transmission(net_b);
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
                    (t_max - t).clamp(0.0, denom) / denom
                };
                (
                    norm(t_r, t_r_min, t_r_max),
                    norm(t_g, t_g_min, t_g_max),
                    norm(t_b, t_b_min, t_b_max),
                )
            }
        }
    };
    let compute_white_balance = |neutral: f32| {
        let scene_spectrum = apply_illuminant(camera_sens.uplift(neutral, neutral, neutral));
        let exposure_vals = film_sens.expose(&scene_spectrum);
        let r_in = (exposure_vals[0] * SPECTRAL_NORM).max(0.0);
        let g_in = (exposure_vals[1] * SPECTRAL_NORM).max(0.0);
        let b_in = (exposure_vals[2] * SPECTRAL_NORM).max(0.0);
        let t_eff = if config.exposure_time > 1.0 {
            config.exposure_time.powf(film.reciprocity_exponent)
        } else {
            config.exposure_time
        };
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
        let (r_lin, g_lin, b_lin) = map_densities(densities);
        let avg = (r_lin + g_lin + b_lin) / 3.0;
        [
            avg / r_lin.max(epsilon),
            avg / g_lin.max(epsilon),
            avg / b_lin.max(epsilon),
        ]
    };
    let white_balance_gray = compute_white_balance(0.18);
    let white_balance_white = compute_white_balance(1.0);
    let mut lum = Vec::with_capacity((width * height) as usize);
    for p in linear_image.pixels() {
        let lin_pixel = p.0;
        lum.push(0.2126 * lin_pixel[0] + 0.7152 * lin_pixel[1] + 0.0722 * lin_pixel[2]);
    }
    let lum_p50 = {
        let mut sorted = lum.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((sorted.len() - 1) as f32 * 0.5).round() as usize;
        sorted[idx]
    };
    let auto_wb = {
        let mut sum = [0.0f32; 3];
        let mut count = 0.0f32;
        for (i, p) in linear_image.pixels().enumerate() {
            if (lum[i] - lum_p50).abs() > lum_p50 * 0.1 {
                continue;
            }
            let lin_pixel = p.0;
            let scene_spectrum =
                apply_illuminant(camera_sens.uplift(lin_pixel[0], lin_pixel[1], lin_pixel[2]));
            let exposure_vals = film_sens.expose(&scene_spectrum);
            let r_in = (exposure_vals[0] * SPECTRAL_NORM).max(0.0);
            let g_in = (exposure_vals[1] * SPECTRAL_NORM).max(0.0);
            let b_in = (exposure_vals[2] * SPECTRAL_NORM).max(0.0);
            let t_eff = if config.exposure_time > 1.0 {
                config.exposure_time.powf(film.reciprocity_exponent)
            } else {
                config.exposure_time
            };
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
            let (r_lin, g_lin, b_lin) = map_densities(densities);
            let lum = 0.2126 * lin_pixel[0] + 0.7152 * lin_pixel[1] + 0.0722 * lin_pixel[2];
            let t = lum.clamp(0.0, 1.0);
            let white_balance = [
                white_balance_gray[0] * (1.0 - t) + white_balance_white[0] * t,
                white_balance_gray[1] * (1.0 - t) + white_balance_white[1] * t,
                white_balance_gray[2] * (1.0 - t) + white_balance_white[2] * t,
            ];
            sum[0] += (r_lin * white_balance[0]).clamp(0.0, 1.0);
            sum[1] += (g_lin * white_balance[1]).clamp(0.0, 1.0);
            sum[2] += (b_lin * white_balance[2]).clamp(0.0, 1.0);
            count += 1.0;
        }
        if count == 0.0 {
            for p in linear_image.pixels() {
                let lin_pixel = p.0;
                let scene_spectrum =
                    apply_illuminant(camera_sens.uplift(lin_pixel[0], lin_pixel[1], lin_pixel[2]));
                let exposure_vals = film_sens.expose(&scene_spectrum);
                let r_in = (exposure_vals[0] * SPECTRAL_NORM).max(0.0);
                let g_in = (exposure_vals[1] * SPECTRAL_NORM).max(0.0);
                let b_in = (exposure_vals[2] * SPECTRAL_NORM).max(0.0);
                let t_eff = if config.exposure_time > 1.0 {
                    config.exposure_time.powf(film.reciprocity_exponent)
                } else {
                    config.exposure_time
                };
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
                let (r_lin, g_lin, b_lin) = map_densities(densities);
                let lum = 0.2126 * lin_pixel[0] + 0.7152 * lin_pixel[1] + 0.0722 * lin_pixel[2];
                let t = lum.clamp(0.0, 1.0);
                let white_balance = [
                    white_balance_gray[0] * (1.0 - t) + white_balance_white[0] * t,
                    white_balance_gray[1] * (1.0 - t) + white_balance_white[1] * t,
                    white_balance_gray[2] * (1.0 - t) + white_balance_white[2] * t,
                ];
                sum[0] += (r_lin * white_balance[0]).clamp(0.0, 1.0);
                sum[1] += (g_lin * white_balance[1]).clamp(0.0, 1.0);
                sum[2] += (b_lin * white_balance[2]).clamp(0.0, 1.0);
                count += 1.0;
            }
        }
        let avg = [sum[0] / count, sum[1] / count, sum[2] / count];
        let avg_all = (avg[0] + avg[1] + avg[2]) / 3.0;
        let epsilon = 1e-6;
        let raw = [
            avg_all / avg[0].max(epsilon),
            avg_all / avg[1].max(epsilon),
            avg_all / avg[2].max(epsilon),
        ];
        let strength = 0.6;
        let apply = |v: f32| (1.0 + (v - 1.0) * strength).clamp(0.85, 1.15);
        [apply(raw[0]), apply(raw[1]), apply(raw[2])]
    };
    let wb_strength = config.white_balance_strength.clamp(0.0, 1.0);
    let wb_target = match config.white_balance_mode {
        WhiteBalanceMode::Auto => auto_wb,
        WhiteBalanceMode::Gray => white_balance_gray,
        WhiteBalanceMode::White => white_balance_white,
        WhiteBalanceMode::Off => [1.0, 1.0, 1.0],
    };
    let white_balance = [
        1.0 + (wb_target[0] - 1.0) * wb_strength,
        1.0 + (wb_target[1] - 1.0) * wb_strength,
        1.0 + (wb_target[2] - 1.0) * wb_strength,
    ];

    pixels.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;

        let lin_pixel = linear_image.get_pixel(x, y).0;

        // Uplift RGB to Spectrum
        let scene_spectrum =
            apply_illuminant(camera_sens.uplift(lin_pixel[0], lin_pixel[1], lin_pixel[2]));

        // Integrate with Film Sensitivities
        let exposure_vals = film_sens.expose(&scene_spectrum);
        let r_in = (exposure_vals[0] * SPECTRAL_NORM).max(0.0);
        let g_in = (exposure_vals[1] * SPECTRAL_NORM).max(0.0);
        let b_in = (exposure_vals[2] * SPECTRAL_NORM).max(0.0);

        // Apply Exposure
        // E = I * t.
        // Reciprocity Failure: Effective Time t_eff = t^p (for t > 1.0s usually, but let's apply globally or with threshold)
        // Simple Schwarzschild model: t_eff = t.powf(film.reciprocity_exponent)
        let t_eff = if config.exposure_time > 1.0 {
            config.exposure_time.powf(film.reciprocity_exponent)
        } else {
            config.exposure_time
        };

        let r_exp = physics::calculate_exposure(r_in, t_eff);
        let g_exp = physics::calculate_exposure(g_in, t_eff);
        let b_exp = physics::calculate_exposure(b_in, t_eff);

        // Avoid log(0)
        let epsilon = 1e-6;
        let log_e = [
            r_exp.max(epsilon).log10(),
            g_exp.max(epsilon).log10(),
            b_exp.max(epsilon).log10(),
        ];

        // Film Response
        let densities = film.map_log_exposure(log_e);

        // Add Grain (Using Film's Grain Model)
        let final_densities = if config.enable_grain {
            let mut rng = rand::thread_rng();

            if film.grain_model.monochrome {
                // For B&W, generate one noise sample based on luminance (or Green channel)
                // and apply it to all channels to ensure neutral grain.
                // Since B&W film produces equal densities for R,G,B, any channel works.
                let d = densities[1]; // Use Green
                let noise = film.grain_model.sample_noise(d, &mut rng);
                [
                    (densities[0] + noise).max(0.0),
                    (densities[1] + noise).max(0.0),
                    (densities[2] + noise).max(0.0),
                ]
            } else {
                // For Color, independent noise per channel
                [
                    film.grain_model.add_grain(densities[0], &mut rng),
                    film.grain_model.add_grain(densities[1], &mut rng),
                    film.grain_model.add_grain(densities[2], &mut rng),
                ]
            }
        } else {
            densities
        };

        // Output Formatting
        let (r_lin, g_lin, b_lin) = map_densities(final_densities);
        let r_out = physics::linear_to_srgb((r_lin * white_balance[0]).clamp(0.0, 1.0));
        let g_out = physics::linear_to_srgb((g_lin * white_balance[1]).clamp(0.0, 1.0));
        let b_out = physics::linear_to_srgb((b_lin * white_balance[2]).clamp(0.0, 1.0));

        chunk[0] = (r_out * 255.0).round() as u8;
        chunk[1] = (g_out * 255.0).round() as u8;
        chunk[2] = (b_out * 255.0).round() as u8;
    });

    RgbImage::from_raw(width, height, pixels).unwrap()
}
