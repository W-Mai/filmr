use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, SimulationMode,
    WhiteBalanceMode,
};
use image::{Rgb, RgbImage};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_gray_gradient() -> RgbImage {
    let (width, height) = (256, 50);
    let mut img = RgbImage::new(width, height);
    for x in 0..width {
        let val = x as u8;
        for y in 0..height {
            img.put_pixel(x, y, Rgb([val, val, val]));
        }
    }
    img
}

fn base_config(mode: SimulationMode) -> SimulationConfig {
    SimulationConfig {
        simulation_mode: mode,
        exposure_time: 1.0,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Auto,
        white_balance_strength: 1.0,
        ..Default::default()
    }
}

fn run_neutral_axis(mode: SimulationMode, label: &str) {
    let input = make_gray_gradient();
    let film = STANDARD_DAYLIGHT();
    let mut config = base_config(mode);
    let t = estimate_exposure_time(&input, &film);
    config.exposure_time = t;

    let output = process_image(&input, &film, &config);

    // Accurate mode has more physical non-linearity → slightly wider tolerance
    let tolerance: i16 = match mode {
        SimulationMode::Fast => 15,
        SimulationMode::Accurate => 20,
    };
    let mut max_drift = 0i16;

    for p in output.pixels() {
        let r = p[0] as i16;
        let g = p[1] as i16;
        let b = p[2] as i16;

        let drift = (r - g).abs().max((r - b).abs()).max((g - b).abs());
        max_drift = max_drift.max(drift);

        if r > 20 && r < 235 {
            assert!(
                drift <= tolerance,
                "[{}] Neutral axis drift too high: R={}, G={}, B={} (drift={})",
                label,
                r,
                g,
                b,
                drift
            );
        }
    }
    println!("[{}] Max neutral axis drift: {}", label, max_drift);
}

fn run_channel_integrity(mode: SimulationMode, label: &str) {
    let input = RgbImage::from_fn(50, 50, |_, _| Rgb([200, 0, 0]));
    let film = STANDARD_DAYLIGHT();
    let mut config = SimulationConfig {
        simulation_mode: mode,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 0.0,
        ..Default::default()
    };
    let t = estimate_exposure_time(&input, &film);
    config.exposure_time = t;

    let output = process_image(&input, &film, &config);
    let center = output.get_pixel(25, 25);
    println!("[{}] Red input → output: {:?}", label, center);

    assert!(
        center[0] > center[1],
        "[{}] Red input should yield R > G, got {:?}",
        label,
        center
    );
    assert!(
        center[0] > center[2],
        "[{}] Red input should yield R > B, got {:?}",
        label,
        center
    );
}

fn run_white_gray_check(mode: SimulationMode, label: &str) {
    let film = STANDARD_DAYLIGHT();

    // White
    let white_img = RgbImage::from_fn(50, 50, |_, _| Rgb([255, 255, 255]));
    let mut config = base_config(mode);
    let t = estimate_exposure_time(&white_img, &film);
    config.exposure_time = t;
    let white_out = process_image(&white_img, &film, &config);
    let wp = white_out.get_pixel(25, 25);
    println!("[{}] White → {:?}", label, wp);

    // Gray
    let gray_img = RgbImage::from_fn(50, 50, |_, _| Rgb([128, 128, 128]));
    config.exposure_time = estimate_exposure_time(&gray_img, &film);
    let gray_out = process_image(&gray_img, &film, &config);
    let gp = gray_out.get_pixel(25, 25);
    println!("[{}] Gray  → {:?}", label, gp);

    // White should be brighter than gray
    assert!(wp[0] >= gp[0], "[{}] White should be >= Gray in R", label);

    // Gray should be roughly neutral
    let drift = ((gp[0] as i16 - gp[1] as i16).abs()).max((gp[0] as i16 - gp[2] as i16).abs());
    println!("[{}] Gray neutrality drift: {}", label, drift);
    assert!(
        drift <= 20,
        "[{}] Gray not neutral: {:?} (drift={})",
        label,
        gp,
        drift
    );
}

// ---------------------------------------------------------------------------
// Fast mode tests (existing behavior)
// ---------------------------------------------------------------------------

#[test]
fn test_neutral_axis_stability() {
    run_neutral_axis(SimulationMode::Fast, "Fast");
}

#[test]
fn test_channel_integrity() {
    run_channel_integrity(SimulationMode::Fast, "Fast");
}

// ---------------------------------------------------------------------------
// Accurate mode tests
// ---------------------------------------------------------------------------

#[test]
fn test_accurate_neutral_axis() {
    run_neutral_axis(SimulationMode::Accurate, "Accurate");
}

#[test]
fn test_accurate_channel_integrity() {
    run_channel_integrity(SimulationMode::Accurate, "Accurate");
}

#[test]
fn test_accurate_white_gray() {
    run_white_gray_check(SimulationMode::Accurate, "Accurate");
}

#[test]
fn test_fast_white_gray() {
    run_white_gray_check(SimulationMode::Fast, "Fast");
}
