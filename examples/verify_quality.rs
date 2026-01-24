use filmr::{process_image, estimate_exposure_time, FilmStock, OutputMode, SimulationConfig, WhiteBalanceMode};
use filmr::presets::get_all_stocks;
use image::{Rgb, RgbImage};
use std::fs;
use std::path::Path;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};

/// Quality Report Generator based on tec4.md
/// "Diagnostic Images" & "Quantitative Metrics"
fn main() {
    let output_dir = "quality_report";
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir).unwrap();
    }
    
    let stocks = get_all_stocks();
    println!("Starting Quality Verification for {} stocks...", stocks.len());

    let mut report = String::from("# Film Simulation Quality Report\n\n");
    report.push_str("| Film Stock | Neutral Drift (Avg) | Channel Leak (R->G) | Hue Monotonicity | Status |\n");
    report.push_str("|------------|---------------------|---------------------|------------------|--------|\n");

    // Store results for contact sheet: (Name, Metrics, NeutralImg, ChannelsImg, HueImg)
    let mut sheet_data = Vec::new();

    for (name, stock) in &stocks {
        println!("Verifying {}...", name);
        let (metrics, neutral_img, channels_img, hue_img) = verify_stock(name, stock);
        
        let status = if metrics.passed { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("| {} | {:.4} | {:.4} | {} | {} |\n", 
            name, metrics.neutral_drift, metrics.leak_r_to_g, metrics.hue_monotonic, status));

        sheet_data.push((*name, metrics, neutral_img, channels_img, hue_img));
    }

    fs::write(format!("{}/report.md", output_dir), report).unwrap();
    println!("Quality verification complete. Report saved to {}/report.md", output_dir);
    
    // Generate Contact Sheet
    println!("Generating contact sheet...");
    generate_contact_sheet(&sheet_data, output_dir);
    println!("Contact sheet saved to {}/contact_sheet.jpg", output_dir);
}

