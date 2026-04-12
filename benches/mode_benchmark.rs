use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use filmr::presets::kodak::KODAK_PORTRA_400;
use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{process_image, OutputMode, SimulationConfig, SimulationMode};
use image::{Rgb, RgbImage};

fn make_gradient(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        Rgb([
            (x * 255 / w) as u8,
            (y * 255 / h) as u8,
            ((x + y) * 128 / (w + h)) as u8,
        ])
    })
}

fn config_for(mode: SimulationMode) -> SimulationConfig {
    SimulationConfig {
        simulation_mode: mode,
        exposure_time: 1.0,
        enable_grain: false,
        output_mode: OutputMode::Positive,
        ..Default::default()
    }
}

fn bench_fast_vs_accurate(c: &mut Criterion) {
    let film = KODAK_PORTRA_400();
    let sizes: &[(u32, u32, &str)] = &[(256, 256, "256"), (512, 512, "512"), (1024, 1024, "1K")];

    let mut group = c.benchmark_group("fast_vs_accurate");
    group.sample_size(10);

    for &(w, h, label) in sizes {
        let img = make_gradient(w, h);

        group.bench_with_input(BenchmarkId::new("Fast", label), &img, |b, img| {
            let cfg = config_for(SimulationMode::Fast);
            b.iter(|| process_image(img, &film, &cfg))
        });

        group.bench_with_input(BenchmarkId::new("Accurate", label), &img, |b, img| {
            let cfg = config_for(SimulationMode::Accurate);
            b.iter(|| process_image(img, &film, &cfg))
        });
    }

    group.finish();
}

fn bench_accurate_by_preset(c: &mut Criterion) {
    let img = make_gradient(512, 512);
    let cfg = config_for(SimulationMode::Accurate);

    let presets: Vec<(&str, filmr::FilmStock)> = vec![
        ("Daylight_8layer", STANDARD_DAYLIGHT()),
        ("Portra400_11layer", KODAK_PORTRA_400()),
    ];

    let mut group = c.benchmark_group("accurate_presets");
    group.sample_size(10);

    for (name, film) in &presets {
        group.bench_with_input(BenchmarkId::from_parameter(name), &img, |b, img| {
            b.iter(|| process_image(img, film, &cfg))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fast_vs_accurate, bench_accurate_by_preset);
criterion_main!(benches);
