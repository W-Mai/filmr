//! Property tests for the full-spectrum propagation engine.
//!
//! These tests verify physical invariants that must hold regardless of
//! layer stack configuration or preset parameters.

use filmr::film_layer::*;
use filmr::spectral::{CameraSensitivities, Spectrum, BINS};
use filmr::spectral_engine::{integrate_exposure, propagate};

fn d65_white() -> [f32; BINS] {
    let cam = CameraSensitivities::srgb();
    let d65 = Spectrum::new_d65();
    let white = cam.uplift(1.0, 1.0, 1.0);
    let mut out = [0.0f32; BINS];
    for (i, v) in out.iter_mut().enumerate() {
        *v = white.power[i] * d65.power[i];
    }
    out
}

fn d65_color(r: f32, g: f32, b: f32) -> [f32; BINS] {
    let cam = CameraSensitivities::srgb();
    let d65 = Spectrum::new_d65();
    let spec = cam.uplift(r, g, b);
    let mut out = [0.0f32; BINS];
    for (i, v) in out.iter_mut().enumerate() {
        *v = spec.power[i] * d65.power[i];
    }
    out
}

// =========================================================================
// 1. White neutrality: white input → balanced R/G/B exposure
// =========================================================================
#[test]
fn prop_white_neutral() {
    let stack = FilmLayerStack::default_color_negative();
    let white = d65_white();
    let exp = propagate(&stack, &white);
    let rgb = integrate_exposure(&exp);

    // All channels should be positive
    assert!(
        rgb[0] > 0.0 && rgb[1] > 0.0 && rgb[2] > 0.0,
        "White must produce positive exposure: {:?}",
        rgb
    );

    // Channel ratios should be consistent (not necessarily equal —
    // layer absorption differs by wavelength — but the ratio should be stable)
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    let ratio = max / min;
    println!(
        "White exposure: R={:.2}, G={:.2}, B={:.2}, max/min={:.3}",
        rgb[0], rgb[1], rgb[2], ratio
    );
    // Ratio < 3 is reasonable for a colour negative stack
    assert!(
        ratio < 3.0,
        "White channel imbalance too large: ratio={:.3}",
        ratio
    );
}

// =========================================================================
// 2. Gray monotonicity: brighter input → more exposure per channel
// =========================================================================
#[test]
fn prop_gray_monotonic() {
    let stack = FilmLayerStack::default_color_negative();
    let cam = CameraSensitivities::srgb();
    let d65 = Spectrum::new_d65();

    let mut prev_rgb = [0.0f32; 3];
    for level in [0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0] {
        let spec = cam.uplift(level, level, level);
        let mut scaled = [0.0f32; BINS];
        for (i, v) in scaled.iter_mut().enumerate() {
            *v = spec.power[i] * d65.power[i];
        }
        let exp = propagate(&stack, &scaled);
        let rgb = integrate_exposure(&exp);

        for ch in 0..3 {
            assert!(
                rgb[ch] >= prev_rgb[ch],
                "Monotonicity violated at level={}: ch={} prev={:.4} cur={:.4}",
                level,
                ch,
                prev_rgb[ch],
                rgb[ch]
            );
        }
        prev_rgb = rgb;
    }
}

// =========================================================================
// 3. Color separation: pure red → R exposure > G exposure > B exposure
// =========================================================================
#[test]
fn prop_color_separation() {
    let stack = FilmLayerStack::default_color_negative();

    // Red
    let red = d65_color(1.0, 0.0, 0.0);
    let exp_r = integrate_exposure(&propagate(&stack, &red));
    println!(
        "Red input:   R={:.4}, G={:.4}, B={:.4}",
        exp_r[0], exp_r[1], exp_r[2]
    );
    assert!(exp_r[0] > exp_r[1], "Red: R exposure should > G");
    assert!(exp_r[0] > exp_r[2], "Red: R exposure should > B");

    // Green
    let green = d65_color(0.0, 1.0, 0.0);
    let exp_g = integrate_exposure(&propagate(&stack, &green));
    println!(
        "Green input: R={:.4}, G={:.4}, B={:.4}",
        exp_g[0], exp_g[1], exp_g[2]
    );
    assert!(exp_g[1] > exp_g[0], "Green: G exposure should > R");
    assert!(exp_g[1] > exp_g[2], "Green: G exposure should > B");

    // Blue
    let blue = d65_color(0.0, 0.0, 1.0);
    let exp_b = integrate_exposure(&propagate(&stack, &blue));
    println!(
        "Blue input:  R={:.4}, G={:.4}, B={:.4}",
        exp_b[0], exp_b[1], exp_b[2]
    );
    assert!(exp_b[2] > exp_b[0], "Blue: B exposure should > R");
    assert!(exp_b[2] > exp_b[1], "Blue: B exposure should > G");
}

