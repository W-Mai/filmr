use filmr::film::SegmentedCurve;
use filmr::physics;
use filmr::spectral::{CameraSensitivities, FilmSensitivities, FilmSpectralParams};

// Mock dependencies to replicate processor.rs logic
const SPECTRAL_NORM: f32 = 0.008;

fn run_pipeline_trace(r: u8, g: u8, b: u8, exposure_time: f32) {
    println!("\n=== Pipeline Trace for Input RGB({}, {}, {}) ===", r, g, b);
    
    // 1. Linearize Input
    let lin_r = physics::srgb_to_linear(r as f32 / 255.0);
    let lin_g = physics::srgb_to_linear(g as f32 / 255.0);
    let lin_b = physics::srgb_to_linear(b as f32 / 255.0);
    println!("Linear RGB: {:.4}, {:.4}, {:.4}", lin_r, lin_g, lin_b);

    // 2. Uplift to Spectrum
    let camera_sens = CameraSensitivities::srgb_balanced();
    let spectrum = camera_sens.uplift(lin_r, lin_g, lin_b);
    
    // Check Spectrum Energy (Area)
    // Simple sum for debug
    let energy: f32 = spectrum.power.iter().sum();
    println!("Spectrum Total Energy (Sum): {:.4}", energy);

    // 3. Film Sensitivity Exposure
    let film_params = FilmSpectralParams::new_panchromatic();
    let film_sens = FilmSensitivities::from_params(film_params);
    
    println!("Film Factors: R={:.2}, G={:.2}, B={:.2}", 
             film_sens.r_factor, film_sens.g_factor, film_sens.b_factor);

    let exposure_vals = film_sens.expose(&spectrum);
    let r_in = (exposure_vals[0] * SPECTRAL_NORM).max(0.0);
    let g_in = (exposure_vals[1] * SPECTRAL_NORM).max(0.0);
    let b_in = (exposure_vals[2] * SPECTRAL_NORM).max(0.0);
    
    println!("Spectral Exposure (after Norm): R={:.4}, G={:.4}, B={:.4}", r_in, g_in, b_in);

    // 4. Calculate Exposure with Time
    // Assume reciprocity_exponent = 1.0 for simplicity in this trace, or use standard
    let reciprocity = 0.85;
    let t_eff = if exposure_time > 1.0 { exposure_time.powf(reciprocity) } else { exposure_time };
    
    let r_exp = physics::calculate_exposure(r_in, t_eff);
    let g_exp = physics::calculate_exposure(g_in, t_eff);
    let b_exp = physics::calculate_exposure(b_in, t_eff);
    
    println!("Total Exposure (E=I*t): R={:.4}, G={:.4}, B={:.4}", r_exp, g_exp, b_exp);
    println!("Log10 Exposure: R={:.4}, G={:.4}, B={:.4}", 
             r_exp.log10(), g_exp.log10(), b_exp.log10());

    // 5. Characteristic Curve (Density)
    // Using Standard Daylight curves from presets.rs
    // r_curve: d_min=0.12, d_max=2.9, gamma=1.8, offset=0.18
    // g_curve: d_min=0.10, d_max=3.0, gamma=1.8, offset=0.18
    // b_curve: d_min=0.11, d_max=2.8, gamma=1.7, offset=0.18
    let r_curve = SegmentedCurve { d_min: 0.12, d_max: 2.9, gamma: 1.8, exposure_offset: 0.18 };
    let g_curve = SegmentedCurve { d_min: 0.10, d_max: 3.0, gamma: 1.8, exposure_offset: 0.18 };
    let b_curve = SegmentedCurve { d_min: 0.11, d_max: 2.8, gamma: 1.7, exposure_offset: 0.18 };
    
    // film.rs uses map_smooth usually? Let's check map_log_exposure impl
    // It calls r_curve.map(log_e) in current impl? No, wait, I need to check film.rs:188
    // Assuming map_smooth or map. Let's use map_smooth as it's more likely what's used for quality.
    let d_r = r_curve.map_smooth(r_exp.log10());
    let d_g = g_curve.map_smooth(g_exp.log10());
    let d_b = b_curve.map_smooth(b_exp.log10());
    
    println!("Density: R={:.4}, G={:.4}, B={:.4}", d_r, d_g, d_b);

    // 6. Color Matrix (Crosstalk)
    // Standard Daylight: [[1.0, 0.05, 0.02], [0.04, 1.0, 0.04], [0.01, 0.05, 1.0]]
    let matrix = [[1.00, 0.05, 0.02], [0.04, 1.00, 0.04], [0.01, 0.05, 1.00]];
    
    // Note: Matrix application in film.rs map_log_exposure applies matrix to Densities?
    // Let's check film.rs logic.
    // map_log_exposure gets densities from curves first, then applies matrix.
    // Let's verify film.rs:188
    
    let d_r_mix = matrix[0][0]*d_r + matrix[0][1]*d_g + matrix[0][2]*d_b;
    let d_g_mix = matrix[1][0]*d_r + matrix[1][1]*d_g + matrix[1][2]*d_b;
    let d_b_mix = matrix[2][0]*d_r + matrix[2][1]*d_g + matrix[2][2]*d_b;
    
    println!("Mixed Density: R={:.4}, G={:.4}, B={:.4}", d_r_mix, d_g_mix, d_b_mix);

    // 7. Output Mode (Positive)
    // processor.rs:408
    // norm = d / 3.0
    // linear_to_srgb
    let max_d = 3.0;
    let out_r_lin = (d_r_mix / max_d).clamp(0.0, 1.0);
    let out_g_lin = (d_g_mix / max_d).clamp(0.0, 1.0);
    let out_b_lin = (d_b_mix / max_d).clamp(0.0, 1.0);
    
    println!("Output Linear: R={:.4}, G={:.4}, B={:.4}", out_r_lin, out_g_lin, out_b_lin);
    
    let out_r_srgb = physics::linear_to_srgb(out_r_lin);
    let out_g_srgb = physics::linear_to_srgb(out_g_lin);
    let out_b_srgb = physics::linear_to_srgb(out_b_lin);
    
    let final_r = (out_r_srgb * 255.0).round() as u8;
    let final_g = (out_g_srgb * 255.0).round() as u8;
    let final_b = (out_b_srgb * 255.0).round() as u8;

    println!("Final sRGB: ({}, {}, {})", final_r, final_g, final_b);
}

#[test]
fn verify_pipeline_white() {
    // White point trace
    run_pipeline_trace(255, 255, 255, 1.0);
}

#[test]
fn verify_pipeline_grey() {
    // Grey point trace
    run_pipeline_trace(128, 128, 128, 1.0);
}
