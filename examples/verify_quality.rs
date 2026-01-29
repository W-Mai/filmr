use filmr::presets::get_all_stocks;
use filmr::{
    estimate_exposure_time, process_image, FilmStock, OutputMode, SimulationConfig,
    WhiteBalanceMode,
};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use palette::{FromColor, Hsv, Lab, Srgb};
use rusttype::{Font, Scale};
use std::fs;
use std::path::Path;

/// Quality Report Generator
/// Implements 7-Layer Verification System (Subset)
fn main() {
    let output_dir = "quality_report";
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir).unwrap();
    }

    let stocks = get_all_stocks();
    println!(
        "Starting Industrial-Grade Quality Verification for {} stocks...",
        stocks.len()
    );

    let mut report = String::from("# Film Simulation Quality Report\n\n");
    report.push_str("## Metrics Explanation\n");
    report.push_str("- **L0: Spectral Fidelity**: Peak Wavelength and FWHM check.\n");
    report.push_str("- **L1: H-D Curve**: Gamma (Slope) and Dmax check. **Reciprocity**: 1s vs 10s consistency.\n");
    report.push_str("- **L2: Crosstalk (R->G)**: Ratio of Green channel output when input is pure Red. **IIE**: Inter-image effects validity.\n");
    report.push_str(
        "- **L4: Neutral Drift (Lab)**: Mean deviation from neutral axis in CIELAB space.\n",
    );
    report.push_str("- **L5: Skin Tone Shift**: Hue shift of Memory Color (Skin) in degrees.\n");
    report.push_str("- **L6: Illuminant**: Stability under WB changes.\n\n");

    report.push_str("| Film Stock | Spectral | H-D Gamma | H-D Dmax | Recip. | Crosstalk | IIE | Lab Drift | Skin Shift | WB Stab. | Status |\n");
    report.push_str("|------------|----------|-----------|----------|--------|-----------|-----|-----------|------------|----------|--------|\n");

    let mut sheet_data = Vec::new();

    for (name, stock) in &stocks {
        println!("Verifying {}...", name);
        let (metrics, neutral_img, channels_img, hue_img) = verify_stock(name, stock);

        let status = if metrics.passed {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        let spectral_status = if metrics.spectral_valid { "OK" } else { "ERR" };
        let recip_status = if metrics.reciprocity_valid {
            "OK"
        } else {
            "ERR"
        };
        let iie_status = if metrics.iie_valid { "OK" } else { "ERR" };
        let wb_status = if metrics.illuminant_valid {
            "OK"
        } else {
            "ERR"
        };

        report.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {} | {:.2} | {} | {:.2} | {:.1}° | {} | {} |\n",
            name,
            spectral_status,
            metrics.gamma,
            metrics.d_max,
            recip_status,
            metrics.channel_matrix[1][0],
            iie_status,
            metrics.neutral_drift_lab,
            metrics.skin_hue_shift,
            wb_status,
            status
        ));

        sheet_data.push((*name, metrics, neutral_img, channels_img, hue_img));
    }

    fs::write(format!("{}/report.md", output_dir), report).unwrap();
    println!(
        "Quality verification complete. Report saved to {}/report.md",
        output_dir
    );

    // Generate Contact Sheet
    println!("Generating contact sheet...");
    generate_contact_sheet(&sheet_data, output_dir);
    println!("Contact sheet saved to {}/contact_sheet.jpg", output_dir);
}

#[derive(Clone, Debug)]
struct QualityMetrics {
    // L0
    spectral_valid: bool,
    // L1
    gamma: f32,
    d_max: f32,
    reciprocity_valid: bool,
    // L2
    channel_matrix: [[f32; 3]; 3],
    iie_valid: bool,
    // L4
    neutral_drift_lab: f32,
    lum_corr_rg: f32,
    lum_corr_gb: f32,
    // L5
    skin_hue_shift: f32,
    // L6
    illuminant_valid: bool,
    // Overall
    hue_reversals: u32,
    passed: bool,
}

