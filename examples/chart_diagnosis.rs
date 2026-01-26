mod generate_chart;
use filmr::{
    estimate_exposure_time, presets, process_image, FilmMetrics, OutputMode, SimulationConfig,
    WhiteBalanceMode,
};
use generate_chart::generate_test_chart;
use image::imageops::FilterType;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut};
use imageproc::rect::Rect;
use palette::{FromColor, Lab, Srgb};
use rusttype::{Font, Scale};
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() {
    let output_dir = "diagnosis_output";
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir).unwrap();
    }

    println!("Generating Test Chart...");
    let chart = generate_test_chart(1024, 1024);
    // Save the original chart to the output directory as well
    chart
        .save(format!("{}/test_chart.png", output_dir))
        .unwrap();
    println!("Saved test_chart.png to {}", output_dir);

    let stocks = presets::get_all_stocks();
    let mut results = Vec::new();

    for (name, stock) in stocks.iter() {
        println!("\nProcessing {}...", name);
        let start = Instant::now();

        // Estimate exposure
        let t_est = estimate_exposure_time(&chart, stock);
        println!("Estimated Exposure Time: {:.4}s", t_est);

        let config = SimulationConfig {
            exposure_time: t_est,
            enable_grain: false, // Disable grain for cleaner chart analysis
            output_mode: OutputMode::Positive, // Always use Positive for diagnosis
            white_balance_mode: WhiteBalanceMode::Auto, // Test Auto WB
            white_balance_strength: 1.0,
        };

        let result = process_image(&chart, stock, &config);
        let duration = start.elapsed();
        println!("Processed in {:.2?}", duration);

        // Calculate Metrics
        let metrics = FilmMetrics::analyze(&result);

        // Generate Plots
        let hist_img = draw_rgb_histogram(&result, 220, 120);
        let lab_img = draw_lab_scatter(&result, 220, 120);
        let lbp_img = draw_lbp_histogram(&metrics.lbp_hist, 220, 120);

        // Store for contact sheet
        results.push((*name, result, t_est, metrics, hist_img, lab_img, lbp_img));
    }

    println!("Generating contact sheet...");
    generate_contact_sheet(&results, output_dir);
    println!("Contact sheet saved to {}/contact_sheet.jpg", output_dir);
}

