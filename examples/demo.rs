use filmr::{presets, process_image, OutputMode, SimulationConfig, WhiteBalanceMode};
use image::{Rgb, RgbImage};

fn main() {
    // 1. Create a test image: Horizontal Gradient Black to White
    let width = 1080;
    let height = 720;
    let mut img = RgbImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let val = (x as f32 / width as f32 * 255.0) as u8;
            img.put_pixel(x, y, Rgb([val, val, val]));
        }
    }

    println!("Generated input gradient image.");

    // 2. Setup Film Simulation
    // Use preset
    let film = presets::STANDARD_DAYLIGHT();
    let config = SimulationConfig {
        exposure_time: 1.0,
        enable_grain: true,
        output_mode: OutputMode::Positive, // Generate a positive image
        white_balance_mode: WhiteBalanceMode::Auto,
        white_balance_strength: 1.0,
        ..Default::default()
    };

    println!("Starting simulation (Positive Mode)...");
    let output = process_image(&img, &film, &config);
    println!("Simulation finished.");

    // 3. Save output
    output.save("output_film.png").unwrap();
    println!("Saved output_film.png");

    // Check some pixel values
    let p_black = output.get_pixel(0, 128);
    let p_mid = output.get_pixel(256, 128);
    let p_white = output.get_pixel(511, 128);

    // For Positive: Black Input -> Dark Output. White Input -> Bright Output.
    println!("Black Input -> Output: {:?}", p_black);
    println!("Mid Input   -> Output: {:?}", p_mid);
    println!("White Input -> Output: {:?}", p_white);
}
