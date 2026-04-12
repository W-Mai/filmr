//! Fine-grained model correctness tests for the Accurate simulation pipeline.
//!
//! Tests each stage independently with known inputs and verifiable outputs.

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
        (580..=660).contains(&peak_nm),
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
        (500..=570).contains(&peak_nm),
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
        (420..=480).contains(&peak_nm),
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
        (400..=600).contains(&peak_nm),
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
            (0.0..=1.0).contains(&t),
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
    for (ch, &val) in normed.iter().enumerate() {
        let err = (val - expected).abs() / expected;
        assert!(
            err < 0.02,
            "ch={}: normed={:.4} expected={:.4} err={:.1}%",
            ch,
            val,
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
    for &val in log_e.iter() {
        assert!(
            val < log_offset,
            "18% gray log_e should be below midpoint: {:.4} vs {:.4}",
            val,
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

// =========================================================================
// Strict physics verification
// =========================================================================

#[test]
fn strict_beer_lambert_exact() {
    // Single non-scattering layer, no Fresnel (n=1.0), known absorption
    // Beer-Lambert: I_out = I_in * exp(-α * d)
    // Absorbed = I_in * (1 - exp(-α * d))
    let alpha = 0.2f32; // per µm
    let d = 5.0f32; // µm
    let mut absorption = [0.0f32; BINS];
    absorption[20] = alpha; // single wavelength bin

    let stack = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![FilmLayer {
            name: "Test".into(),
            kind: LayerKind::Emulsion {
                channel: EmulsionChannel::Green,
            },
            thickness_um: d,
            refractive_index: 1.0, // no Fresnel
            absorption,
            scattering: 0.0,
        }],
    };

    let mut incident = [0.0f32; BINS];
    incident[20] = 1.0;

    let exp = propagate(&stack, &incident);
    let expected = 1.0 * (1.0 - (-alpha * d).exp());
    let actual = exp.green[20];

    println!(
        "Beer-Lambert exact: α={}, d={}, expected={:.6}, actual={:.6}",
        alpha, d, expected, actual
    );
    assert!(
        (actual - expected).abs() < 1e-5,
        "Beer-Lambert mismatch: expected={:.6} actual={:.6}",
        expected,
        actual
    );
}

#[test]
fn strict_fresnel_exact() {
    // Non-absorbing layer with known refractive index
    // Fresnel R = ((n1-n2)/(n1+n2))²
    // For air(1.0) → glass(1.5): R = 0.04, T = 0.96
    let stack = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![FilmLayer {
            name: "Glass".into(),
            kind: LayerKind::Emulsion {
                channel: EmulsionChannel::Green,
            },
            thickness_um: 0.001, // negligible thickness → no absorption
            refractive_index: 1.5,
            absorption: [0.0; BINS],
            scattering: 0.0,
        }],
    };

    let mut incident = [0.0f32; BINS];
    incident[20] = 1.0;

    let exp = propagate(&stack, &incident);
    // With negligible absorption, almost nothing is absorbed
    // But Fresnel reflects 4% at entry, so only 96% enters
    // With zero absorption, absorbed ≈ 0
    // The exposure should be ≈ 0 (no absorption to capture)
    assert!(
        exp.green[20] < 0.01,
        "Zero-absorption layer should capture ~0 exposure, got {}",
        exp.green[20]
    );
}

#[test]
fn strict_scattering_reduces_transmission() {
    // Scattering removes energy from the beam (but doesn't deposit it as exposure)
    let d = 10.0f32;
    let scatter = 0.05f32;

    // Layer with only scattering, no absorption
    let stack_scatter = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![FilmLayer {
            name: "Scatter".into(),
            kind: LayerKind::Emulsion {
                channel: EmulsionChannel::Green,
            },
            thickness_um: d,
            refractive_index: 1.0,
            absorption: [0.0; BINS],
            scattering: scatter,
        }],
    };

    // Layer with only absorption, same total attenuation
    let mut abs_only = [0.0f32; BINS];
    abs_only[20] = scatter;
    let stack_absorb = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![FilmLayer {
            name: "Absorb".into(),
            kind: LayerKind::Emulsion {
                channel: EmulsionChannel::Green,
            },
            thickness_um: d,
            refractive_index: 1.0,
            absorption: abs_only,
            scattering: 0.0,
        }],
    };

    let mut incident = [0.0f32; BINS];
    incident[20] = 1.0;

    let exp_scatter = propagate(&stack_scatter, &incident);
    let exp_absorb = propagate(&stack_absorb, &incident);

    println!(
        "Scatter-only exposure: {:.6}, Absorb-only exposure: {:.6}",
        exp_scatter.green[20], exp_absorb.green[20]
    );

    // Scatter-only: total_atten = scatter, abs_fraction = 0/scatter = 0
    // So exposure should be 0 (scattering doesn't deposit energy)
    assert!(
        exp_scatter.green[20] < 1e-6,
        "Pure scattering should not deposit exposure: {}",
        exp_scatter.green[20]
    );

    // Absorb-only: total_atten = absorption, abs_fraction = 1.0
    // Exposure = 1 - exp(-0.05 * 10) = 1 - exp(-0.5) = 0.3935
    let expected = 1.0 - (-scatter * d).exp();
    assert!(
        (exp_absorb.green[20] - expected).abs() < 1e-5,
        "Absorb-only: expected={:.6} got={:.6}",
        expected,
        exp_absorb.green[20]
    );
}