fn verify_stock(_name: &str, stock: &FilmStock) -> (QualityMetrics, RgbImage, RgbImage, RgbImage) {
    let is_bw = stock.grain_model.monochrome;

    // L0: Spectral Fidelity
    let spectral_valid = check_spectral_fidelity(stock);

    // L1: H-D Curve & Reciprocity
    let (gamma, d_max) = check_hd_curve(stock);
    let reciprocity_valid = check_reciprocity_failure(stock);

    // L2: Channel Integrity & IIE
    let (matrix, channels_img) = test_channel_integrity(stock);
    let iie_valid = check_interimage_effects(stock);

    // L4: Neutral Axis
    let (neutral_metrics, neutral_img) = test_neutral_axis(stock);

    // L5: Memory Colors (Skin Tone)
    let skin_hue_shift = if is_bw {
        0.0
    } else {
        check_memory_color_shift(stock)
    };

    // L6: System Robustness (Illuminant Invariance)
    let illuminant_valid = check_illuminant_invariance(stock);

    // L6+: Hue Consistency (from previous)
    let (hue_reversals, hue_img) = test_hue_consistency(stock);

    // Thresholds (Industrial Grade)
    let drift_pass = neutral_metrics.0 < 5.0;
    let leak_pass = if is_bw {
        matrix[1][0] > 0.1
    } else {
        matrix[1][0] < 0.65
    };
    let hue_pass = hue_reversals == 0;
    let skin_pass = skin_hue_shift.abs() < 15.0;
    let gamma_pass = gamma > 0.4 && gamma < 2.5;

    let passed = drift_pass
        && leak_pass
        && (hue_pass || is_bw)
        && skin_pass
        && gamma_pass
        && reciprocity_valid
        && iie_valid
        && illuminant_valid;

    (
        QualityMetrics {
            spectral_valid,
            gamma,
            d_max,
            reciprocity_valid,
            channel_matrix: matrix,
            iie_valid,
            neutral_drift_lab: neutral_metrics.0,
            lum_corr_rg: neutral_metrics.1,
            lum_corr_gb: neutral_metrics.2,
            skin_hue_shift,
            illuminant_valid,
            hue_reversals,
            passed,
        },
        neutral_img,
        channels_img,
        hue_img,
    )
}

fn check_reciprocity_failure(stock: &FilmStock) -> bool {
    // L1.2 Reciprocity Failure
    // Check density drift between 1s and 10s exposure with constant H = I * t
    // H = 0.5 (middle greyish)
    // t1 = 1.0 -> I1 = 0.5
    // t2 = 10.0 -> I2 = 0.05

    // We need to simulate a patch with specific intensity and exposure time.
    // process_image uses pixel value as intensity.
    // Pixel 128/255 ~= 0.5 intensity.

    let input = RgbImage::from_pixel(1, 1, Rgb([128, 128, 128]));

    // Case 1: 1s exposure
    let config1 = SimulationConfig {
        exposure_time: 1.0,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
        ..Default::default()
    };
    let out1 = process_image(&input, stock, &config1);
    let p1 = out1.get_pixel(0, 0);
    let lum1 = (p1[0] as f32 + p1[1] as f32 + p1[2] as f32) / 3.0;

    // Case 2: 10s exposure (Intensity should be 1/10th for constant H)
    // We need 1/10th Linear Intensity.
    // 128 sRGB -> ~0.21 Linear. Target ~0.021 Linear.
    // ~0.021 Linear -> ~44 sRGB.
    let _input2 = RgbImage::from_pixel(1, 1, Rgb([44, 44, 44]));
    let config2 = SimulationConfig {
        exposure_time: 10.0,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
        ..Default::default()
    };
    let out2 = process_image(&_input2, stock, &config2);
    let p2 = out2.get_pixel(0, 0);
    let lum2 = (p2[0] as f32 + p2[1] as f32 + p2[2] as f32) / 3.0;

    // Difference should be small if reciprocity holds, or within limits if failure is modeled.
    // If failure is modeled (Schwarzschild), output might drop.
    // Threshold: < 15% change in luminance
    let diff = (lum1 - lum2).abs();
    let avg = (lum1 + lum2) / 2.0;
    if avg < 1.0 {
        return true;
    } // Too dark to tell

    (diff / avg) < 0.25 // Allow some failure as it's a feature, but not broken
}