fn generate_contact_sheet(
    data: &[(
        &str,
        RgbImage,
        f32,
        FilmMetrics,
        RgbImage,
        RgbImage,
        RgbImage,
    )],
    output_dir: &str,
) {
    let thumb_size = 300;
    let padding = 20;
    let header_height = 40;
    let cols = 3;
    let rows = (data.len() as f32 / cols as f32).ceil() as u32;

    // Cell Layout:
    // [ Image (300x300) ] [ Stats Text ]
    // [ RGB Hist ] [ Lab Scatter ] [ LBP Hist ]

    let cell_width = 750 + padding * 2;
    let cell_height = 500 + padding * 2;

    let total_width = cols * cell_width;
    let total_height = rows * cell_height + header_height;

    let mut contact_sheet = RgbImage::new(total_width, total_height);
    // Dark background
    for p in contact_sheet.pixels_mut() {
        *p = Rgb([30, 30, 30]);
    }

    let font_data = include_bytes!("/System/Library/Fonts/Monaco.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale_title = Scale { x: 20.0, y: 20.0 };
    let scale_text = Scale { x: 13.0, y: 13.0 }; // Slightly smaller text

    for (i, (name, img, t_est, metrics, hist, lab, lbp)) in data.iter().enumerate() {
        let col = i as u32 % cols;
        let row = i as u32 / cols;

        let x_base = col * cell_width + padding;
        let y_base = header_height + row * cell_height + padding;

        // 1. Draw Main Image (Top Left)
        let thumb = image::imageops::resize(img, thumb_size, thumb_size, FilterType::Lanczos3);
        copy_image(&mut contact_sheet, &thumb, x_base, y_base);

        // 2. Draw Stats (Top Right)
        let x_stats = x_base + thumb_size + padding;
        let mut y_text = y_base;

        draw_text_mut(
            &mut contact_sheet,
            Rgb([255, 255, 255]),
            x_stats as i32,
            y_text as i32,
            scale_title,
            &font,
            name,
        );
        y_text += 30;

        let lines = [
            format!("Exp: {:.4}s | DR: {:.1}dB", t_est, metrics.dynamic_range),
            format!(
                "Mean: {:.0}/{:.0}/{:.0} | Entropy: {:.2}",
                metrics.mean_rgb[0], metrics.mean_rgb[1], metrics.mean_rgb[2], metrics.entropy
            ),
            format!(
                "Std:  {:.1}/{:.1}/{:.1}",
                metrics.std_rgb[0], metrics.std_rgb[1], metrics.std_rgb[2]
            ),
            format!(
                "Skew: {:.1}/{:.1}/{:.1}",
                metrics.skewness_rgb[0], metrics.skewness_rgb[1], metrics.skewness_rgb[2]
            ),
            format!(
                "Kurt: {:.1}/{:.1}/{:.1}",
                metrics.kurtosis_rgb[0], metrics.kurtosis_rgb[1], metrics.kurtosis_rgb[2]
            ),
            format!(
                "Clip: Z{:.1}% S{:.1}%",
                metrics.clipping_ratio[0] * 100.0,
                metrics.clipping_ratio[1] * 100.0
            ),
            format!(
                "Lab Mean: {:.1}/{:.1}/{:.1}",
                metrics.lab_mean[0], metrics.lab_mean[1], metrics.lab_mean[2]
            ),
            format!(
                "Sat Mean: {:.1} Skew: {:.1}",
                metrics.saturation_mean, metrics.saturation_skew
            ),
            format!("Rg/Bg: {:.2}/{:.2}", metrics.rg_ratio, metrics.bg_ratio),
            format!(
                "CCT: {:.0}K Tint: {:.4}",
                metrics.cct_tint.0, metrics.cct_tint.1
            ),
            format!("Texture Lap: {:.1}", metrics.laplacian_variance),
            format!("PSD Beta: {:.2}", metrics.psd_slope),
            format!(
                "GLCM: C{:.1} E{:.2} H{:.2}",
                metrics.glcm_stats[0], metrics.glcm_stats[2], metrics.glcm_stats[3]
            ),
        ];

        for line in lines {
            draw_text_mut(
                &mut contact_sheet,
                Rgb([200, 200, 200]),
                x_stats as i32,
                y_text as i32,
                scale_text,
                &font,
                &line,
            );
            y_text += 18;
        }

        // 3. Draw Plots (Bottom Row)
        let y_plots = y_base + 320;
        let plot_w = 220;
        let plot_gap = 20;

        // RGB Hist
        copy_image(&mut contact_sheet, hist, x_base, y_plots);
        draw_text_mut(
            &mut contact_sheet,
            Rgb([150, 150, 150]),
            x_base as i32,
            (y_plots + 125) as i32,
            scale_text,
            &font,
            "RGB Hist",
        );

        // Lab Scatter
        copy_image(&mut contact_sheet, lab, x_base + plot_w + plot_gap, y_plots);
        draw_text_mut(
            &mut contact_sheet,
            Rgb([150, 150, 150]),
            (x_base + plot_w + plot_gap) as i32,
            (y_plots + 125) as i32,
            scale_text,
            &font,
            "Lab Scatter",
        );

        // LBP Hist
        copy_image(
            &mut contact_sheet,
            lbp,
            x_base + (plot_w + plot_gap) * 2,
            y_plots,
        );
        draw_text_mut(
            &mut contact_sheet,
            Rgb([150, 150, 150]),
            (x_base + (plot_w + plot_gap) * 2) as i32,
            (y_plots + 125) as i32,
            scale_text,
            &font,
            "LBP Texture",
        );
    }

    let out_file = std::fs::File::create(format!("{}/contact_sheet.jpg", output_dir)).unwrap();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 95);
    enc.encode_image(&contact_sheet).unwrap();
}

fn copy_image(target: &mut RgbImage, source: &RgbImage, x_start: u32, y_start: u32) {
    for y in 0..source.height() {
        for x in 0..source.width() {
            if x_start + x < target.width() && y_start + y < target.height() {
                let p = source.get_pixel(x, y);
                target.put_pixel(x_start + x, y_start + y, *p);
            }
        }
    }
}