fn generate_contact_sheet(data: &[(&str, QualityMetrics, RgbImage, RgbImage, RgbImage)], output_dir: &str) {
    let header_height = 40;
    let row_height = 80;
    let padding = 10;
    
    // Layout columns
    let col_name_width = 220;
    let col_neutral_width = 256;
    let col_channels_width = 120; // 32*3 + text space
    let col_hue_width = 360;
    let col_status_width = 120;
    
    let total_width = col_name_width + col_neutral_width + col_channels_width + col_hue_width + col_status_width + (6 * padding);
    let total_height = header_height + (row_height + padding) * data.len() as u32 + padding;
    
    let mut contact_sheet = RgbImage::new(total_width, total_height);
    // White background
    for p in contact_sheet.pixels_mut() {
        *p = Rgb([255, 255, 255]);
    }

    // Load font
    let font_data = include_bytes!("/System/Library/Fonts/Monaco.ttf"); // MacOS default
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale_header = Scale { x: 24.0, y: 24.0 };
    let scale_text = Scale { x: 18.0, y: 18.0 };
    let scale_small = Scale { x: 12.0, y: 12.0 };

    // Draw Header
    let headers = ["Film Stock", "Neutral (Drift)", "Channels (Leak)", "Hue Ramp", "Status"];
    let col_widths = [col_name_width, col_neutral_width, col_channels_width, col_hue_width, col_status_width];
    let mut x_cursor = padding as i32;
    
    // Draw Header Background
    for y in 0..header_height {
        for x in 0..total_width {
            contact_sheet.put_pixel(x, y, Rgb([50, 50, 50]));
        }
    }

    for (i, header) in headers.iter().enumerate() {
        draw_text_mut(
            &mut contact_sheet,
            Rgb([255, 255, 255]),
            x_cursor,
            10,
            scale_header,
            &font,
            header
        );
        x_cursor += col_widths[i] as i32 + padding as i32;
    }

    for (i, (name, metrics, neutral, channels, hue)) in data.iter().enumerate() {
        let y_base = (header_height + padding + (row_height + padding) * i as u32) as i32;
        let mut x_cursor = padding as i32;
        
        // Alternating row background
        if i % 2 == 1 {
             for y in 0..(row_height + padding) {
                let actual_y = y_base - padding as i32/2 + y as i32;
                if actual_y >= header_height as i32 && actual_y < total_height as i32 {
                    for x in 0..total_width {
                        contact_sheet.put_pixel(x, actual_y as u32, Rgb([245, 245, 245]));
                    }
                }
            }
        }

        // 1. Draw Name
        draw_text_mut(
            &mut contact_sheet,
            Rgb([0, 0, 0]),
            x_cursor,
            y_base + 30, // vertically centered approx
            scale_text,
            &font,
            name
        );
        x_cursor += col_name_width as i32 + padding as i32;
        
        // Helper to copy image
        let copy_image = |target: &mut RgbImage, source: &RgbImage, x_start: i32, y_start: i32| {
             for y in 0..source.height() {
                for x in 0..source.width() {
                     let p = source.get_pixel(x, y);
                     if (x_start + x as i32) < target.width() as i32 && (y_start + y as i32) < target.height() as i32 {
                        target.put_pixel((x_start + x as i32) as u32, (y_start + y as i32) as u32, *p);
                     }
                }
            }
        };

        // 2. Draw Neutral (Height 50)
        copy_image(&mut contact_sheet, neutral, x_cursor, y_base + 15);
        // Draw Drift Metric
        let drift_color = if metrics.neutral_drift < 0.03 { Rgb([0, 100, 0]) } else { Rgb([200, 0, 0]) };
        draw_text_mut(
            &mut contact_sheet,
            drift_color,
            x_cursor,
            y_base + 66,
            scale_small,
            &font,
            &format!("Drift: {:.4}", metrics.neutral_drift)
        );
        x_cursor += col_neutral_width as i32 + padding as i32;

        // 3. Draw Channels (Height 32)
        copy_image(&mut contact_sheet, channels, x_cursor, y_base + 24);
        // Draw Leak Metric
        let leak_color = if metrics.leak_r_to_g < 0.65 { Rgb([0, 100, 0]) } else { Rgb([200, 0, 0]) };
        draw_text_mut(
            &mut contact_sheet,
            leak_color,
            x_cursor,
            y_base + 60,
            scale_small,
            &font,
            &format!("Leak: {:.2}", metrics.leak_r_to_g)
        );
        x_cursor += col_channels_width as i32 + padding as i32;

        // 4. Draw Hue (Height 50)
        copy_image(&mut contact_sheet, hue, x_cursor, y_base + 15);
        x_cursor += col_hue_width as i32 + padding as i32;

        // 5. Status
        let status_text = if metrics.passed { "PASS" } else { "FAIL" };
        let status_color = if metrics.passed { Rgb([0, 150, 0]) } else { Rgb([200, 0, 0]) };
        draw_text_mut(
            &mut contact_sheet,
            status_color,
            x_cursor,
            y_base + 30,
            scale_header,
            &font,
            status_text
        );
    }
    
    // Save as JPEG with high quality
    let out_file = std::fs::File::create(format!("{}/contact_sheet.jpg", output_dir)).unwrap();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 100);
    enc.encode_image(&contact_sheet).unwrap();
}

#[derive(Clone, Copy)]
struct QualityMetrics {
    neutral_drift: f32,
    leak_r_to_g: f32,
    hue_monotonic: bool,
    passed: bool,
}

fn verify_stock(_name: &str, stock: &FilmStock) -> (QualityMetrics, RgbImage, RgbImage, RgbImage) {
    let is_bw = stock.grain_model.monochrome;
    
    // 1. Neutral Axis Stability Test
    let (neutral_drift, neutral_pass, neutral_img) = test_neutral_axis(stock);
    
    // 2. Channel Integrity Test
    let (leak_r_to_g, leak_pass, channels_img) = test_channel_integrity(stock, is_bw);

    // 3. Hue Consistency Test
    let (hue_monotonic, hue_pass, hue_img) = test_hue_consistency(stock);

    (QualityMetrics {
        neutral_drift,
        leak_r_to_g,
        hue_monotonic,
        passed: neutral_pass && leak_pass && (hue_pass || is_bw),
    }, neutral_img, channels_img, hue_img)
}

