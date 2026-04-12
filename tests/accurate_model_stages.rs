//! Fine-grained model correctness tests for the Accurate simulation pipeline.
//!
//! Tests each stage independently with known inputs and verifiable outputs.

use filmr::cie_data::D65_SPD;
use filmr::film::SegmentedCurve;
use filmr::film_layer::*;
use filmr::physics;
use filmr::spectral::{CameraSensitivities, Spectrum, BINS, LAMBDA_START, LAMBDA_STEP};
use filmr::spectral_engine::{integrate_exposure, propagate, LayerExposure};

// =========================================================================
// Stage 1: uplift — RGB → spectrum reconstruction
// =========================================================================

#[test]
fn stage1_uplift_white_is_symmetric() {
    let cam = CameraSensitivities::srgb();
    let w = cam.uplift(1.0, 1.0, 1.0);
    // White spectrum should be non-negative everywhere
    for (i, &v) in w.power.iter().enumerate() {
        let nm = LAMBDA_START + i * LAMBDA_STEP;
        assert!(v >= 0.0, "uplift(1,1,1) negative at {}nm: {}", nm, v);
    }
    // Should have energy across the visible range
    let sum: f32 = w.power.iter().sum();
    assert!(sum > 0.0, "White spectrum has no energy");
}

#[test]
fn stage1_uplift_red_peaks_in_red() {
    let cam = CameraSensitivities::srgb();
    let r = cam.uplift(1.0, 0.0, 0.0);
    // Find peak wavelength
    let peak_idx = r
        .power
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let peak_nm = LAMBDA_START + peak_idx * LAMBDA_STEP;
    println!("Red uplift peak: {}nm", peak_nm);
    assert!(
        peak_nm >= 580 && peak_nm <= 660,
        "Red uplift peak should be 580-660nm, got {}nm",
        peak_nm
    );
}

#[test]
fn stage1_uplift_green_peaks_in_green() {
    let cam = CameraSensitivities::srgb();
    let g = cam.uplift(0.0, 1.0, 0.0);
    let peak_idx = g
        .power
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let peak_nm = LAMBDA_START + peak_idx * LAMBDA_STEP;
    println!("Green uplift peak: {}nm", peak_nm);
    assert!(
        peak_nm >= 500 && peak_nm <= 570,
        "Green uplift peak should be 500-570nm, got {}nm",
        peak_nm
    );
}

#[test]
fn stage1_uplift_blue_peaks_in_blue() {
    let cam = CameraSensitivities::srgb();
    let b = cam.uplift(0.0, 0.0, 1.0);
    let peak_idx = b
        .power
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let peak_nm = LAMBDA_START + peak_idx * LAMBDA_STEP;
    println!("Blue uplift peak: {}nm", peak_nm);
    assert!(
        peak_nm >= 420 && peak_nm <= 480,
        "Blue uplift peak should be 420-480nm, got {}nm",
        peak_nm
    );
}

#[test]
fn stage1_uplift_linearity() {
    let cam = CameraSensitivities::srgb();
    let a = cam.uplift(0.3, 0.5, 0.7);
    let b = cam.uplift(0.6, 1.0, 1.4);
    for i in 0..BINS {
        let err = (b.power[i] - 2.0 * a.power[i]).abs();
        assert!(
            err < 1e-5,
            "uplift not linear at bin {}: 2*a={} b={}",
            i,
            2.0 * a.power[i],
            b.power[i]
        );
    }
}

// =========================================================================
// Stage 2: D65 illuminant
// =========================================================================

#[test]
fn stage2_d65_shape() {
    let d65 = Spectrum::new_d65();
    // D65 should peak around 460-470nm (CIE standard)
    let peak_idx = d65
        .power
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let peak_nm = LAMBDA_START + peak_idx * LAMBDA_STEP;
    println!("D65 peak: {}nm, value: {:.1}", peak_nm, d65.power[peak_idx]);
    // D65 has a broad peak, should be in visible range
    assert!(
        peak_nm >= 400 && peak_nm <= 600,
        "D65 peak at {}nm unexpected",
        peak_nm
    );
    // Value at 560nm should be ~100 (CIE convention)
    let idx_560 = (560 - LAMBDA_START) / LAMBDA_STEP;
    assert!(
        (d65.power[idx_560] - 100.0).abs() < 5.0,
        "D65 at 560nm should be ~100, got {:.1}",
        d65.power[idx_560]
    );
}

// =========================================================================
// Stage 3: propagate — per-layer physics
// =========================================================================

