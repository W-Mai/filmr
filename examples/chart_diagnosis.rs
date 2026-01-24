mod generate_chart;
use generate_chart::generate_test_chart;
use filmr::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode, presets,
};
use std::time::Instant;

fn main() {
    println!("Generating Test Chart...");
    let chart = generate_test_chart(1024, 1024);
    chart.save("test_chart.png").unwrap();
    println!("Saved test_chart.png");

    let stocks = [
        ("Kodak_Tri-X_400", presets::KODAK_TRI_X_400, OutputMode::Positive),
        ("Fujifilm_Velvia_50", presets::FUJIFILM_VELVIA_50, OutputMode::Positive),
        ("Ilford_HP5_Plus", presets::ILFORD_HP5_PLUS, OutputMode::Positive),
        ("Kodak_Portra_400", presets::KODAK_PORTRA_400, OutputMode::Positive),
    ];

    for (name, stock, mode) in stocks.iter() {
        println!("\nProcessing {}...", name);
        let start = Instant::now();

        // Estimate exposure
        let t_est = estimate_exposure_time(&chart, stock);
        println!("Estimated Exposure Time: {:.4}s", t_est);

        let config = SimulationConfig {
            exposure_time: t_est,
            enable_grain: false, // Disable grain for cleaner chart analysis
            output_mode: *mode,
            white_balance_mode: WhiteBalanceMode::Auto, // Test Auto WB
            white_balance_strength: 1.0,
        };

        let result = process_image(&chart, stock, &config);
        let duration = start.elapsed();
        println!("Processed in {:.2?}", duration);

        let output_path = format!("chart_{}.jpg", name);
        result.save(&output_path).unwrap();
        println!("Saved to {}", output_path);
    }
}