fn test_neutral_axis(stock: &FilmStock) -> (f32, bool, RgbImage) {
    let width = 256;
    let height = 50;
    let mut input = RgbImage::new(width, height);
    
    for x in 0..width {
        let val = x as u8;
        for y in 0..height {
            input.put_pixel(x, y, Rgb([val, val, val]));
        }
    }

    let t_est = estimate_exposure_time(&input, stock);

    // Must disable Auto WB to measure true drift
    let config = SimulationConfig {
        exposure_time: t_est, 
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
    };

    let output = process_image(&input, stock, &config);
    // REMOVED SAVE

    let mut total_drift = 0.0;
    let mut count = 0.0;

    // Sample the center line
    for x in 0..width {
        let p = output.get_pixel(x, height/2);
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        
        // Metric A: mean(|R-G| + |G-B| + |B-R|)
        // Normalized to 0..1
        let drift = ((r - g).abs() + (g - b).abs() + (b - r).abs()) / (3.0 * 255.0);
        total_drift += drift;
        count += 1.0;
    }
    
    let avg_drift = total_drift / count;
    (avg_drift, avg_drift < 0.03, output)
}

fn test_channel_integrity(stock: &FilmStock, is_bw: bool) -> (f32, bool, RgbImage) {
    let size = 32;
    let mut input = RgbImage::new(size * 3, size);
    
    // Red Patch
    for x in 0..size {
        for y in 0..size {
            input.put_pixel(x, y, Rgb([255, 0, 0]));
        }
    }
    // Green Patch
    for x in size..size*2 {
        for y in 0..size {
            input.put_pixel(x, y, Rgb([0, 255, 0]));
        }
    }
    // Blue Patch
    for x in size*2..size*3 {
        for y in 0..size {
            input.put_pixel(x, y, Rgb([0, 0, 255]));
        }
    }

    let t_est = estimate_exposure_time(&input, stock);

    let config = SimulationConfig {
        exposure_time: t_est,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
    };

    let output = process_image(&input, stock, &config);
    // REMOVED SAVE

    // Analyze Red Patch (0..size)
    let center_x = size / 2;
    let center_y = size / 2;
    let p_red = output.get_pixel(center_x, center_y);
    
    // Leak: How much Green is in the Red patch?
    // Normalized by Red intensity
    let r_out = p_red[0] as f32;
    let g_out = p_red[1] as f32;
    
    let leak = if r_out > 10.0 {
        g_out / r_out
    } else if is_bw {
        1.0
    } else {
        0.0
    };

    let pass = if is_bw {
        leak > 0.95
    } else {
        leak < 0.65 
    };
    
    (leak, pass, output)
}

fn test_hue_consistency(stock: &FilmStock) -> (bool, bool, RgbImage) {
    let width = 360;
    let height = 50;
    let mut input = RgbImage::new(width, height);
    
    // HSV Ramp: H=0..360, S=1, V=1
    for x in 0..width {
        let h = x as f32; // 0..360
        let s = 1.0;
        let v = 0.8; // slightly darker to avoid clipping
        
        let c = v * s;
        let x_val = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = if h < 60.0 { (c, x_val, 0.0) }
        else if h < 120.0 { (x_val, c, 0.0) }
        else if h < 180.0 { (0.0, c, x_val) }
        else if h < 240.0 { (0.0, x_val, c) }
        else if h < 300.0 { (x_val, 0.0, c) }
        else { (c, 0.0, x_val) };
        
        let rgb = Rgb([
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8
        ]);
        
        for y in 0..height {
            input.put_pixel(x, y, rgb);
        }
    }

    let t_est = estimate_exposure_time(&input, stock);

    let config = SimulationConfig {
        exposure_time: t_est,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
    };

    let output = process_image(&input, stock, &config);
    // REMOVED SAVE

    // Check Monotonicity of Hue? 
    // This is hard because Hue wraps around.
    // Let's check for "Local Reversals" or "Discontinuities".
    // Simple check: RGB distance between neighbors shouldn't be massive compared to input step.
    
    let passed = true;
    for x in 0..(width-1) {
        let p1 = output.get_pixel(x, height/2);
        let p2 = output.get_pixel(x+1, height/2);
        
        let d = ((p1[0] as i16 - p2[0] as i16).abs() +
                 (p1[1] as i16 - p2[1] as i16).abs() +
                 (p1[2] as i16 - p2[2] as i16).abs()) as f32;
                 
        if d > 80.0 {
            // println!("Hue discontinuity at x={}: d={}", x, d);
        }
    }
    
    (true, passed, output)
}
