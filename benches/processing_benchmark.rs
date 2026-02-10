use criterion::{criterion_group, criterion_main, Criterion};
use filmr::presets::kodak::KODAK_PORTRA_400;
use filmr::processor::{process_image, SimulationConfig};
use image::{Rgb, RgbImage};

fn benchmark_processing(c: &mut Criterion) {
    let film = KODAK_PORTRA_400();

    // CPU Config
    let config_cpu = SimulationConfig::default();

    // GPU Config (if feature enabled)
    #[cfg(feature = "compute-gpu")]
    let config_gpu = SimulationConfig {
        use_gpu: true,
        light_leak: filmr::light_leak::LightLeakConfig {
            enabled: true,
            leaks: vec![filmr::light_leak::LightLeak::default()],
        },
        ..Default::default()
    };

    // 720p (HD) - Faster for CI
    let width = 1280;
    let height = 720;
    let input = RgbImage::from_fn(width, height, |x, y| {
        Rgb([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8])
    });

    let mut group = c.benchmark_group("film_simulation");
    group.sample_size(10); // Reduced sample size for heavy operations

    group.bench_function("720p_cpu", |b| {
        b.iter(|| process_image(&input, &film, &config_cpu))
    });

    #[cfg(feature = "compute-gpu")]
    group.bench_function("720p_gpu", |b| {
        b.iter(|| process_image(&input, &film, &config_gpu))
    });

    group.finish();
}

criterion_group!(benches, benchmark_processing);
criterion_main!(benches);