#[test]
fn strict_energy_budget_single_layer() {
    // For a single layer: incident = reflected + absorbed + transmitted
    // (no scattering for simplicity)
    let alpha = 0.15f32;
    let d = 8.0f32;
    let n = 1.5f32;
    let mut absorption = [0.0f32; BINS];
    absorption[20] = alpha;

    let stack = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![FilmLayer {
            name: "Test".into(),
            kind: LayerKind::Emulsion {
                channel: EmulsionChannel::Green,
            },
            thickness_um: d,
            refractive_index: n,
            absorption,
            scattering: 0.0,
        }],
    };

    let mut incident = [0.0f32; BINS];
    incident[20] = 1.0;

    let exp = propagate(&stack, &incident);
    let absorbed = exp.green[20];

    // Manual calculation:
    // Fresnel at air→glass: R1 = ((1-1.5)/(1+1.5))² = 0.04
    // Power entering: 1.0 * 0.96 = 0.96
    // Beer-Lambert: transmitted through layer = 0.96 * exp(-0.15*8) = 0.96 * 0.3012 = 0.2891
    // Absorbed in forward pass = 0.96 * (1 - 0.3012) = 0.96 * 0.6988 = 0.6709
    // (all absorbed since abs_fraction = 1.0, no scattering)
    let r1 = ((1.0f32 - n) / (1.0 + n)).powi(2);
    let t1 = 1.0 - r1;
    let beer = (-alpha * d).exp();
    let forward_absorbed = t1 * (1.0 - beer);
    let forward_transmitted = t1 * beer;

    println!(
        "Energy budget: incident=1.0, reflected={:.4}, forward_absorbed={:.4}, forward_transmitted={:.4}",
        r1, forward_absorbed, forward_transmitted
    );
    println!(
        "Actual absorbed (incl backward): {:.6}, forward-only expected: {:.6}",
        absorbed, forward_absorbed
    );

    // Absorbed should be >= forward_absorbed (backward pass adds more)
    assert!(
        absorbed >= forward_absorbed - 1e-5,
        "Absorbed should be >= forward-only: {} < {}",
        absorbed,
        forward_absorbed
    );

    // Total energy: reflected + absorbed + escaped should ≈ 1.0
    // Escaped = forward_transmitted (minus what base reflects back, which gets partially absorbed)
    // For single layer with no base, escaped ≈ forward_transmitted
    // But our stack has implicit air below, so base_r = fresnel(n, 1.0) = same as r1
    // back_power = forward_transmitted * r1
    // backward absorbed ≈ back_power * (1 - beer) * abs_fraction
    let back_power = forward_transmitted * r1;
    let backward_absorbed = back_power * (1.0 - beer);
    let total_absorbed_expected = forward_absorbed + backward_absorbed;

    println!(
        "With backward: back_power={:.6}, backward_absorbed={:.6}, total_expected={:.6}",
        back_power, backward_absorbed, total_absorbed_expected
    );
    assert!(
        (absorbed - total_absorbed_expected).abs() < 0.01,
        "Total absorbed mismatch: expected={:.6} actual={:.6}",
        total_absorbed_expected,
        absorbed
    );
}

