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

        // Warmup / Context Init
        // Note: The first run might include shader compilation and context creation overhead.
        // Ideally, we should measure steady state, but for this SOP, a single run is often indicative enough
        // if we consider "cold start" vs "warm".
        // process_image creates context internally if not cached (but our current impl creates context per call for pipelines).
        // Actually, GpuContext is global/static in some places or passed around?
        // Let's just run it.

        let start_gpu = Instant::now();
        let _output_gpu = process_image(&input, &film, &config_gpu);
        let duration_gpu = start_gpu.elapsed();
        println!("GPU Processing Time: {:.2?}", duration_gpu);

        // Calculate Speedup
        let speedup = duration_cpu.as_secs_f64() / duration_gpu.as_secs_f64();
        println!("GPU Speedup: {:.2}x", speedup);
    }
    #[cfg(not(feature = "compute-gpu"))]
    {
        println!("GPU feature not enabled. Skipping GPU benchmark.");
    }

    println!("\n============================================================");
    println!("Benchmark Complete.");
}
