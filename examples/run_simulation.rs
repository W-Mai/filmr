use filmr::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode,
};
use image::ImageReader;
use std::path::Path;
use std::time::Instant;

// Import presets directly from source (if public) or copy them
// Since presets are in src/presets.rs and public, we can use them if lib.rs exposes them.
// Let's check lib.rs again. It exposes `presets` mod.
use filmr::presets::{fujifilm, ilford, kodak};

fn main() {
    let input_path = "DSC_2497.JPG";
    if !Path::new(input_path).exists() {
        eprintln!("Error: {} not found.", input_path);
        return;
    }

    println!("Loading image...");
    let img = ImageReader::open(input_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();
    println!("Image loaded: {}x{}", img.width(), img.height());

    let stocks = [
        (
            "Kodak_Tri-X_400",
            kodak::KODAK_TRI_X_400(),
            OutputMode::Positive,
        ),
        (
            "Fujifilm_Velvia_50",
            fujifilm::VELVIA_50(),
            OutputMode::Positive,
        ),
        (
            "Ilford_HP5_Plus",
            ilford::HP5_PLUS_400(),
            OutputMode::Positive,
        ),
        (
            "Kodak_Portra_400",
            kodak::KODAK_PORTRA_400(),
            OutputMode::Positive,
        ),
    ];

    for (name, stock, mode) in stocks.iter() {
        println!("\nProcessing {}...", name);
        let start = Instant::now();

        // Estimate exposure
        let t_est = estimate_exposure_time(&img, stock);
        println!("Estimated Exposure Time: {:.4}s", t_est);

        let config = SimulationConfig {
            exposure_time: t_est,
            enable_grain: true,
            output_mode: *mode,
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 0.8,
            ..Default::default()
        };

        let result = process_image(&img, stock, &config);
        let duration = start.elapsed();
        println!("Processed in {:.2?}", duration);

        let output_path = format!("output_{}.jpg", name);
        result.save(&output_path).unwrap();
        println!("Saved to {}", output_path);
    }
}