fn draw_rgb_histogram(img: &RgbImage, width: u32, height: u32) -> RgbImage {
    let mut canvas = RgbImage::from_pixel(width, height, Rgb([40, 40, 40]));

    let mut hist_r = [0u32; 256];
    let mut hist_g = [0u32; 256];
    let mut hist_b = [0u32; 256];
    let mut max_count = 0;

    for p in img.pixels() {
        hist_r[p[0] as usize] += 1;
        hist_g[p[1] as usize] += 1;
        hist_b[p[2] as usize] += 1;
    }

    for i in 0..256 {
        max_count = max_count.max(hist_r[i]).max(hist_g[i]).max(hist_b[i]);
    }

    // Robust scaling: Use 99.5th percentile to avoid spikes
    let mut all_counts = Vec::with_capacity(256 * 3);
    for i in 0..256 {
        if hist_r[i] > 0 {
            all_counts.push(hist_r[i]);
        }
        if hist_g[i] > 0 {
            all_counts.push(hist_g[i]);
        }
        if hist_b[i] > 0 {
            all_counts.push(hist_b[i]);
        }
    }
    all_counts.sort_unstable();

    let norm_max = if !all_counts.is_empty() {
        let idx = ((all_counts.len() as f32) * 0.995) as usize;
        all_counts[idx.min(all_counts.len() - 1)] as f32
    } else {
        max_count as f32
    };

    let max_h = (height - 5) as f32; // Small padding
    let scale_y = if norm_max > 0.0 {
        max_h / norm_max
    } else {
        0.0
    };
    let scale_x = width as f32 / 256.0;

    for i in 0..255 {
        let x1 = i as f32 * scale_x;
        let x2 = (i + 1) as f32 * scale_x;

        let mut draw_channel = |hist: &[u32; 256], color: Rgb<u8>| {
            let val1 = (hist[i] as f32).min(norm_max);
            let val2 = (hist[i + 1] as f32).min(norm_max);

            let y1 = height as f32 - (val1 * scale_y);
            let y2 = height as f32 - (val2 * scale_y);
            draw_line_segment_mut(&mut canvas, (x1, y1), (x2, y2), color);
        };

        draw_channel(&hist_r, Rgb([255, 80, 80]));
        draw_channel(&hist_g, Rgb([80, 255, 80]));
        draw_channel(&hist_b, Rgb([80, 80, 255]));
    }

    canvas
}

fn draw_lab_scatter(img: &RgbImage, width: u32, height: u32) -> RgbImage {
    let mut canvas = RgbImage::from_pixel(width, height, Rgb([40, 40, 40]));

    // Draw axes
    draw_line_segment_mut(
        &mut canvas,
        (0.0, height as f32 / 2.0),
        (width as f32, height as f32 / 2.0),
        Rgb([100, 100, 100]),
    );
    draw_line_segment_mut(
        &mut canvas,
        (width as f32 / 2.0, 0.0),
        (width as f32 / 2.0, height as f32),
        Rgb([100, 100, 100]),
    );

    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Sample 2000 points
    for _ in 0..2000 {
        let x = rng.gen_range(0..img.width());
        let y = rng.gen_range(0..img.height());
        let p = img.get_pixel(x, y);

        let srgb = Srgb::new(
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        );
        let lab: Lab = Lab::from_color(srgb);

        // Map a/b (-100..100) to fit
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let scale = 0.8;

        let plot_x = center_x + lab.a * scale;
        let plot_y = center_y - lab.b * scale;

        if plot_x >= 0.0 && plot_x < width as f32 && plot_y >= 0.0 && plot_y < height as f32 {
            canvas.put_pixel(plot_x as u32, plot_y as u32, Rgb([200, 200, 200]));
        }
    }

    canvas
}

fn draw_lbp_histogram(hist: &[f32; 10], width: u32, height: u32) -> RgbImage {
    let mut canvas = RgbImage::from_pixel(width, height, Rgb([40, 40, 40]));

    let mut max_val = 0.0f32;
    for &v in hist {
        max_val = max_val.max(v);
    }

    let bar_width = width as f32 / 10.0;
    let scale_y = if max_val > 0.0 {
        (height - 10) as f32 / max_val
    } else {
        0.0
    };

    for (i, &val) in hist.iter().enumerate().take(10) {
        let h = val * scale_y;
        if h >= 1.0 {
            let x = i as f32 * bar_width;
            let y = height as f32 - h;

            let rect = Rect::at(x as i32, y as i32).of_size(bar_width as u32 - 2, h as u32);
            draw_filled_rect_mut(&mut canvas, rect, Rgb([100, 200, 255]));
        }
    }

    canvas
}
