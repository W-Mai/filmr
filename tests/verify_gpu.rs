use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{process_image, SimulationConfig, WhiteBalanceMode};
use image::{Rgb, RgbImage};

#[test]
fn test_gpu_full_pipeline() {
    let width = 64;
    let height = 64;
    let input = RgbImage::from_fn(width, height, |x, y| {
        Rgb([(x % 255) as u8, (y % 255) as u8, 128])
    });

    let film = STANDARD_DAYLIGHT();
    let config = SimulationConfig {
        use_gpu: true,
        light_leak: filmr::light_leak::LightLeakConfig {
            enabled: true,
            leaks: vec![filmr::light_leak::LightLeak::default()],
        },
        ..Default::default()
    };

    println!("Running GPU pipeline test...");
    // This will print to stdout if run with --nocapture
    // It should trigger GPU paths if available.
    // We cannot easily assert GPU WAS used without internal telemetry,
    // but we can check it doesn't crash and produces valid image.
    let output = process_image(&input, &film, &config);

    let center = output.get_pixel(32, 32);
    println!("Center pixel: {:?}", center);
    assert!(
        center[0] > 0 || center[1] > 0 || center[2] > 0,
        "Output should not be black"
    );
}

#[test]
fn test_gpu_halation_effect() {
    let width = 64;
    let height = 64;
    // Create an image with a single very bright pixel in the center
    let input = RgbImage::from_fn(width, height, |x, y| {
        if x == 32 && y == 32 {
            Rgb([255, 255, 255])
        } else {
            Rgb([0, 0, 0])
        }
    });

    let mut film = STANDARD_DAYLIGHT();
    film.halation_strength = 1.0; // Strong halation
    film.halation_threshold = 0.0; // Trigger on EVERYTHING
    film.halation_sigma = 0.1; // Large radius
    film.halation_tint = [1.0, 0.0, 0.0]; // Pure Red halation

    let config = SimulationConfig {
        use_gpu: true,
        enable_grain: false,
        white_balance_mode: WhiteBalanceMode::Off,
        exposure_time: 100.0,
        ..Default::default()
    };

    println!("Running GPU Halation test...");
    let output = process_image(&input, &film, &config);

    // Check center pixel (should be bright)
    let center = output.get_pixel(32, 32);
    println!("Center pixel: {:?}", center);

    // Check neighbor pixel (should be red due to halation)
    let neighbor = output.get_pixel(32 + 5, 32);
    println!("Neighbor pixel (5px away): {:?}", neighbor);

    assert!(neighbor[0] > 0, "Halation should add red glow");
    assert!(
        neighbor[0] > neighbor[1],
        "Halation tint is red, so R should be > G"
    );
    assert!(
        neighbor[0] > neighbor[2],
        "Halation tint is red, so R should be > B"
    );
}
