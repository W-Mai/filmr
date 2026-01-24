use filmr::{process_image, FilmStock, OutputMode, SimulationConfig, WhiteBalanceMode};
use filmr::presets::get_all_stocks;
use image::{RgbImage};

/// Calibration Target: 18% Mid-Gray input should result in ~50% Luminance output.
/// In sRGB, 50% Luminance is approx 128/255.
/// However, 18% Linear is 0.18.
/// 0.18 ^ (1/2.2) = 0.458. 0.458 * 255 = 117.
/// Many Zone Systems place Zone V at 127/128 for print/screen match.
/// Let's target 124 (a compromise).
const TARGET_LUMINANCE: f32 = 124.0;
const TOLERANCE: f32 = 2.0;
const MAX_ITERATIONS: usize = 100;

fn main() {
    println!("=== Film Stock Auto-Calibration Tool ===");
    println!("Target Mid-Tone Luminance: {:.1}", TARGET_LUMINANCE);
    
    let stocks = get_all_stocks();
    let mut results = Vec::new();

    for (name, mut stock) in stocks {
        let (optimal_offset, final_lum) = calibrate_film(&mut stock, name);
        
        results.push((name, stock.iso, optimal_offset, final_lum));
        
        if (final_lum - TARGET_LUMINANCE).abs() > TOLERANCE {
            println!("Calibrating {} (ISO {:.0}) -> WARNING: Final Lum {:.1} (Offset {:.5})", name, stock.iso, final_lum, optimal_offset);
        } else {
            println!("Calibrating {} (ISO {:.0}) -> Success! Offset {:.5}", name, stock.iso, optimal_offset);
        }
    }

    println!("\n=== Calibration Report ===");
    println!("Please manually update src/presets.rs with these values:\n");
    
    for (name, _iso, offset, _lum) in results {
        println!("Film: {}", name);
        println!("exposure_offset: {:.5},", offset);
        println!("--------------------------------");
    }
}

fn calibrate_film(stock: &mut FilmStock, _name: &str) -> (f32, f32) {
    // Create a 16x16 standard 18% gray patch
    // In Linear RGB: 0.18
    // In sRGB input to process_image: sRGB(0.18)
    // sRGB(0.18) = 1.055 * 0.18^(1/2.4) - 0.055 = 0.46 (approx)
    // Actually process_image takes sRGB and linearizes it.
    // So we should feed it an image with sRGB pixel value corresponding to linear 0.18.
    // 0.18 linear ~ 117/255 sRGB.
    let gray_pixel_val = (linear_to_srgb(0.18) * 255.0).round() as u8;
    
    let width = 16;
    let height = 16;
    let mut input = RgbImage::new(width, height);
    for p in input.pixels_mut() {
        p.0 = [gray_pixel_val, gray_pixel_val, gray_pixel_val];
    }

    // Binary Search / Bisection for Exposure Offset
    // exposure_offset controls the "speed point" of the curve.
    // Higher offset = Shift curve to right = Needs more light = Darker output for same exposure.
    // Lower offset = Shift curve to left = Needs less light = Brighter output.
    
    // Range: 0.0001 (Very bright) to 10000.0 (Very dark)
    let mut low = 0.0001;
    let mut high = 10000.0;
    let mut current_offset = stock.r_curve.exposure_offset;
    
    // Initial config
    let config = SimulationConfig {
        exposure_time: 1.0, // Standard 1s exposure
        enable_grain: false,
        output_mode: OutputMode::Positive, // Calibrate for positive
        white_balance_mode: WhiteBalanceMode::Auto,
        white_balance_strength: 1.0,
    };

    // Override the stock's offset for testing
    // We assume R, G, B offsets are linked for ISO calibration
    
    let mut final_lum = 0.0;

    for _i in 0..MAX_ITERATIONS {
        stock.r_curve.exposure_offset = current_offset;
        stock.g_curve.exposure_offset = current_offset;
        stock.b_curve.exposure_offset = current_offset;
        
        let output = process_image(&input, stock, &config);
        let lum = calculate_mean_luminance(&output);
        final_lum = lum;

        // println!("  Iter {}: Offset {:.5} -> Lum {:.1}", i, current_offset, lum);

        if (lum - TARGET_LUMINANCE).abs() < TOLERANCE {
            break;
        }

        if lum > TARGET_LUMINANCE {
            // Image is too bright -> Need Darker
            // Darker -> Higher Offset (Move curve right -> Input falls on Toe -> Low Density -> High Trans -> Black Output)
            // Wait, Low Density -> High Trans -> Black Output (Positive).
            // Yes. High Offset -> Darker.
            low = current_offset;
        } else {
            // Too Dark -> Need Brighter -> Lower Offset
            high = current_offset;
        }
        
        current_offset = (low + high) / 2.0;
    }

    (current_offset, final_lum)
}

fn calculate_mean_luminance(img: &RgbImage) -> f32 {
    let mut sum = 0.0;
    for p in img.pixels() {
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
    sum / (img.width() * img.height()) as f32
}

// sRGB transfer function (Gamma 2.4 approx)
fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}
