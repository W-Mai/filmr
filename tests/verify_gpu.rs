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
fn test_gpu_matches_cpu_without_duplicate_blur() {
    // Wide enough that the MTF stage is active (width > 36 * resolution_lp_mm).
    // A high-frequency pattern makes a duplicated blur obvious: if the GPU path
    // applies its own global Gaussian on top of the CPU radial MTF, contrast
    // collapses and the difference against the CPU render is large.
    let width = 3072;
    let height = 32;
    let input = RgbImage::from_fn(width, height, |x, y| {
        let bar = if (x / 4) % 2 == 0 { 240 } else { 16 };
        Rgb([bar, bar, ((y * 7) % 255) as u8])
    });

    let film = STANDARD_DAYLIGHT();
    let base_config = SimulationConfig {
        enable_grain: false,
        motion_blur_amount: 0.0,
        object_motion_amount: 0.0,
        dof_amount: 0.0,
        rotational_blur_amount: 0.0,
        auto_levels: false,
        white_balance_mode: WhiteBalanceMode::Off,
        ..Default::default()
    };

    let cpu = process_image(
        &input,
        &film,
        &SimulationConfig {
            use_gpu: false,
            ..base_config.clone()
        },
    );
    let gpu = process_image(
        &input,
        &film,
        &SimulationConfig {
            use_gpu: true,
            ..base_config
        },
    );

    assert_eq!(cpu.dimensions(), gpu.dimensions());

    let mut total_diff = 0u64;
    let mut large_diff = 0u64;
    for (c, g) in cpu.pixels().zip(gpu.pixels()) {
        for ch in 0..3 {
            let diff = c[ch].abs_diff(g[ch]) as u64;
            total_diff += diff;
            if diff > 8 {
                large_diff += 1;
            }
        }
    }

    let channels = (width * height * 3) as u64;
    let mean_diff = total_diff as f64 / channels as f64;
    let large_fraction = large_diff as f64 / channels as f64;

    println!(
        "CPU/GPU parity: mean abs diff = {:.4}, pixels with diff > 8 = {:.4}%",
        mean_diff,
        large_fraction * 100.0
    );

    assert!(
        mean_diff < 1.5,
        "GPU output deviates from CPU too much (mean diff {:.4}); duplicate GPU blur?",
        mean_diff
    );
    assert!(
        large_fraction < 0.001,
        "Too many pixels differ by more than 8 levels ({:.3}%); duplicate GPU blur?",
        large_fraction * 100.0
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
