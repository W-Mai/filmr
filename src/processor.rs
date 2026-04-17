use crate::film::FilmStock;
use crate::film_layer::FilmLayerStack;
use crate::light_leak::{LightLeakConfig, LightLeakStage};
use crate::physics;
use crate::pipeline::{
    create_linear_image, create_output_image, ChromaticAberrationStage, DepthOfFieldStage,
    DevelopStage, GrainStage, HalationStage, MicroMotionStage, MtfStage, ObjectMotionStage,
    PipelineContext, PipelineStage, RotationalBlurStage,
};
use crate::spectral_engine;
use image::RgbImage;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

#[cfg(feature = "compute-gpu")]
use crate::gpu::get_gpu_context;
#[cfg(feature = "compute-gpu")]
use crate::gpu_pipelines::{
    get_gaussian_pipeline, get_halation_pipeline, get_light_leak_pipeline, get_linearize_pipeline,
    read_gpu_buffer,
};

/// Simulation fidelity mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SimulationMode {
    /// 3×3 matrix approximation — fast, kept for compatibility.
    Fast,
    /// Full-spectrum per-wavelength propagation through film layer stack.
    #[default]
    Accurate,
}

/// Configuration for the simulation run.
/// Controls all aspects of the physical simulation pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Simulation fidelity mode.
    #[serde(default)]
    pub simulation_mode: SimulationMode,
    /// Exposure time (t) in seconds.
    /// Used for Reciprocity Failure calculation (E = I * t).
    pub exposure_time: f32, // t in E = I * t
    /// Enable or disable grain simulation.
    pub enable_grain: bool,
    /// Enable GPU acceleration if available.
    #[serde(default)]
    pub use_gpu: bool,
    /// Output mode: Negative (Transmission) or Positive (Scanned).
    pub output_mode: OutputMode,
    /// White Balance mode.
    pub white_balance_mode: WhiteBalanceMode,
    /// Strength of White Balance correction (0.0 to 1.0).
    pub white_balance_strength: f32,
    /// Warmth adjustment (-1.0 to 1.0).
    pub warmth: f32,
    /// Saturation adjustment (0.0 to 2.0).
    pub saturation: f32,
    /// Light leak simulation configuration.
    pub light_leak: LightLeakConfig,
    /// Motion blur amount (0.0 = off, 1.0 = default hand shake).
    #[serde(default = "default_motion_blur")]
    pub motion_blur_amount: f32,
    /// Motion blur random seed (same seed = same trajectory).
    #[serde(default)]
    pub motion_blur_seed: u64,
    /// Object motion amount (0.0 = off, 1.0 = default depth-based motion).
    #[serde(default)]
    pub object_motion_amount: f32,
    /// Auto black/white point stretch (like scanner auto-levels).
    #[serde(default)]
    pub auto_levels: bool,
    /// Depth of field blur amount (0.0 = off, 1.0 = default).
    #[serde(default)]
    pub dof_amount: f32,
    /// Depth of field focus point (0.0 = nearest, 1.0 = farthest).
    #[serde(default = "default_dof_focus")]
    pub dof_focus: f32,
    /// Rotational blur amount (0.0 = off, simulates camera rotation).
    #[serde(default)]
    pub rotational_blur_amount: f32,
}

fn default_motion_blur() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputMode {
    Negative, // Transmission of the negative (Dark -> Bright, Bright -> Dark)
    Positive, // Scanned/Inverted Positive (Dark -> Dark, Bright -> Bright)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WhiteBalanceMode {
    Auto,
    Gray,
    White,
    Off,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            simulation_mode: SimulationMode::default(),
            exposure_time: 1.0,
            enable_grain: true,
            use_gpu: false,                    // Default to CPU for stability
            output_mode: OutputMode::Positive, // Default to what users expect
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
            warmth: 0.0,
            saturation: 1.0,
            light_leak: LightLeakConfig::default(),
            motion_blur_amount: 1.0,
            motion_blur_seed: 42,
            object_motion_amount: 0.0,
            auto_levels: false,
            dof_amount: 0.0,
            dof_focus: 0.5,
            rotational_blur_amount: 0.0,
        }
    }
}

fn default_dof_focus() -> f32 {
    0.5
}