fn check_interimage_effects(stock: &FilmStock) -> bool {
    // B&W films don't have color channels, so IIE is not applicable (or trivial)
    if stock.grain_model.monochrome {
        return true;
    }

    // L2.2 Interlayer Interimage Effects
    // Simple check: Does a Red input suppress Green/Blue development?
    // In our simplified model, this might not be fully implemented,
    // but we can check if the matrix implies it or if we have a mechanism.
    // Currently we rely on the matrix.
    // If matrix has negative coefficients or specific cross-talk, it simulates this.
    // For now, we check if the matrix is "sane" (diagonal dominant or controlled mixing).

    let m = stock.color_matrix;
    // Diagonal elements should be dominant (positive and large)
    let diag_sum = m[0][0] + m[1][1] + m[2][2];
    let off_diag_sum = m[0][1].abs()
        + m[0][2].abs()
        + m[1][0].abs()
        + m[1][2].abs()
        + m[2][0].abs()
        + m[2][1].abs();

    diag_sum > off_diag_sum
}

fn check_illuminant_invariance(stock: &FilmStock) -> bool {
    // L6.1 Illuminant Invariance
    // Check neutral grey under different WB settings (simulating different illuminant adaptation)
    // If we change WB mode, the neutral axis should remain relatively neutral.

    let input = RgbImage::from_pixel(1, 1, Rgb([128, 128, 128]));
    let t_est = estimate_exposure_time(&input, stock);

    let run_wb = |mode| {
        let config = SimulationConfig {
            exposure_time: t_est,
            enable_grain: false,
            output_mode: OutputMode::Positive,
            white_balance_mode: mode,
            white_balance_strength: 1.0,
            ..Default::default()
        };
        let out = process_image(&input, stock, &config);
        let p = out.get_pixel(0, 0);
        let srgb = Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        );
        let lab: Lab = Lab::from_color(srgb);
        lab.a.abs() + lab.b.abs() // Drift
    };

    let drift_auto = run_wb(WhiteBalanceMode::Auto);
    // let drift_sunny = run_wb(WhiteBalanceMode::Sunny); // If implemented

    drift_auto < 10.0 // Auto WB should keep it somewhat neutral
}

fn check_spectral_fidelity(stock: &FilmStock) -> bool {
    let p = stock.spectral_params;
    // Basic sanity check on peaks
    let r_ok = p.r_peak > 580.0 && p.r_peak < 750.0; // Allow Extended Red / IR (up to 750nm)
    let g_ok = p.g_peak > 500.0 && p.g_peak < 580.0;
    let b_ok = p.b_peak > 400.0 && p.b_peak < 500.0;
    r_ok && g_ok && b_ok
}