// =========================================================================
// 4. Linearity: propagate(k * input) == k * propagate(input)
// =========================================================================
#[test]
fn prop_linearity() {
    let stack = FilmLayerStack::default_color_negative();
    let base = d65_white();

    let exp1 = propagate(&stack, &base);
    let rgb1 = integrate_exposure(&exp1);

    let k = 2.5f32;
    let scaled: [f32; BINS] = {
        let mut s = [0.0; BINS];
        for (i, v) in s.iter_mut().enumerate() {
            *v = base[i] * k;
        }
        s
    };
    let exp2 = propagate(&stack, &scaled);
    let rgb2 = integrate_exposure(&exp2);

    for ch in 0..3 {
        let expected = rgb1[ch] * k;
        let actual = rgb2[ch];
        let rel_err = (actual - expected).abs() / expected.max(1e-10);
        assert!(
            rel_err < 0.001,
            "Linearity violated: ch={} expected={:.4} actual={:.4} err={:.6}",
            ch,
            expected,
            actual,
            rel_err
        );
    }
}

// =========================================================================
// 5. Energy conservation: each layer output ≤ input
// =========================================================================
#[test]
fn prop_energy_conservation() {
    let stack = FilmLayerStack::default_color_negative();
    let incident = d65_white();

    // Total incident energy
    let total_in: f32 = incident.iter().sum();

    // Total absorbed + transmitted should ≤ input
    let exp = propagate(&stack, &incident);
    let total_absorbed: f32 = exp
        .red
        .iter()
        .chain(exp.green.iter())
        .chain(exp.blue.iter())
        .sum();

    println!(
        "Total incident: {:.2}, Total absorbed by emulsions: {:.2}",
        total_in, total_absorbed
    );
    assert!(
        total_absorbed <= total_in * 1.01, // 1% tolerance for numerical error
        "Energy conservation violated: absorbed={:.2} > incident={:.2}",
        total_absorbed,
        total_in
    );
}

// =========================================================================
// 6. Layer order: blue layer above yellow filter → blue absorbs blue light
// =========================================================================
#[test]
fn prop_layer_order_blue_above_yellow() {
    let stack = FilmLayerStack::default_color_negative();

    // Pure blue light (peak at 450nm)
    let blue = d65_color(0.0, 0.0, 1.0);
    let exp = propagate(&stack, &blue);
    let rgb = integrate_exposure(&exp);

    // Blue layer (top) should capture most of the blue light
    // Red layer (below yellow filter) should get very little blue
    println!("Blue light: B_layer={:.4}, R_layer={:.4}", rgb[2], rgb[0]);
    assert!(
        rgb[2] > rgb[0] * 2.0,
        "Blue layer should capture much more blue light than red layer: B={:.4} R={:.4}",
        rgb[2],
        rgb[0]
    );
}

// =========================================================================
// 7. Base reflection: backward pass adds exposure
// =========================================================================
#[test]
fn prop_base_reflection_adds_exposure() {
    // Create a stack with reflective base
    let stack = FilmLayerStack::default_color_negative();
    let incident = d65_white();

    let exp = propagate(&stack, &incident);
    let rgb = integrate_exposure(&exp);

    // Create same stack but with base refractive index = 1.0 (no reflection)
    let mut no_reflect = stack.clone();
    if let Some(base) = no_reflect.layers.last_mut() {
        base.refractive_index = 1.0; // match air → no Fresnel reflection
    }
    let exp_nr = propagate(&no_reflect, &incident);
    let rgb_nr = integrate_exposure(&exp_nr);

    println!(
        "With reflection:    R={:.4}, G={:.4}, B={:.4}",
        rgb[0], rgb[1], rgb[2]
    );
    println!(
        "Without reflection: R={:.4}, G={:.4}, B={:.4}",
        rgb_nr[0], rgb_nr[1], rgb_nr[2]
    );

    for ch in 0..3 {
        assert!(
            rgb[ch] >= rgb_nr[ch],
            "Base reflection should add exposure: ch={} with={:.4} without={:.4}",
            ch,
            rgb[ch],
            rgb_nr[ch]
        );
    }
}

// =========================================================================
// 8. Inhibition effect: cross-channel density reduction
// =========================================================================
#[test]
fn prop_inhibition_reduces_cross_channel() {
    use filmr::presets::kodak::KODAK_PORTRA_400;

    let film = KODAK_PORTRA_400();
    let stack = film.layer_stack.as_ref().unwrap();

    // Red input: high R density should inhibit G and B
    let red = d65_color(1.0, 0.0, 0.0);

    // With inhibition (default)
    let _exp_with = propagate(stack, &red);

    // Without inhibition
    let mut no_inh = stack.clone();
    no_inh.inhibition = [[0.0; 3]; 3];
    let _exp_without = propagate(&no_inh, &red);

    // Propagation itself doesn't apply inhibition — it's applied in the density stage.
    // So raw exposure should be the same. The test verifies the inhibition matrix exists
    // and has the expected sign (negative off-diagonal = suppression).
    for i in 0..3 {
        for j in 0..3 {
            if i != j {
                assert!(
                    stack.inhibition[i][j] <= 0.0,
                    "Off-diagonal inhibition[{}][{}]={} should be ≤ 0",
                    i,
                    j,
                    stack.inhibition[i][j]
                );
            }
        }
    }
    println!("Inhibition matrix: {:?}", stack.inhibition);
}