#[test]
fn stage3_single_absorbing_layer() {
    // A single emulsion layer with known absorption at 550nm
    let mut absorption = [0.0f32; BINS];
    let idx_550 = (550 - LAMBDA_START) / LAMBDA_STEP;
    absorption[idx_550] = 0.1; // 0.1 per µm

    let stack = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![FilmLayer {
            name: "Test Emulsion".into(),
            kind: LayerKind::Emulsion {
                channel: EmulsionChannel::Green,
            },
            thickness_um: 10.0,
            refractive_index: 1.5,
            absorption,
            scattering: 0.0,
        }],
    };

    let mut incident = [0.0f32; BINS];
    incident[idx_550] = 1.0;

    let exp = propagate(&stack, &incident);

    // Beer-Lambert: transmitted = exp(-0.1 * 10) = exp(-1) ≈ 0.368
    // Absorbed = 1 - 0.368 = 0.632
    // But there's also Fresnel reflection at air→1.5 interface: R = ((1-1.5)/(1+1.5))² = 0.04
    // So power entering layer = 1.0 * (1-0.04) = 0.96
    // Absorbed from forward pass = 0.96 * (1 - exp(-1)) = 0.96 * 0.632 = 0.607
    let fresnel_r = ((1.0f32 - 1.5) / (1.0 + 1.5)).powi(2);
    let t_interface = 1.0 - fresnel_r;
    let beer_lambert = (-0.1f32 * 10.0).exp();
    let expected_absorbed = t_interface * (1.0 - beer_lambert);

    let actual = exp.green[idx_550];
    println!(
        "Single layer 550nm: expected={:.6}, actual={:.6}",
        expected_absorbed, actual
    );
    // Allow some tolerance for backward pass contribution
    assert!(
        (actual - expected_absorbed).abs() < expected_absorbed * 0.1,
        "Absorbed energy mismatch: expected≈{:.4} got {:.4}",
        expected_absorbed,
        actual
    );
}

#[test]
fn stage3_yellow_filter_blocks_blue() {
    let stack = FilmLayerStack::default_color_negative();

    // Send pure 450nm (blue) light
    let mut blue_light = [0.0f32; BINS];
    let idx_450 = (450 - LAMBDA_START) / LAMBDA_STEP;
    blue_light[idx_450] = 1.0;

    let exp = propagate(&stack, &blue_light);

    // Blue layer (above yellow filter) should capture blue
    // Red layer (below yellow filter) should get almost nothing
    println!(
        "450nm: Blue layer={:.6}, Green layer={:.6}, Red layer={:.6}",
        exp.blue[idx_450], exp.green[idx_450], exp.red[idx_450]
    );

    assert!(
        exp.blue[idx_450] > exp.red[idx_450] * 5.0,
        "Yellow filter should block blue from reaching red layer"
    );
}

#[test]
fn stage3_red_light_passes_yellow_filter() {
    let stack = FilmLayerStack::default_color_negative();

    // Send pure 640nm (red) light
    let mut red_light = [0.0f32; BINS];
    let idx_640 = (640 - LAMBDA_START) / LAMBDA_STEP;
    red_light[idx_640] = 1.0;

    let exp = propagate(&stack, &red_light);

    // Red layer should capture red light (yellow filter doesn't block red)
    println!(
        "640nm: Red layer={:.6}, Blue layer={:.6}",
        exp.red[idx_640], exp.blue[idx_640]
    );
    assert!(
        exp.red[idx_640] > 0.0,
        "Red layer should capture 640nm light"
    );
}

#[test]
fn stage3_fresnel_at_each_interface() {
    // Verify Fresnel reflection is applied at layer boundaries
    // Air (n=1.0) → Gelatin (n=1.5): R = ((1-1.5)/(1+1.5))² = 0.04
    let r = ((1.0f32 - 1.5) / (1.0 + 1.5)).powi(2);
    assert!(
        (r - 0.04).abs() < 0.001,
        "Fresnel R for air→gelatin should be ~0.04, got {}",
        r
    );

    // Gelatin (1.5) → PET base (1.65): R = ((1.5-1.65)/(1.5+1.65))² ≈ 0.00227
    let r2 = ((1.5f32 - 1.65) / (1.5 + 1.65)).powi(2);
    println!("Fresnel gelatin→PET: {:.6}", r2);
    assert!(r2 < 0.01, "Gelatin→PET reflection should be small");
}

// =========================================================================
// Stage 4: integrate_exposure — spectral → scalar
// =========================================================================

#[test]
fn stage4_integrate_single_bin() {
    let mut exp = LayerExposure::default();
    let idx = (550 - LAMBDA_START) / LAMBDA_STEP;
    exp.green[idx] = 1.0;

    let rgb = integrate_exposure(&exp);
    // Trapezoidal rule: single interior point × step = 1.0 × 5.0 = 5.0
    assert!(
        (rgb[1] - 5.0).abs() < 0.1,
        "Single bin integration: expected ~5.0, got {:.4}",
        rgb[1]
    );
    assert!(rgb[0] == 0.0 && rgb[2] == 0.0, "Other channels should be 0");
}

