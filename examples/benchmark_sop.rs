use filmr::presets::KODAK_PORTRA_400;
use filmr::processor::{process_image, SimulationConfig, WhiteBalanceMode};
use image::{Rgb, RgbImage};
use std::time::Instant;

fn main() {
    // Enable logging for tracing (requires tracing-subscriber in dev-dependencies or just use standard logging if not available)
    // Since this is an example, we might not have tracing-subscriber in the main dependencies.
    // Let's check if we can skip it or just use println.
    // tracing_subscriber::fmt::init();
    println!("Tracing initialization skipped (tracing-subscriber not in dev-dependencies)");

    println!("============================================================");
    println!("   Filmr Performance Benchmark (SOP)");
    println!("============================================================");
    println!("Image Size: 24MP (6000x4000)");

    // 1. Generate 24MP Image
    let width = 6000;
    let height = 4000;
    println!("Generating test image...");
    let start_gen = Instant::now();
    let input = RgbImage::from_fn(width, height, |x, y| {
        Rgb([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8])
    });
    println!("Image generation took: {:.2?}", start_gen.elapsed());

    let film = KODAK_PORTRA_400();

    // 2. CPU Benchmark
    println!("\n------------------------------------------------------------");
    println!("Running CPU Benchmark...");
    let config_cpu = SimulationConfig {
        use_gpu: false,
        exposure_time: 0.01, // 1/100s
        enable_grain: true,
        white_balance_mode: WhiteBalanceMode::Auto,
        ..Default::default()
    };

    let start_cpu = Instant::now();
    let _output_cpu = process_image(&input, &film, &config_cpu);
    let duration_cpu = start_cpu.elapsed();
    println!("CPU Processing Time: {:.2?}", duration_cpu);

    // 3. GPU Benchmark (if available)
    println!("\n------------------------------------------------------------");
    println!("Running GPU Benchmark...");
    #[cfg(feature = "compute-gpu")]
    {
        // GPU Config
        let config_gpu = SimulationConfig {
            use_gpu: true,
            exposure_time: 0.01,
            enable_grain: true,
            white_balance_mode: WhiteBalanceMode::Auto,
            light_leak: filmr::light_leak::LightLeakConfig {
                enabled: true, // Enable light leak to test full pipeline
                leaks: vec![filmr::light_leak::LightLeak::default()],
            },
            ..Default::default()
        };

        // 3.1 Cold Start (First Run)
        println!("  -> Running Cold Start (Initialization + Processing)...");
        let start_cold = Instant::now();
        let _ = process_image(&input, &film, &config_gpu);
        let duration_cold = start_cold.elapsed();
        println!("  GPU Cold Start Time: {:.2?}", duration_cold);

        // 3.2 Hot Runs (Steady State)
        println!("  -> Running Hot Iterations (3 runs)...");
        let mut total_hot_duration = std::time::Duration::new(0, 0);
        for i in 1..=3 {
            let start_hot = Instant::now();
            let _ = process_image(&input, &film, &config_gpu);
            let duration = start_hot.elapsed();
            println!("    Run {}: {:.2?}", i, duration);
            total_hot_duration += duration;
        }
        let avg_gpu = total_hot_duration / 3;
        println!("  GPU Avg Hot Time: {:.2?}", avg_gpu);

        // Calculate Speedups
        let speedup_cold = duration_cpu.as_secs_f64() / duration_cold.as_secs_f64();
        let speedup_hot = duration_cpu.as_secs_f64() / avg_gpu.as_secs_f64();

        println!("\n------------------------------------------------------------");
        println!("   Final Results Comparison");
        println!("------------------------------------------------------------");
        println!("CPU Time:        {:.2?}", duration_cpu);
        println!(
            "GPU Cold Time:   {:.2?} (Speedup: {:.2}x)",
            duration_cold, speedup_cold
        );
        println!(
            "GPU Hot Time:    {:.2?} (Speedup: {:.2}x)",
            avg_gpu, speedup_hot
        );

        if avg_gpu < duration_cpu {
            println!("\n✅ GPU is FASTER in steady state!");
        } else {
            println!("\n⚠️ GPU is SLOWER even in steady state. Optimization needed.");
        }
    }
    #[cfg(not(feature = "compute-gpu"))]
    {
        println!("GPU feature not enabled. Skipping GPU benchmark.");
    }

    println!("\n============================================================");
    println!("Benchmark Complete.");
}
