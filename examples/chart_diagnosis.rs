mod generate_chart;
use generate_chart::generate_test_chart;
use filmr::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode, presets,
};
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

    for (name, stock) in stocks.iter() {
        // Sanitize name for filename
        let safe_name = name.replace(" ", "_");
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

        let output_path = format!("{}/chart_{}.jpg", output_dir, safe_name);
        result.save(&output_path).unwrap();
        println!("Saved to {}", output_path);
    }
}
