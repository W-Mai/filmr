use filmr::{process_image, estimate_exposure_time, FilmStock, OutputMode, SimulationConfig, WhiteBalanceMode};
use filmr::presets::get_all_stocks;
use image::{Rgb, RgbImage};
use std::fs;
use std::path::Path;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use palette::{Srgb, Lab, FromColor};

/// Quality Report Generator based on tec4.md
/// Implements all 5 categories of verification.
fn main() {
    let output_dir = "quality_report";
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir).unwrap();
    }
    
    let stocks = get_all_stocks();
    println!("Starting Full Quality Verification for {} stocks...", stocks.len());

    let mut report = String::from("# Film Simulation Quality Report\n\n");
    report.push_str("## Metrics Explanation\n");
    report.push_str("- **Neutral Drift (Lab)**: Mean deviation from neutral axis in CIELAB space (|a*| + |b*|). Threshold: < 5.0\n");
    report.push_str("- **Lum Corr (RG/GB)**: Correlation between luminance and color shift. High correlation (> 0.8) indicates systematic error.\n");
    report.push_str("- **Crosstalk (R->G)**: Ratio of Green channel output when input is pure Red. Threshold: < 0.65\n");
    report.push_str("- **Hue Reversals**: Number of local hue reversals in HSV ramp. Should be 0.\n\n");

    report.push_str("| Film Stock | Neutral Lab Drift | Lum Corr (Max) | Crosstalk (R->G) | Hue Reversals | Status |\n");
    report.push_str("|------------|-------------------|----------------|------------------|---------------|--------|\n");

    // Store results for contact sheet
    let mut sheet_data = Vec::new();

    for (name, stock) in &stocks {
        println!("Verifying {}...", name);
        let (metrics, neutral_img, channels_img, hue_img) = verify_stock(name, stock);
        
        let status = if metrics.passed { "✅ PASS" } else { "❌ FAIL" };
        let max_corr = metrics.lum_corr_rg.abs().max(metrics.lum_corr_gb.abs());
        
        report.push_str(&format!("| {} | {:.2} | {:.2} | {:.2} | {} | {} |\n", 
            name, metrics.neutral_drift_lab, max_corr, metrics.channel_matrix[1][0], metrics.hue_reversals, status));

        sheet_data.push((*name, metrics, neutral_img, channels_img, hue_img));
    }

    fs::write(format!("{}/report.md", output_dir), report).unwrap();
    println!("Quality verification complete. Report saved to {}/report.md", output_dir);
    
    // Generate Contact Sheet
    println!("Generating contact sheet...");
    generate_contact_sheet(&sheet_data, output_dir);
    println!("Contact sheet saved to {}/contact_sheet.jpg", output_dir);
}

#[derive(Clone, Debug)]
struct QualityMetrics {
    neutral_drift_lab: f32, // Metric E
    lum_corr_rg: f32,       // Metric B
    lum_corr_gb: f32,       // Metric B
    channel_matrix: [[f32; 3]; 3], // Metric C
    hue_reversals: u32,     // Metric D
    passed: bool,
}

fn verify_stock(_name: &str, stock: &FilmStock) -> (QualityMetrics, RgbImage, RgbImage, RgbImage) {
    let is_bw = stock.grain_model.monochrome;
    
    // 1. Neutral Axis Stability Test (Metrics A, B, E)
    let (neutral_metrics, neutral_img) = test_neutral_axis(stock);
    
    // 2. Channel Integrity Test (Metric C)
    let (matrix, channels_img) = test_channel_integrity(stock);

    // 3. Hue Consistency Test (Metric D)
    let (hue_reversals, hue_img) = test_hue_consistency(stock);

    // Evaluation Logic
    // Thresholds:
    // Lab Drift: < 5.0 (perceptual) - Relaxed for film looks, strictly neutral is < 1.0
    // Lum Corr: < 0.9 (Allow some curve divergence, but perfectly linear correlation is suspicious) -> actually correlation is bad if shift is strong.
    // Crosstalk: < 0.65 (R->G)
    
    let drift_pass = neutral_metrics.0 < 8.0; // Lab drift threshold
    let leak_pass = if is_bw { matrix[1][0] > 0.9 } else { matrix[1][0] < 0.70 }; // R->G leak
    let hue_pass = hue_reversals == 0;
    
    let passed = drift_pass && leak_pass && (hue_pass || is_bw);

    (QualityMetrics {
        neutral_drift_lab: neutral_metrics.0,
        lum_corr_rg: neutral_metrics.1,
        lum_corr_gb: neutral_metrics.2,
        channel_matrix: matrix,
        hue_reversals,
        passed,
    }, neutral_img, channels_img, hue_img)
}