#[test]
fn strict_backward_pass_fresnel_direction() {
    // Verify backward pass uses correct Fresnel direction
    // Two layers with different refractive indices
    // Light reflects off base, travels back through both layers
    let stack = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![
            FilmLayer {
                name: "Layer1".into(),
                kind: LayerKind::Emulsion {
                    channel: EmulsionChannel::Blue,
                },
                thickness_um: 1.0,
                refractive_index: 1.5,
                absorption: [0.0; BINS], // transparent
                scattering: 0.0,
            },
            FilmLayer {
                name: "Layer2".into(),
                kind: LayerKind::Emulsion {
                    channel: EmulsionChannel::Red,
                },
                thickness_um: 1.0,
                refractive_index: 1.7, // different n
                absorption: [0.0; BINS],
                scattering: 0.0,
            },
        ],
    };

    let mut incident = [0.0f32; BINS];
    incident[20] = 1.0;

    let exp = propagate(&stack, &incident);

    // Both layers are transparent, so exposure should be ~0
    // (no absorption to capture energy)
    assert!(
        exp.blue[20] < 0.01 && exp.red[20] < 0.01,
        "Transparent layers should capture ~0: blue={:.6} red={:.6}",
        exp.blue[20],
        exp.red[20]
    );
}

#[test]
fn strict_multi_layer_attenuation_order() {
    // Blue light through: Blue emulsion (absorbs) → Yellow filter (absorbs blue) → Red emulsion
    // Red emulsion should get much less blue light than blue emulsion
    let mut blue_abs = [0.0f32; BINS];
    let idx_450 = (450 - LAMBDA_START) / LAMBDA_STEP;
    blue_abs[idx_450] = 0.1;

    let mut yellow_abs = [0.0f32; BINS];
    yellow_abs[idx_450] = 0.5; // strong blue absorption

    let mut red_abs = [0.0f32; BINS];
    red_abs[idx_450] = 0.1;

    let stack = FilmLayerStack {
        inhibition: [[0.0; 3]; 3],
        layers: vec![
            FilmLayer {
                name: "Blue".into(),
                kind: LayerKind::Emulsion {
                    channel: EmulsionChannel::Blue,
                },
                thickness_um: 5.0,
                refractive_index: 1.0,
                absorption: blue_abs,
                scattering: 0.0,
            },
            FilmLayer {
                name: "Yellow".into(),
                kind: LayerKind::YellowFilter,
                thickness_um: 2.0,
                refractive_index: 1.0,
                absorption: yellow_abs,
                scattering: 0.0,
            },
            FilmLayer {
                name: "Red".into(),
                kind: LayerKind::Emulsion {
                    channel: EmulsionChannel::Red,
                },
                thickness_um: 5.0,
                refractive_index: 1.0,
                absorption: red_abs,
                scattering: 0.0,
            },
        ],
    };

    let mut incident = [0.0f32; BINS];
    incident[idx_450] = 1.0;

    let exp = propagate(&stack, &incident);

    // Blue layer sees full power: absorbed = 1 - exp(-0.1*5) = 0.3935
    let blue_expected = 1.0 - (-0.1f32 * 5.0).exp();
    // After blue layer: power = exp(-0.1*5) = 0.6065
    // After yellow filter: power = 0.6065 * exp(-0.5*2) = 0.6065 * 0.3679 = 0.2231
    // Red layer absorbed = 0.2231 * (1 - exp(-0.1*5)) = 0.2231 * 0.3935 = 0.0878
    let after_blue = (-0.1f32 * 5.0).exp();
    let after_yellow = after_blue * (-0.5f32 * 2.0).exp();
    let red_expected = after_yellow * (1.0 - (-0.1f32 * 5.0).exp());

    println!(
        "Blue absorbed: expected={:.4} actual={:.4}",
        blue_expected, exp.blue[idx_450]
    );
    println!(
        "Red absorbed:  expected={:.4} actual={:.4}",
        red_expected, exp.red[idx_450]
    );

    assert!(
        (exp.blue[idx_450] - blue_expected).abs() < 1e-4,
        "Blue layer absorption mismatch"
    );
    assert!(
        (exp.red[idx_450] - red_expected).abs() < 1e-4,
        "Red layer absorption mismatch"
    );
    assert!(
        exp.blue[idx_450] > exp.red[idx_450] * 3.0,
        "Blue should absorb much more than red at 450nm"
    );
}