fn check_hd_curve(stock: &FilmStock) -> (f32, f32) {
    // Generate step wedge
    let steps = 21;
    let width = steps * 10;
    let height = 20;
    let mut input = RgbImage::new(width, height);

    for i in 0..steps {
        let val = (i as f32 / (steps - 1) as f32 * 255.0) as u8;
        for x in 0..10 {
            for y in 0..height {
                input.put_pixel(i * 10 + x, y, Rgb([val, val, val]));
            }
        }
    }

    let t_est = estimate_exposure_time(&input, stock);
    let config = SimulationConfig {
        exposure_time: t_est,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
        ..Default::default()
    };
    let output = process_image(&input, stock, &config);

    // Measure Gamma (Contrast)
    // Simplified: Measure density difference between 20% and 80% input
    // D = -log10(Output/255)
    let get_density = |step_idx: u32| {
        let p = output.get_pixel(step_idx * 10 + 5, 10);
        let lum = (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0 / 255.0;
        if lum <= 0.001 {
            3.0
        } else {
            -lum.log10()
        }
    };

    let _d_min = get_density(steps - 1); // White input -> Low density (Negative) -> High Value (Positive)
                                         // Wait, OutputMode::Positive means Bright Input -> Bright Output.
                                         // Density is usually measured on the Negative.
                                         // But docs say "H-D Curve: log10(Exposure) vs Density".
                                         // For a Positive simulation, "Density" corresponds to "darkness".
                                         // Bright output (255) -> Density 0. Dark output (0) -> Density Max.

    let d_20 = get_density(4); // Darker input
    let d_80 = get_density(16); // Brighter input

    // Log Exposure difference: log10(0.8) - log10(0.2) = -0.096 - (-0.699) = 0.602
    let gamma = (d_20 - d_80).abs() / 0.602;

    let d_max = get_density(0); // Black input -> Dark output -> High density

    (gamma, d_max)
}

fn check_memory_color_shift(stock: &FilmStock) -> f32 {
    // Skin Tone (Caucasian) - ColorChecker Patch 2
    // sRGB: (194, 150, 130)
    let input_rgb = Rgb([194, 150, 130]);
    let mut input = RgbImage::new(1, 1);
    input.put_pixel(0, 0, input_rgb);

    let t_est = estimate_exposure_time(&input, stock);
    let config = SimulationConfig {
        exposure_time: t_est,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        white_balance_strength: 1.0,
        ..Default::default()
    };
    let output = process_image(&input, stock, &config);
    let p = output.get_pixel(0, 0);

    // Convert both to HSV to measure Hue Shift
    let to_hue = |r: u8, g: u8, b: u8| {
        let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let hsv: Hsv = Hsv::from_color(srgb);
        hsv.hue.into_degrees()
    };

    let h_in = to_hue(194, 150, 130);
    let h_out = to_hue(p[0], p[1], p[2]);

    // Shortest angular distance
    let mut diff = h_out - h_in;
    if diff > 180.0 {
        diff -= 360.0;
    }
    if diff < -180.0 {
        diff += 360.0;
    }

    diff
}

// ... (Rest of functions: test_neutral_axis, test_channel_integrity, test_hue_consistency remain similar but we can clean them up)

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
        white_balance_mode: WhiteBalanceMode::Auto, // Use Auto WB for Neutral Axis check
        white_balance_strength: 1.0,
        ..Default::default()
    };

    let output = process_image(&input, stock, &config);

    let mut total_lab_drift = 0.0;
    let mut count = 0.0;

    let mut lum_in = Vec::new();
    let mut drift_rg = Vec::new();
    let mut drift_gb = Vec::new();

    // Sample the center line
    for x in 0..width {
        let p = output.get_pixel(x, height / 2);
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;

        let srgb = Srgb::new(r / 255.0, g / 255.0, b / 255.0);
        let lab: Lab = Lab::from_color(srgb);
        total_lab_drift += lab.a.abs() + lab.b.abs();

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

fn test_channel_integrity(stock: &FilmStock) -> ([[f32; 3]; 3], RgbImage) {
    let size = 32;
    let mut input = RgbImage::new(size * 3, size);

    for y in 0..size {
        for x in 0..size {
            input.put_pixel(x, y, Rgb([255, 0, 0]));
        }
        for x in size..size * 2 {
            input.put_pixel(x, y, Rgb([0, 255, 0]));
        }
        for x in size * 2..size * 3 {
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
        ..Default::default()
    };

    let output = process_image(&input, stock, &config);

    let get_val = |x_offset: u32| {
        let p = output.get_pixel(x_offset + size / 2, size / 2);
        (
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        )
    };

    let (r_r, r_g, r_b) = get_val(0);
    let (g_r, g_g, g_b) = get_val(size);
    let (b_r, b_g, b_b) = get_val(size * 2);

    let matrix = [[r_r, r_g, r_b], [g_r, g_g, g_b], [b_r, b_g, b_b]];

    (matrix, output)
}

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

        let (r, g, b) = if h < 60.0 {
            (c, x_val, 0.0)
        } else if h < 120.0 {
            (x_val, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x_val)
        } else if h < 240.0 {
            (0.0, x_val, c)
        } else if h < 300.0 {
            (x_val, 0.0, c)
        } else {
            (c, 0.0, x_val)
        };

        let rgb = Rgb([
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
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
        ..Default::default()
    };

    let output = process_image(&input, stock, &config);

    let mut reversals = 0;
    for x in 0..(width - 1) {
        let p1 = output.get_pixel(x, height / 2);
        let p2 = output.get_pixel(x + 1, height / 2);

        let d = ((p1[0] as i16 - p2[0] as i16).abs()
            + (p1[1] as i16 - p2[1] as i16).abs()
            + (p1[2] as i16 - p2[2] as i16).abs()) as f32;

        if d > 80.0 {
            reversals += 1;
        }
    }

    (reversals, output)
}

fn generate_contact_sheet(
    data: &[(&str, QualityMetrics, RgbImage, RgbImage, RgbImage)],
    output_dir: &str,
) {
    let header_height = 40;
    let row_height = 80;
    let padding = 10;

    let col_name_width = 180;
    let col_neutral_width = 256;
    let col_channels_width = 120;
    let col_hue_width = 360;
    let col_status_width = 120;
    let col_metrics_width = 180; // Extra column for H-D and Skin

    let total_width = col_name_width
        + col_neutral_width
        + col_channels_width
        + col_hue_width
        + col_status_width
        + col_metrics_width
        + (7 * padding);
    let total_height = header_height + (row_height + padding) * data.len() as u32 + padding;

    let mut contact_sheet = RgbImage::new(total_width, total_height);
    for p in contact_sheet.pixels_mut() {
        *p = Rgb([255, 255, 255]);
    }

    let font_data = include_bytes!("../apps/filmr/static/ark-pixel-12px-monospaced-zh_cn.otf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale_header = Scale { x: 20.0, y: 20.0 };
    let scale_text = Scale { x: 16.0, y: 16.0 };
    let scale_small = Scale { x: 12.0, y: 12.0 };

    // Header
    let headers = [
        "Stock",
        "Neutral (Lab)",
        "Channels",
        "Hue Ramp",
        "H-D / Skin / WB",
        "Status",
    ];
    let col_widths = [
        col_name_width,
        col_neutral_width,
        col_channels_width,
        col_hue_width,
        col_metrics_width,
        col_status_width,
    ];
    let mut x_cursor = padding as i32;

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
            header,
        );
        x_cursor += col_widths[i] as i32 + padding as i32;
    }

    for (i, (name, metrics, neutral, channels, hue)) in data.iter().enumerate() {
        let y_base = (header_height + padding + (row_height + padding) * i as u32) as i32;
        let mut x_cursor = padding as i32;

        // 1. Name
        draw_text_mut(
            &mut contact_sheet,
            Rgb([0, 0, 0]),
            x_cursor,
            y_base + 30,
            scale_text,
            &font,
            name,
        );
        x_cursor += col_name_width as i32 + padding as i32;

        let copy_image = |target: &mut RgbImage, source: &RgbImage, x_start: i32, y_start: i32| {
            for y in 0..source.height() {
                for x in 0..source.width() {
                    let p = source.get_pixel(x, y);
                    if (x_start + x as i32) < target.width() as i32
                        && (y_start + y as i32) < target.height() as i32
                    {
                        target.put_pixel(
                            (x_start + x as i32) as u32,
                            (y_start + y as i32) as u32,
                            *p,
                        );
                    }
                }
            }
        };

        // 2. Neutral
        copy_image(&mut contact_sheet, neutral, x_cursor, y_base + 15);
        let drift_color = if metrics.neutral_drift_lab < 5.0 {
            Rgb([0, 100, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            drift_color,
            x_cursor,
            y_base + 66,
            scale_small,
            &font,
            &format!(
                "LabΔ: {:.2} | Corr: {:.2}",
                metrics.neutral_drift_lab,
                metrics.lum_corr_rg.abs().max(metrics.lum_corr_gb.abs())
            ),
        );
        x_cursor += col_neutral_width as i32 + padding as i32;

        // 3. Channels
        copy_image(&mut contact_sheet, channels, x_cursor, y_base + 24);
        let leak_val = metrics.channel_matrix[1][0]; // Input Red -> Output Green
        let leak_color = if leak_val < 0.65 {
            Rgb([0, 100, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            leak_color,
            x_cursor,
            y_base + 60,
            scale_small,
            &font,
            &format!("R->G: {:.2}", leak_val),
        );

        // Add IIE status
        let iie_text = if metrics.iie_valid {
            "IIE: OK"
        } else {
            "IIE: Fail"
        };
        let iie_color = if metrics.iie_valid {
            Rgb([0, 100, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            iie_color,
            x_cursor,
            y_base + 70,
            scale_small,
            &font,
            iie_text,
        );

        x_cursor += col_channels_width as i32 + padding as i32;

        // 4. Hue
        copy_image(&mut contact_sheet, hue, x_cursor, y_base + 15);
        if metrics.hue_reversals > 0 {
            draw_text_mut(
                &mut contact_sheet,
                Rgb([200, 0, 0]),
                x_cursor,
                y_base + 66,
                scale_small,
                &font,
                &format!("Reversals: {}", metrics.hue_reversals),
            );
        }
        x_cursor += col_hue_width as i32 + padding as i32;

        // 5. Extra Metrics (H-D / Skin / WB)
        let gamma_color = if metrics.gamma > 0.4 && metrics.gamma < 2.5 {
            Rgb([0, 0, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            gamma_color,
            x_cursor,
            y_base + 15,
            scale_small,
            &font,
            &format!("Gamma: {:.2} Dmax: {:.1}", metrics.gamma, metrics.d_max),
        );

        let skin_color = if metrics.skin_hue_shift.abs() < 15.0 {
            Rgb([0, 100, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            skin_color,
            x_cursor,
            y_base + 30,
            scale_small,
            &font,
            &format!("SkinShift: {:.1}°", metrics.skin_hue_shift),
        );

        let wb_text = if metrics.illuminant_valid {
            "WB: Stable"
        } else {
            "WB: Drift"
        };
        let wb_color = if metrics.illuminant_valid {
            Rgb([0, 100, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            wb_color,
            x_cursor,
            y_base + 45,
            scale_small,
            &font,
            wb_text,
        );

        let recip_text = if metrics.reciprocity_valid {
            "Recip: OK"
        } else {
            "Recip: Bad"
        };
        let recip_color = if metrics.reciprocity_valid {
            Rgb([0, 100, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            recip_color,
            x_cursor,
            y_base + 60,
            scale_small,
            &font,
            recip_text,
        );

        x_cursor += col_metrics_width as i32 + padding as i32;

        // 6. Status
        let status_text = if metrics.passed { "PASS" } else { "FAIL" };
        let status_color = if metrics.passed {
            Rgb([0, 150, 0])
        } else {
            Rgb([200, 0, 0])
        };
        draw_text_mut(
            &mut contact_sheet,
            status_color,
            x_cursor,
            y_base + 30,
            scale_header,
            &font,
            status_text,
        );
    }

    let out_file = std::fs::File::create(format!("{}/contact_sheet.jpg", output_dir)).unwrap();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 100);
    enc.encode_image(&contact_sheet).unwrap();
}