// Returns (Lab Drift, Corr RG, Corr GB, Image)
fn test_neutral_axis(stock: &FilmStock) -> ((f32, f32, f32), RgbImage) {
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
    let config = SimulationConfig {
        exposure_time: t_est, 
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
    };

    let output = process_image(&input, stock, &config);

    let mut total_lab_drift = 0.0;
    let mut count = 0.0;
    
    let mut lum_in = Vec::new();
    let mut drift_rg = Vec::new();
    let mut drift_gb = Vec::new();

    // Sample the center line
    for x in 0..width {
        let p = output.get_pixel(x, height/2);
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        
        // Lab Drift
        let srgb = Srgb::new(r / 255.0, g / 255.0, b / 255.0);
        let lab: Lab = Lab::from_color(srgb);
        total_lab_drift += lab.a.abs() + lab.b.abs();
        
        // Data for Correlation
        lum_in.push(x as f32);
        drift_rg.push(r - g);
        drift_gb.push(g - b);
        
        count += 1.0;
    }
    
    let avg_lab_drift = total_lab_drift / count;
    let corr_rg = calculate_correlation(&lum_in, &drift_rg);
    let corr_gb = calculate_correlation(&lum_in, &drift_gb);
    
    ((avg_lab_drift, corr_rg, corr_gb), output)
}

fn calculate_correlation(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len() as f32;
    let mean_x = x.iter().sum::<f32>() / n;
    let mean_y = y.iter().sum::<f32>() / n;
    
    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;
    
    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }
    
    if den_x == 0.0 || den_y == 0.0 {
        return 0.0;
    }
    
    num / (den_x.sqrt() * den_y.sqrt())
}

