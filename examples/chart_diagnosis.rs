mod generate_chart;
use generate_chart::generate_test_chart;
use filmr::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode, presets,
};
use image::{Rgb, RgbImage};
use image::imageops::FilterType;
use imageproc::drawing::draw_text_mut;
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
    chart.save(format!("{}/test_chart.png", output_dir)).unwrap();
    println!("Saved test_chart.png to {}", output_dir);

    let stocks = presets::get_all_stocks();
    let mut results = Vec::new();

    for (name, stock) in stocks.iter() {
        // Sanitize name for filename
        // let safe_name = name.replace(" ", "_");
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

        // Analyze result statistics
        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;
        let mut min_val = 255.0;
        let mut max_val = 0.0;
        let pixel_count = (result.width() * result.height()) as f64;

        for p in result.pixels() {
            let r = p[0] as f64;
            let g = p[1] as f64;
            let b = p[2] as f64;
            sum_r += r;
            sum_g += g;
            sum_b += b;
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            if lum < min_val { min_val = lum; }
            if lum > max_val { max_val = lum; }
        }
        
        println!("Stats: Mean RGB=[{:.1}, {:.1}, {:.1}], Lum Range=[{:.1}, {:.1}]", 
                 sum_r / pixel_count, sum_g / pixel_count, sum_b / pixel_count, min_val, max_val);

        // Store for contact sheet
        results.push((
            *name, 
            result, 
            t_est, 
            (sum_r / pixel_count, sum_g / pixel_count, sum_b / pixel_count),
            (min_val, max_val)
        ));
    }

    println!("Generating contact sheet...");
    generate_contact_sheet(&results, output_dir);
    println!("Contact sheet saved to {}/contact_sheet.jpg", output_dir);
}

fn generate_contact_sheet(data: &[(&str, RgbImage, f32, (f64, f64, f64), (f64, f64))], output_dir: &str) {
    let thumb_size = 300;
    let padding = 20;
    let header_height = 40; // For title if we want, or just spacing
    let cols = 4;
    let rows = (data.len() as f32 / cols as f32).ceil() as u32;
    
    let cell_width = thumb_size + padding * 2;
    let cell_height = thumb_size + padding * 2 + 80; // Extra space for text
    
    let total_width = cols * cell_width;
    let total_height = rows * cell_height + header_height;
    
    let mut contact_sheet = RgbImage::new(total_width, total_height);
    // Dark background for professional look
    for p in contact_sheet.pixels_mut() {
        *p = Rgb([30, 30, 30]);
    }

    let font_data = include_bytes!("/System/Library/Fonts/Monaco.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
    let scale = Scale { x: 20.0, y: 20.0 };
    let scale_small = Scale { x: 12.0, y: 12.0 };

    for (i, (name, img, t_est, mean_rgb, lum_range)) in data.iter().enumerate() {
        let col = i as u32 % cols;
        let row = i as u32 / cols;
        
        let x_off = col * cell_width + padding;
        let y_off = header_height + row * cell_height + padding;
        
        // Resize image
        let thumb = image::imageops::resize(img, thumb_size, thumb_size, FilterType::Lanczos3);
        
        // Copy image
        for y in 0..thumb.height() {
            for x in 0..thumb.width() {
                let p = thumb.get_pixel(x, y);
                contact_sheet.put_pixel(x_off + x, y_off + y, *p);
            }
        }
        
        // Draw Label
        draw_text_mut(
            &mut contact_sheet,
            Rgb([200, 200, 200]),
            x_off as i32,
            (y_off + thumb_size + 10) as i32,
            scale,
            &font,
            name
        );

        // Draw Stats
        let stats_text_1 = format!("Exp: {:.4}s", t_est);
        let stats_text_2 = format!("RGB: {:.0}/{:.0}/{:.0}", mean_rgb.0, mean_rgb.1, mean_rgb.2);
        let stats_text_3 = format!("Lum: {:.0}-{:.0}", lum_range.0, lum_range.1);

        draw_text_mut(
            &mut contact_sheet,
            Rgb([150, 150, 150]),
            x_off as i32,
            (y_off + thumb_size + 35) as i32,
            scale_small,
            &font,
            &stats_text_1
        );

        draw_text_mut(
            &mut contact_sheet,
            Rgb([150, 150, 150]),
            x_off as i32,
            (y_off + thumb_size + 50) as i32,
            scale_small,
            &font,
            &stats_text_2
        );

        draw_text_mut(
            &mut contact_sheet,
            Rgb([150, 150, 150]),
            x_off as i32,
            (y_off + thumb_size + 65) as i32,
            scale_small,
            &font,
            &stats_text_3
        );
    }
    
    let out_file = std::fs::File::create(format!("{}/contact_sheet.jpg", output_dir)).unwrap();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 95);
    enc.encode_image(&contact_sheet).unwrap();
}
