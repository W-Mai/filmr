use filmr::presets;
use filmr::processor::{process_image, SimulationConfig};
use image::{Rgb, RgbImage};

#[test]
fn test_gpu_full_pipeline() {
    let width = 64;
    let height = 64;
    let input = RgbImage::from_fn(width, height, |x, y| {
        Rgb([(x % 255) as u8, (y % 255) as u8, 128])
    });

    let film = presets::STANDARD_DAYLIGHT();
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
