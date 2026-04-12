//! End-to-end integration tests for Accurate simulation mode.
//!
//! These test the full pipeline: input image → Accurate develop → output image.

use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, SimulationMode,
    WhiteBalanceMode,
};
use image::{Rgb, RgbImage};

fn accurate_config() -> SimulationConfig {
    SimulationConfig {
        simulation_mode: SimulationMode::Accurate,
        exposure_time: 1.0,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 0.0,
        ..Default::default()
    }
}

fn process_uniform(r: u8, g: u8, b: u8) -> Rgb<u8> {
    let img = RgbImage::from_fn(50, 50, |_, _| Rgb([r, g, b]));
    let film = STANDARD_DAYLIGHT();
    let mut config = accurate_config();
    config.exposure_time = estimate_exposure_time(&img, &film);
    let out = process_image(&img, &film, &config);
    *out.get_pixel(25, 25)
}

// =========================================================================
// 1. Gray gradient: monotonic + neutral
// =========================================================================
#[test]
fn accurate_gray_gradient_monotonic() {
    let film = STANDARD_DAYLIGHT();
    let width = 256u32;
    let height = 50u32;
    let mut input = RgbImage::new(width, height);
    for x in 0..width {
        let val = x as u8;
        for y in 0..height {
            input.put_pixel(x, y, Rgb([val, val, val]));
        }
    }

    let mut config = accurate_config();
    config.exposure_time = estimate_exposure_time(&input, &film);
    let output = process_image(&input, &film, &config);

    // Check monotonicity: luma should increase with input
    let mut prev_luma = 0.0f32;
    let mut max_drift = 0i16;
    let y_mid = height / 2;

    for x in 0..width {
        let p = output.get_pixel(x, y_mid);
        let luma = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;

        if x > 10 {
            // skip very dark pixels (toe noise)
            assert!(
                luma >= prev_luma - 2.0,
                "Monotonicity: x={} luma={:.1} < prev={:.1}",
                x,
                luma,
                prev_luma
            );
        }
        prev_luma = luma;

        // Neutrality check for mid-tones
        let r = p[0] as i16;
        let g = p[1] as i16;
        let b = p[2] as i16;
        let drift = (r - g).abs().max((r - b).abs()).max((g - b).abs());
        max_drift = max_drift.max(drift);
    }

    println!("Accurate gray gradient: max neutral drift = {}", max_drift);
    assert!(
        max_drift <= 25,
        "Neutral axis drift too high: {}",
        max_drift
    );
}

// =========================================================================
// 2. Color card: R/G/B/C/M/Y hue preservation
// =========================================================================
#[test]
fn accurate_color_card_hue() {
    // (input_r, input_g, input_b, expected_dominant_channel)
    let colors = [
        ("Red", 200, 0, 0, 0usize),  // R dominant
        ("Green", 0, 200, 0, 1),     // G dominant
        ("Blue", 0, 0, 200, 2),      // B dominant
        ("Cyan", 0, 200, 200, 1),    // G or B dominant, R lowest
        ("Magenta", 200, 0, 200, 0), // R or B dominant, G lowest
        ("Yellow", 200, 200, 0, 0),  // R or G dominant, B lowest
    ];

    for (name, r, g, b, _dominant) in colors {
        let p = process_uniform(r, g, b);
        let channels = [p[0], p[1], p[2]];
        let max_ch = channels
            .iter()
            .enumerate()
            .max_by_key(|(_, v)| *v)
            .unwrap()
            .0;

        println!(
            "{:8}: input=({},{},{}) → output=({},{},{}), dominant={}",
            name, r, g, b, p[0], p[1], p[2], max_ch
        );

        // For primary colors, the dominant output channel should match
        if r > 0 && g == 0 && b == 0 {
            assert!(
                p[0] > p[1] && p[0] > p[2],
                "{}: R should dominate, got {:?}",
                name,
                p
            );
        }
        if g > 0 && r == 0 && b == 0 {
            assert!(
                p[1] > p[0] && p[1] > p[2],
                "{}: G should dominate, got {:?}",
                name,
                p
            );
        }
        if b > 0 && r == 0 && g == 0 {
            assert!(
                p[2] > p[0] && p[2] > p[1],
                "{}: B should dominate, got {:?}",
                name,
                p
            );
        }

        // For secondary colors, the complementary channel should be lowest
        if name == "Cyan" {
            assert!(
                p[0] <= p[1] && p[0] <= p[2],
                "Cyan: R should be lowest, got {:?}",
                p
            );
        }
        if name == "Magenta" {
            assert!(
                p[1] <= p[0] && p[1] <= p[2],
                "Magenta: G should be lowest, got {:?}",
                p
            );
        }
        if name == "Yellow" {
            assert!(
                p[2] <= p[0] && p[2] <= p[1],
                "Yellow: B should be lowest, got {:?}",
                p
            );
        }
    }
}

// =========================================================================
// 3. White → output close to white
// =========================================================================
#[test]
fn accurate_white_output() {
    let p = process_uniform(255, 255, 255);
    println!("White → ({}, {}, {})", p[0], p[1], p[2]);

    let drift = ((p[0] as i16 - p[1] as i16).abs())
        .max((p[0] as i16 - p[2] as i16).abs())
        .max((p[1] as i16 - p[2] as i16).abs());

    assert!(drift <= 15, "White not neutral: {:?} drift={}", p, drift);
    // Should be reasonably bright (not black, not clipped)
    let avg = (p[0] as u16 + p[1] as u16 + p[2] as u16) / 3;
    assert!(avg > 20, "White too dark: avg={}", avg);
}

// =========================================================================
// 4. 18% gray → reasonable mid-gray output
// =========================================================================
#[test]
fn accurate_midgray_output() {
    // sRGB 119 ≈ 18% linear gray
    let p = process_uniform(119, 119, 119);
    println!("18% gray → ({}, {}, {})", p[0], p[1], p[2]);

    let avg = (p[0] as u16 + p[1] as u16 + p[2] as u16) / 3;
    assert!(
        (20..=220).contains(&avg),
        "18% gray output out of range: avg={} (expected 20-220)",
        avg
    );

    let drift = ((p[0] as i16 - p[1] as i16).abs()).max((p[0] as i16 - p[2] as i16).abs());
    assert!(drift <= 20, "18% gray not neutral: {:?} drift={}", p, drift);
}