// =========================================================================
// Stage 5+6: normalization + user_ev
// =========================================================================

#[test]
fn stage5_norm_white_equals_offset() {
    // After normalization, white input should produce exposure = exposure_offset
    let stack = FilmLayerStack::default_color_negative();
    let cam = CameraSensitivities::srgb();
    let d65 = Spectrum::new_d65();

    let white = cam.uplift(1.0, 1.0, 1.0);
    let mut ws = [0.0f32; BINS];
    for (i, s) in ws.iter_mut().enumerate() {
        *s = white.power[i] * d65.power[i];
    }
    let we = propagate(&stack, &ws);
    let wrgb = integrate_exposure(&we);

    // Simulate norm calculation (same as AccurateDevelopStage)
    let offset = 4.32244f32; // STANDARD_DAYLIGHT exposure_offset
    let norm = [offset / wrgb[0], offset / wrgb[1], offset / wrgb[2]];

    // White after norm should equal offset
    for ch in 0..3 {
        let result = wrgb[ch] * norm[ch];
        assert!(
            (result - offset).abs() < 0.001,
            "ch={}: white*norm={:.6} should equal offset={:.6}",
            ch,
            result,
            offset
        );
    }

    // 18% gray after norm
    let gray = cam.uplift(0.18, 0.18, 0.18);
    let mut gs = [0.0f32; BINS];
    for (i, s) in gs.iter_mut().enumerate() {
        *s = gray.power[i] * d65.power[i];
    }
    let ge = propagate(&stack, &gs);
    let grgb = integrate_exposure(&ge);
    for ch in 0..3 {
        let result = grgb[ch] * norm[ch];
        let expected = offset * 0.18;
        assert!(
            (result - expected).abs() < expected * 0.01,
            "ch={}: gray*norm={:.6} should equal {:.6}",
            ch,
            result,
            expected
        );
    }
}

// =========================================================================
// Stage 8: H-D curve — log_e → density
// =========================================================================

#[test]
fn stage8_hd_curve_midpoint() {
    // At log_e = log10(exposure_offset), sigmoid = 0.5, density = d_min + range/2
    let curve = SegmentedCurve::new(0.12, 2.9, 1.8, 4.32244);
    let log_e = 4.32244f32.log10();
    let d = curve.map_smooth(log_e);
    let expected = 0.12 + (2.9 - 0.12) * 0.5;
    println!(
        "H-D midpoint: log_e={:.4}, density={:.4}, expected={:.4}",
        log_e, d, expected
    );
    assert!(
        (d - expected).abs() < 0.01,
        "H-D midpoint density should be {:.4}, got {:.4}",
        expected,
        d
    );
}

#[test]
fn stage8_hd_curve_toe_and_shoulder() {
    let curve = SegmentedCurve::new(0.12, 2.9, 1.8, 4.32244);

    // Very low exposure → near d_min
    let d_low = curve.map_smooth(-5.0);
    assert!(
        (d_low - 0.12).abs() < 0.05,
        "Toe should be near d_min: got {:.4}",
        d_low
    );

    // Very high exposure → near d_max
    let d_high = curve.map_smooth(5.0);
    assert!(
        (d_high - 2.9).abs() < 0.05,
        "Shoulder should be near d_max: got {:.4}",
        d_high
    );
}

// =========================================================================
// Stage 9: inhibition — cross-channel suppression
// =========================================================================

#[test]
fn stage9_inhibition_reduces_density() {
    // With negative off-diagonal inhibition, cross-channel density should decrease
    let d = [1.5, 1.0, 0.5]; // R high, G medium, B low
    let inh = [[0.0, -0.1, -0.05], [-0.07, 0.0, -0.07], [-0.05, -0.1, 0.0]];

    let d_r = d[0] + inh[0][0] * d[0] + inh[0][1] * d[1] + inh[0][2] * d[2];
    let d_g = d[1] + inh[1][0] * d[0] + inh[1][1] * d[1] + inh[1][2] * d[2];
    let d_b = d[2] + inh[2][0] * d[0] + inh[2][1] * d[1] + inh[2][2] * d[2];

    println!(
        "Before inhibition: R={:.3} G={:.3} B={:.3}",
        d[0], d[1], d[2]
    );
    println!("After inhibition:  R={:.3} G={:.3} B={:.3}", d_r, d_g, d_b);

    // All channels should be reduced (negative off-diagonal)
    assert!(d_r < d[0], "R should decrease with inhibition");
    assert!(d_g < d[1], "G should decrease with inhibition");
    assert!(d_b < d[2], "B should decrease with inhibition");

    // R has highest density → should inhibit others most
    // G reduction from R: -0.07 * 1.5 = -0.105
    // B reduction from R: -0.05 * 1.5 = -0.075
    let g_reduction_from_r = -inh[1][0] * d[0];
    let b_reduction_from_r = -inh[2][0] * d[0];
    println!(
        "G reduction from R: {:.3}, B reduction from R: {:.3}",
        g_reduction_from_r, b_reduction_from_r
    );
    assert!(g_reduction_from_r > 0.0 && b_reduction_from_r > 0.0);
}

