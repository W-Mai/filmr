use crate::film::FilmStock;
use crate::light_leak::{LightLeakConfig, LightLeakStage};
use crate::physics;
use crate::pipeline::{
    create_linear_image, create_output_image, DevelopStage, GrainStage, HalationStage, MtfStage,
    PipelineContext, PipelineStage,
};
use crate::spectral::{CameraSensitivities, Spectrum};
use image::RgbImage;
use tracing::{debug, info, instrument};

/// Configuration for the simulation run.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationConfig {
    pub exposure_time: f32, // t in E = I * t
    pub enable_grain: bool,
    pub output_mode: OutputMode,
    pub white_balance_mode: WhiteBalanceMode,
    pub white_balance_strength: f32,
    pub light_leak: LightLeakConfig,
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
            light_leak: LightLeakConfig::default(),
        }
    }
}

const SPECTRAL_NORM: f32 = 1.0;

#[instrument(skip(input, film))]
pub fn estimate_exposure_time(input: &RgbImage, film: &FilmStock) -> f32 {
    debug!("Estimating exposure time...");
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
    info!("Starting film simulation processing");
    // Pipeline initialization
    let context = PipelineContext { film, config };
    let mut image_buffer = create_linear_image(input);

    // Sequential Stage Execution
    let stages: Vec<Box<dyn PipelineStage>> = vec![
        Box::new(LightLeakStage), // Apply light leaks before halation so they contribute to halation
        Box::new(HalationStage),
        Box::new(MtfStage),
        Box::new(DevelopStage),
        Box::new(GrainStage),
    ];

    for stage in stages {
        stage.process(&mut image_buffer, &context);
    }

    // Output Conversion
    create_output_image(&image_buffer, &context)
}