const SPECTRAL_NORM: f32 = 1.0;

#[instrument(skip(input, film))]
pub fn estimate_exposure_time(input: &RgbImage, film: &FilmStock) -> f32 {
    estimate_exposure_time_for_mode(input, film, SimulationMode::Fast)
}

/// Estimate exposure time, accounting for simulation mode.
/// In Accurate mode, normalization already handles auto-exposure,
/// so this returns 1.0 (neutral EV).
#[instrument(skip(input, film))]
pub fn estimate_exposure_time_for_mode(
    input: &RgbImage,
    film: &FilmStock,
    mode: SimulationMode,
) -> f32 {
    if mode == SimulationMode::Accurate {
        return 1.0;
    }
    debug!("Estimating exposure time...");

    let spectral_matrix = film.compute_spectral_matrix();
    let apply_matrix = |r: f32, g: f32, b: f32| -> [f32; 3] {
        [
            r * spectral_matrix[0][0] + g * spectral_matrix[0][1] + b * spectral_matrix[0][2],
            r * spectral_matrix[1][0] + g * spectral_matrix[1][1] + b * spectral_matrix[1][2],
            r * spectral_matrix[2][0] + g * spectral_matrix[2][1] + b * spectral_matrix[2][2],
        ]
    };

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
        let exposure_vals = apply_matrix(r, g, b);
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
        // Density linear mapping (same as create_output_image Positive path)
        let range_r = (film.r_curve.d_max - film.r_curve.d_min).max(0.01);
        let range_g = (film.g_curve.d_max - film.g_curve.d_min).max(0.01);
        let range_b = (film.b_curve.d_max - film.b_curve.d_min).max(0.01);
        let tone_gamma = 2.47f32;
        (
            (net_r / range_r).clamp(0.0, 1.0).powf(tone_gamma),
            (net_g / range_g).clamp(0.0, 1.0).powf(tone_gamma),
            (net_b / range_b).clamp(0.0, 1.0).powf(tone_gamma),
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
            let factor = 1.0 + film.reciprocity.beta * t.log10().powi(2);
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

/// Main processor function.
/// Takes an input image and film parameters, returns the simulated image.
#[instrument(skip(input, film, config))]
pub fn process_image(input: &RgbImage, film: &FilmStock, config: &SimulationConfig) -> RgbImage {
    process_image_with_depth(input, film, config, None)
}

pub fn process_image_with_depth(
    input: &RgbImage,
    film: &FilmStock,
    config: &SimulationConfig,
    depth_map: Option<&crate::depth::DepthMap>,
) -> RgbImage {
    info!("Starting film simulation processing");

    let context = PipelineContext {
        film,
        config,
        depth_map,
    };

    #[cfg(feature = "compute-gpu")]
    let gpu_result = if config.use_gpu {
        process_gpu_pipeline(&context, input, film, config)
    } else {
        None
    };

    #[cfg(not(feature = "compute-gpu"))]
    let gpu_result: Option<image::ImageBuffer<image::Rgb<f32>, Vec<f32>>> = None;

    let mut image_buffer = if let Some(buffer) = gpu_result {
        buffer
    } else {
        let mut buffer = create_linear_image(input);
        process_cpu_fallback(&mut buffer, &context);
        buffer
    };

    // 3. Other Stages
    let stages: Vec<Box<dyn PipelineStage>> = match config.simulation_mode {
        SimulationMode::Fast => vec![
            Box::new(MicroMotionStage),
            Box::new(ObjectMotionStage),
            Box::new(DepthOfFieldStage),
            Box::new(RotationalBlurStage),
            Box::new(MtfStage),
            Box::new(ChromaticAberrationStage),
            Box::new(DevelopStage),
            Box::new(GrainStage),
        ],
        SimulationMode::Accurate => {
            // Lens/scene effects BEFORE develop (operate on linear RGB)
            let pre_stages: Vec<Box<dyn PipelineStage>> = vec![
                Box::new(MicroMotionStage),
                Box::new(ObjectMotionStage),
                Box::new(DepthOfFieldStage),
                Box::new(RotationalBlurStage),
                Box::new(MtfStage),
                Box::new(ChromaticAberrationStage),
            ];
            for stage in pre_stages {
                stage.process(&mut image_buffer, &context);
            }
            // Spectral develop (includes BW grayscale merge)
            AccurateDevelopStage.process(&mut image_buffer, &context);
            vec![Box::new(GrainStage)]
        }
    };

    for stage in stages {
        stage.process(&mut image_buffer, &context);
    }

    // Output Conversion
    create_output_image(&image_buffer, &context)
}

/// # Accurate Develop Stage
///
/// Full-spectrum per-wavelength propagation through the film layer stack.
/// Replaces DevelopStage in Accurate mode.
struct AccurateDevelopStage;

impl PipelineStage for AccurateDevelopStage {
    #[instrument(skip(self, image, context))]
    fn process(
        &self,
        image: &mut image::ImageBuffer<image::Rgb<f32>, Vec<f32>>,
        context: &PipelineContext,
    ) {
        info!("Developing film (Accurate: full-spectrum propagation)");
        let film = context.film;
        let config = context.config;

        let stack = film.layer_stack.clone().unwrap_or_else(|| {
            use crate::film::FilmType;
            match film.film_type {
                FilmType::BwNegative => FilmLayerStack::default_bw_negative(),
                _ => FilmLayerStack::default_color_negative(),
            }
        });

        let camera = crate::spectral::CameraSensitivities::srgb();
        let d65 = crate::spectral::Spectrum::new_d65();

        // Exposure calibration: find norm so that 18% gray → sRGB ~118 output.
        // Binary search over a scale factor applied to propagated exposure.
        // This makes brightness consistent across all presets.
        let gray_lin = 0.18f32;

        let gray_spectrum = camera.uplift(gray_lin, gray_lin, gray_lin);
        let mut gray_scaled = [0.0f32; crate::spectral::BINS];
        for (i, s) in gray_scaled.iter_mut().enumerate() {
            *s = gray_spectrum.power[i] * d65.power[i];
        }
        let gray_exp = spectral_engine::propagate(&stack, &gray_scaled);
        let acc_gray = spectral_engine::integrate_exposure(&gray_exp);
        let acc_gray_avg = (acc_gray[0] + acc_gray[1] + acc_gray[2]) / 3.0;

        // BW weights (precompute once, used in simulate_gray and pixel path)
        let is_bw = film.grain_model.monochrome;
        let bw_w = if is_bw { film.bw_weights() } else { [0.0; 3] };

        // Simulate full pipeline for a single gray pixel at a given scale
        let simulate_gray = |scale: f32| -> f32 {
            let mut exposure = [
                acc_gray[0] * scale,
                acc_gray[1] * scale,
                acc_gray[2] * scale,
            ];
            // BW: merge channels before density mapping (same as pixel path)
            if is_bw {
                let v = bw_w[0] * exposure[0] + bw_w[1] * exposure[1] + bw_w[2] * exposure[2];
                exposure = [v, v, v];
            }
            let epsilon = 1e-6f32;
            let log_e = [
                exposure[0].max(epsilon).log10(),
                exposure[1].max(epsilon).log10(),
                exposure[2].max(epsilon).log10(),
            ];
            let d = film.map_log_exposure(log_e);
            // Simplified positive output (same as create_output_image)
            let net_r = (d[0] - film.r_curve.d_min).max(0.0);
            let net_g = (d[1] - film.g_curve.d_min).max(0.0);
            let net_b = (d[2] - film.b_curve.d_min).max(0.0);
            let range_r = (film.r_curve.d_max - film.r_curve.d_min).max(0.01);
            let range_g = (film.g_curve.d_max - film.g_curve.d_min).max(0.01);
            let range_b = (film.b_curve.d_max - film.b_curve.d_min).max(0.01);
            let tone_gamma = 2.47f32;
            let r = (net_r / range_r).clamp(0.0, 1.0).powf(tone_gamma);
            let g = (net_g / range_g).clamp(0.0, 1.0).powf(tone_gamma);
            let b = (net_b / range_b).clamp(0.0, 1.0).powf(tone_gamma);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        };

        // Binary search: find scale where simulate_gray(scale) ≈ 0.18 (linear)
        // Target: 18% gray output. Slightly above 0.18 to compensate for
        // scatter/WB losses in the full Accurate pipeline that simulate_gray
        // does not model.
        let target_linear = 0.18f32;
        let mut lo = 1e-8f32;
        let mut hi = 1e4f32;
        for _ in 0..40 {
            let mid = (lo * hi).sqrt(); // geometric midpoint (log-space search)
            let result = simulate_gray(mid);
            if result < target_linear {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let optimal_scale = (lo * hi).sqrt();

        // norm = optimal_scale / acc_gray (per-channel for white balance)
        let norm = [
            if acc_gray[0] > 1e-10 {
                optimal_scale * acc_gray_avg / acc_gray[0]
            } else {
                1.0
            },
            if acc_gray[1] > 1e-10 {
                optimal_scale * acc_gray_avg / acc_gray[1]
            } else {
                1.0
            },
            if acc_gray[2] > 1e-10 {
                optimal_scale * acc_gray_avg / acc_gray[2]
            } else {
                1.0
            },
        ];

        // exposure_time: user EV adjustment (1.0 = neutral for Accurate mode)
        let t_eff = config.exposure_time;
        let width = image.width();

        // Precompute layer coefficients (once per frame, not per pixel)
        let (fwd_coeffs, bwd_coeffs) = spectral_engine::precompute(&stack);
        let base_n = stack
            .layers
            .last()
            .map(|l| l.refractive_index)
            .unwrap_or(1.0);
        let base_r = ((base_n - 1.0) / (base_n + 1.0)).powi(2);

        // Precompute uplift × D65 matrix: 3 input channels → 81 spectral bins
        let uplift_d65 = {
            let r_spec = camera.uplift(1.0, 0.0, 0.0);
            let g_spec = camera.uplift(0.0, 1.0, 0.0);
            let b_spec = camera.uplift(0.0, 0.0, 1.0);
            let mut m = [[0.0f32; crate::spectral::BINS]; 3];
            for (i, ((r, g), b)) in r_spec
                .power
                .iter()
                .zip(g_spec.power.iter())
                .zip(b_spec.power.iter())
                .enumerate()
            {
                let d = d65.power[i];
                m[0][i] = r * d;
                m[1][i] = g * d;
                m[2][i] = b * d;
            }
            m
        };

        // Pass 1: per-pixel spectral propagation → RGB exposure
        image.par_chunks_mut(3).for_each(|pixel| {
            // Inline uplift × D65 (3 multiplies + 2 adds per bin instead of full uplift)
            let mut scaled = [0.0f32; crate::spectral::BINS];
            let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
            for (i, s) in scaled.iter_mut().enumerate() {
                *s = r * uplift_d65[0][i] + g * uplift_d65[1][i] + b * uplift_d65[2][i];
            }
            let exposure =
                spectral_engine::propagate_fast(&fwd_coeffs, &bwd_coeffs, base_r, &scaled);
            let rgb = spectral_engine::integrate_exposure(&exposure);
            pixel[0] = rgb[0] * norm[0] * t_eff;
            pixel[1] = rgb[1] * norm[1] * t_eff;
            pixel[2] = rgb[2] * norm[2] * t_eff;
        });

        // BW films: single emulsion layer only produces one channel of exposure.
        // Merge RGB→mono using the film's spectral response weights before
        // density mapping, so all subsequent stages operate on grayscale.
        if is_bw {
            let [wr, wg, wb] = bw_w;
            image.par_chunks_mut(3).for_each(|pixel| {
                let v = wr * pixel[0] + wg * pixel[1] + wb * pixel[2];
                pixel[0] = v;
                pixel[1] = v;
                pixel[2] = v;
            });
        }

        // Pass 2: scattering spatial diffusion (Gaussian blur per emulsion scatter)
        // Total scatter sigma ≈ sum of (thickness * scattering) across emulsion layers,
        // converted from µm to pixels assuming 35mm width.
        let scatter_um: f32 = stack
            .layers
            .iter()
            .map(|l| l.thickness_um * l.scattering)
            .sum();
        if scatter_um > 0.0 {
            let pixels_per_um = width as f32 / 36_000.0; // 36mm = 36000µm
            let sigma_px = scatter_um * pixels_per_um;
            if sigma_px > 0.3 {
                info!(
                    "Applying scattering diffusion blur (sigma: {:.2}px)",
                    sigma_px
                );
                crate::utils::apply_gaussian_blur(image, sigma_px);
            }
        }

        // Pass 2.5: White balance + warmth (same logic as Fast mode DevelopStage)
        let wb_gains = match config.white_balance_mode {
            crate::processor::WhiteBalanceMode::Auto => {
                let step = (width * image.height() / 1000).max(1);
                let mut sum_r = 0.0f32;
                let mut sum_g = 0.0f32;
                let mut sum_b = 0.0f32;
                let mut count = 0.0f32;
                for (i, pixel) in image.chunks(3).enumerate() {
                    if (i as u32).is_multiple_of(step) {
                        sum_r += pixel[0];
                        sum_g += pixel[1];
                        sum_b += pixel[2];
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    let avg_r = sum_r / count;
                    let avg_g = sum_g / count;
                    let avg_b = sum_b / count;
                    let lum = (avg_r + avg_g + avg_b) / 3.0;
                    let eps = 1e-9;
                    let s = config.white_balance_strength.clamp(0.0, 1.0);
                    let warmth = config.warmth.clamp(-1.0, 1.0);
                    [
                        (1.0 + (lum / avg_r.max(eps) - 1.0) * s) * (1.0 + warmth * 0.1),
                        1.0 + (lum / avg_g.max(eps) - 1.0) * s,
                        (1.0 + (lum / avg_b.max(eps) - 1.0) * s) * (1.0 - warmth * 0.1),
                    ]
                } else {
                    [1.0, 1.0, 1.0]
                }
            }
            _ => {
                let warmth = config.warmth.clamp(-1.0, 1.0);
                [1.0 + warmth * 0.1, 1.0, 1.0 - warmth * 0.1]
            }
        };

        image.par_chunks_mut(3).for_each(|pixel| {
            pixel[0] *= wb_gains[0];
            pixel[1] *= wb_gains[1];
            pixel[2] *= wb_gains[2];
        });

        // Pass 3: log-exposure → density via H-D curves + color matrix + inhibition
        let inhibition = stack.inhibition;
        image.par_chunks_mut(3).for_each(|pixel| {
            let epsilon = 1e-6;
            let log_e = [
                pixel[0].max(epsilon).log10(),
                pixel[1].max(epsilon).log10(),
                pixel[2].max(epsilon).log10(),
            ];
            let d = film.map_log_exposure(log_e);

            // Interlayer interimage effect: inhibition based on density DEVIATION
            // from the mean. This ensures neutral gray is unaffected while
            // colour differences are enhanced (physically: DIR couplers respond
            // to development rate differences between adjacent layers).
            let d_mean = (d[0] + d[1] + d[2]) / 3.0;
            let dd = [d[0] - d_mean, d[1] - d_mean, d[2] - d_mean];
            pixel[0] = (d[0]
                + inhibition[0][0] * dd[0]
                + inhibition[0][1] * dd[1]
                + inhibition[0][2] * dd[2])
                .max(0.0);
            pixel[1] = (d[1]
                + inhibition[1][0] * dd[0]
                + inhibition[1][1] * dd[1]
                + inhibition[1][2] * dd[2])
                .max(0.0);
            pixel[2] = (d[2]
                + inhibition[2][0] * dd[0]
                + inhibition[2][1] * dd[1]
                + inhibition[2][2] * dd[2])
                .max(0.0);
        });
    }
}

/// Helper function to execute CPU fallback stages
#[allow(dead_code)] // Suppress unused warning if gpu feature is disabled or always used
fn process_cpu_fallback(
    image_buffer: &mut image::ImageBuffer<image::Rgb<f32>, Vec<f32>>,
    context: &PipelineContext,
) {
    // 1. Light Leak (CPU)
    let light_leak = LightLeakStage;
    light_leak.process(image_buffer, context);

    // 2. Halation (CPU)
    let halation = HalationStage;
    halation.process(image_buffer, context);
}

#[cfg(feature = "compute-gpu")]
fn process_gpu_pipeline(
    _context: &PipelineContext,
    input: &RgbImage,
    film: &FilmStock,
    config: &SimulationConfig,
) -> Option<image::ImageBuffer<image::Rgb<f32>, Vec<f32>>> {
    use crate::gpu::GpuBuffer;

    let gpu_ctx = get_gpu_context()?;
    let mut gpu_buffer: Option<GpuBuffer>;

    // Linearization
    {
        let _span = tracing::info_span!("GPU Linearization").entered();
        info!("Attempting GPU Linearization...");
        let pipeline = get_linearize_pipeline(gpu_ctx);
        gpu_buffer = pipeline.process_to_gpu_buffer(gpu_ctx, input);
    }

    gpu_buffer.as_ref()?;

    // Light Leak
    if let Some(ref mut buffer) = gpu_buffer {
        let _span = tracing::info_span!("GPU Light Leak").entered();
        info!("Applying Light Leak on GPU");
        let pipeline = get_light_leak_pipeline(gpu_ctx);
        pipeline.process(gpu_ctx, buffer, &config.light_leak);
    }

    // Halation
    if let Some(buffer) = gpu_buffer.take() {
        if film.halation_strength > 0.0 {
            let _span = tracing::info_span!("GPU Halation").entered();
            info!("Applying Halation on GPU");
            let pipeline = get_halation_pipeline(gpu_ctx);
            if let Some(out_buffer) = pipeline.process(gpu_ctx, &buffer, film) {
                gpu_buffer = Some(out_buffer);
            } else {
                gpu_buffer = Some(buffer);
            }
        } else {
            gpu_buffer = Some(buffer);
        }
    }

    // MTF
    if let Some(buffer) = gpu_buffer.take() {
        let pixels_per_mm = buffer.width as f32 / 36.0;
        let mtf_sigma = (0.5 / film.resolution_lp_mm) * pixels_per_mm;

        if mtf_sigma > 0.5 {
            let _span = tracing::info_span!("GPU MTF Blur").entered();
            info!("Applying MTF Blur on GPU (sigma: {:.2})", mtf_sigma);
            let pipeline = get_gaussian_pipeline(gpu_ctx);
            if let Some(out_buffer) = pipeline.process(gpu_ctx, &buffer, mtf_sigma) {
                gpu_buffer = Some(out_buffer);
            } else {
                gpu_buffer = Some(buffer);
            }
        } else {
            gpu_buffer = Some(buffer);
        }
    }

    // Readback
    if let Some(ref buffer) = gpu_buffer {
        let _span = tracing::info_span!("GPU Readback").entered();
        info!("Reading back from GPU pipeline");
        crate::gpu::block_on(read_gpu_buffer(gpu_ctx, buffer))
    } else {
        None
    }
}

/// Main processor function (Async).
#[instrument(skip(input, film, config))]
pub async fn process_image_async(
    input: &RgbImage,
    film: &FilmStock,
    config: &SimulationConfig,
) -> RgbImage {
    info!("Starting film simulation processing (Async)");
    let context = PipelineContext {
        film,
        config,
        depth_map: None,
    };

    #[cfg(feature = "compute-gpu")]
    let gpu_result = if config.use_gpu {
        if let Some(gpu_ctx) = get_gpu_context() {
            info!("Attempting GPU Linearization...");
            let pipeline = get_linearize_pipeline(gpu_ctx);
            pipeline.process_image_async(gpu_ctx, input).await
        } else {
            None
        }
    } else {
        None
    };

    #[cfg(not(feature = "compute-gpu"))]
    let gpu_result: Option<image::ImageBuffer<image::Rgb<f32>, Vec<f32>>> = None;

    let mut image_buffer = if let Some(buffer) = gpu_result {
        info!("Used GPU for linearization");
        buffer
    } else {
        create_linear_image(input)
    };

    // Sequential Stage Execution
    let stages: Vec<Box<dyn PipelineStage>> = vec![
        Box::new(LightLeakStage),
        Box::new(HalationStage),
        Box::new(MicroMotionStage),
        Box::new(DepthOfFieldStage),
        Box::new(RotationalBlurStage),
        Box::new(MtfStage),
        Box::new(ChromaticAberrationStage),
        Box::new(DevelopStage),
        Box::new(GrainStage),
    ];

    for stage in stages {
        stage.process(&mut image_buffer, &context);
    }

    create_output_image(&image_buffer, &context)
}