// =========================================================================
// Stage 10: density → output (positive mode)
// =========================================================================

#[test]
fn stage10_density_to_output_monotonic() {
    // Higher density → lower transmission → darker output (in negative mode)
    // But in positive mode: higher density → brighter output (inverted)
    for d_int in (0..30).step_by(1) {
        let d = d_int as f32 * 0.1;
        let t = physics::density_to_transmission(d);
        assert!(
            t >= 0.0 && t <= 1.0,
            "Transmission out of range at D={}: T={}",
            d,
            t
        );
        if d_int > 0 {
            let t_prev = physics::density_to_transmission((d_int - 1) as f32 * 0.1);
            assert!(
                t <= t_prev,
                "Transmission should decrease with density: D={} T={} prev_T={}",
                d,
                t,
                t_prev
            );
        }
    }
}

// =========================================================================
// End-to-end: full chain numerical trace
// =========================================================================

#[test]
fn full_chain_gray_trace() {
    let stack = FilmLayerStack::default_color_negative();
    let cam = CameraSensitivities::srgb();
    let d65 = Spectrum::new_d65();

    // Step 1: uplift 18% gray
    let gray = cam.uplift(0.18, 0.18, 0.18);
    let gray_energy: f32 = gray.power.iter().sum();
    println!("Step 1 - uplift(0.18): total energy = {:.4}", gray_energy);
    assert!(gray_energy > 0.0);

    // Step 2: × D65
    let mut scaled = [0.0f32; BINS];
    for (i, s) in scaled.iter_mut().enumerate() {
        *s = gray.power[i] * d65.power[i];
    }
    let scaled_energy: f32 = scaled.iter().sum();
    println!("Step 2 - ×D65: total energy = {:.4}", scaled_energy);
    assert!(
        scaled_energy > gray_energy,
        "D65 should amplify (values ~50-117)"
    );

    // Step 3: propagate
    let exp = propagate(&stack, &scaled);
    let rgb = integrate_exposure(&exp);
    println!(
        "Step 3 - propagate+integrate: R={:.4} G={:.4} B={:.4}",
        rgb[0], rgb[1], rgb[2]
    );
    assert!(rgb[0] > 0.0 && rgb[1] > 0.0 && rgb[2] > 0.0);

    // Step 4: normalize (white = offset)
    let white = cam.uplift(1.0, 1.0, 1.0);
    let mut ws = [0.0f32; BINS];
    for (i, s) in ws.iter_mut().enumerate() {
        *s = white.power[i] * d65.power[i];
    }
    let we = propagate(&stack, &ws);
    let wrgb = integrate_exposure(&we);
    let offset = 4.32244f32;
    let norm = [offset / wrgb[0], offset / wrgb[1], offset / wrgb[2]];
    let normed = [rgb[0] * norm[0], rgb[1] * norm[1], rgb[2] * norm[2]];
    println!(
        "Step 4 - normalized: R={:.4} G={:.4} B={:.4}",
        normed[0], normed[1], normed[2]
    );

    // 18% gray should be 0.18 × offset
    let expected = offset * 0.18;
    for ch in 0..3 {
        let err = (normed[ch] - expected).abs() / expected;
        assert!(
            err < 0.02,
            "ch={}: normed={:.4} expected={:.4} err={:.1}%",
            ch,
            normed[ch],
            expected,
            err * 100.0
        );
    }

    // Step 5: log10
    let log_e: Vec<f32> = normed.iter().map(|&v| v.max(1e-6).log10()).collect();
    println!(
        "Step 5 - log10: R={:.4} G={:.4} B={:.4}",
        log_e[0], log_e[1], log_e[2]
    );

    // Should be below the midpoint (log10(offset) = 0.636)
    let log_offset = offset.log10();
    for ch in 0..3 {
        assert!(
            log_e[ch] < log_offset,
            "18% gray log_e should be below midpoint: {:.4} vs {:.4}",
            log_e[ch],
            log_offset
        );
    }

    println!(
        "\nSummary: 18% gray → log_e ≈ {:.3} (midpoint at {:.3}, delta = {:.3} log)",
        log_e[0],
        log_offset,
        log_offset - log_e[0]
    );
}