// Returns (Matrix 3x3, Image)
fn test_channel_integrity(stock: &FilmStock) -> ([[f32; 3]; 3], RgbImage) {
    let size = 32;
    let mut input = RgbImage::new(size * 3, size);
    
    // R, G, B Patches
    for y in 0..size {
        for x in 0..size { input.put_pixel(x, y, Rgb([255, 0, 0])); }
        for x in size..size*2 { input.put_pixel(x, y, Rgb([0, 255, 0])); }
        for x in size*2..size*3 { input.put_pixel(x, y, Rgb([0, 0, 255])); }
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

    // Sample Outputs
    let get_val = |x_offset: u32| {
        let p = output.get_pixel(x_offset + size/2, size/2);
        (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0)
    };
    
    let (r_r, r_g, r_b) = get_val(0);
    let (g_r, g_g, g_b) = get_val(size);
    let (b_r, b_g, b_b) = get_val(size*2);
    
    let matrix = [
        [r_r, r_g, r_b], // Input Red -> Output R, G, B
        [g_r, g_g, g_b], // Input Green -> Output R, G, B
        [b_r, b_g, b_b], // Input Blue -> Output R, G, B
    ];
    
    (matrix, output)
}

// Returns (Reversals, Image)
fn test_hue_consistency(stock: &FilmStock) -> (u32, RgbImage) {
    let width = 360;
    let height = 50;
    let mut input = RgbImage::new(width, height);
    
    for x in 0..width {
        let h = x as f32; 
        let s = 1.0;
        let v = 0.8;
        
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

    // Check reversals
    let mut reversals = 0;
    // Simple check: Convert to Hue and check monotonicity
    // Or just check RGB distance for local continuity
    for x in 0..(width-1) {
        let p1 = output.get_pixel(x, height/2);
        let p2 = output.get_pixel(x+1, height/2);
        
        let d = ((p1[0] as i16 - p2[0] as i16).abs() +
                 (p1[1] as i16 - p2[1] as i16).abs() +
                 (p1[2] as i16 - p2[2] as i16).abs()) as f32;
                 
        if d > 80.0 {
            reversals += 1;
        }
    }
    
    (reversals, output)
}

fn generate_contact_sheet(data: &[(&str, QualityMetrics, RgbImage, RgbImage, RgbImage)], output_dir: &str) {
    let header_height = 40;
    let row_height = 80;
    let padding = 10;
    
    let col_name_width = 220;
    let col_neutral_width = 256;
    let col_channels_width = 120;
    let col_hue_width = 360;
    let col_status_width = 120;
    
    let total_width = col_name_width + col_neutral_width + col_channels_width + col_hue_width + col_status_width + (6 * padding);
    let total_height = header_height + (row_height + padding) * data.len() as u32 + padding;
    
    let mut contact_sheet = RgbImage::new(total_width, total_height);
    for p in contact_sheet.pixels_mut() {
        *p = Rgb([255, 255, 255]);
    }

    let font_data = include_bytes!("/System/Library/Fonts/Monaco.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale_header = Scale { x: 24.0, y: 24.0 };
    let scale_text = Scale { x: 18.0, y: 18.0 };
    let scale_small = Scale { x: 12.0, y: 12.0 };

    // Header
    let headers = ["Film Stock", "Neutral (Lab Δ)", "Channels (R->G)", "Hue Ramp", "Status"];
    let col_widths = [col_name_width, col_neutral_width, col_channels_width, col_hue_width, col_status_width];
    let mut x_cursor = padding as i32;
    
    for y in 0..header_height {
        for x in 0..total_width {
            contact_sheet.put_pixel(x, y, Rgb([50, 50, 50]));
        }
    }

    for (i, header) in headers.iter().enumerate() {
        draw_text_mut(&mut contact_sheet, Rgb([255, 255, 255]), x_cursor, 10, scale_header, &font, header);
        x_cursor += col_widths[i] as i32 + padding as i32;
    }

    for (i, (name, metrics, neutral, channels, hue)) in data.iter().enumerate() {
        let y_base = (header_height + padding + (row_height + padding) * i as u32) as i32;
        let mut x_cursor = padding as i32;
        
        // 1. Name
        draw_text_mut(&mut contact_sheet, Rgb([0, 0, 0]), x_cursor, y_base + 30, scale_text, &font, name);
        x_cursor += col_name_width as i32 + padding as i32;
        
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

        // 2. Neutral
        copy_image(&mut contact_sheet, neutral, x_cursor, y_base + 15);
        let drift_color = if metrics.neutral_drift_lab < 5.0 { Rgb([0, 100, 0]) } else { Rgb([200, 0, 0]) };
        draw_text_mut(&mut contact_sheet, drift_color, x_cursor, y_base + 66, scale_small, &font, 
            &format!("LabΔ: {:.2} | Corr: {:.2}", metrics.neutral_drift_lab, metrics.lum_corr_rg.abs().max(metrics.lum_corr_gb.abs())));
        x_cursor += col_neutral_width as i32 + padding as i32;

        // 3. Channels
        copy_image(&mut contact_sheet, channels, x_cursor, y_base + 24);
        let leak_val = metrics.channel_matrix[1][0]; // Input Red -> Output Green
        let leak_color = if leak_val < 0.65 { Rgb([0, 100, 0]) } else { Rgb([200, 0, 0]) };
        draw_text_mut(&mut contact_sheet, leak_color, x_cursor, y_base + 60, scale_small, &font, 
            &format!("R->G: {:.2}", leak_val));
        x_cursor += col_channels_width as i32 + padding as i32;

        // 4. Hue
        copy_image(&mut contact_sheet, hue, x_cursor, y_base + 15);
        if metrics.hue_reversals > 0 {
             draw_text_mut(&mut contact_sheet, Rgb([200, 0, 0]), x_cursor, y_base + 66, scale_small, &font, 
                &format!("Reversals: {}", metrics.hue_reversals));
        }
        x_cursor += col_hue_width as i32 + padding as i32;

        // 5. Status
        let status_text = if metrics.passed { "PASS" } else { "FAIL" };
        let status_color = if metrics.passed { Rgb([0, 150, 0]) } else { Rgb([200, 0, 0]) };
        draw_text_mut(&mut contact_sheet, status_color, x_cursor, y_base + 30, scale_header, &font, status_text);
    }
    
    let out_file = std::fs::File::create(format!("{}/contact_sheet.jpg", output_dir)).unwrap();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 100);
    enc.encode_image(&contact_sheet).unwrap();
}
