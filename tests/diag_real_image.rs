use filmr::presets::kodak::KODAK_PORTRA_400;
use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, SimulationMode,
    WhiteBalanceMode,
};
use image::RgbImage;

fn avg_luma(img: &RgbImage) -> f32 {
    let mut sum = 0.0f32;
    for p in img.pixels() {
        sum += 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
    }
    sum / (img.width() * img.height()) as f32
}

fn percentiles(img: &RgbImage) -> (f32, f32, f32) {
    let mut lumas: Vec<f32> = img
        .pixels()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    lumas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = lumas.len();
    (
        lumas[n / 10],     // p10 (shadows)
        lumas[n / 2],      // p50 (midtones)
        lumas[n * 9 / 10], // p90 (highlights)
    )
}

#[test]
fn diag_real_image() {
    let img = image::open("static/snapshot_001.jpeg").unwrap().to_rgb8();
    eprintln!("Image: {}×{}", img.width(), img.height());

    // Downsample for speed
    let small = image::imageops::resize(
        &img,
        512,
        512 * img.height() / img.width(),
        image::imageops::FilterType::Triangle,
    );
    let small = RgbImage::from_raw(small.width(), small.height(), small.into_raw()).unwrap();

    for (film_name, film) in [
        ("Daylight", STANDARD_DAYLIGHT()),
        ("Portra400", KODAK_PORTRA_400()),
    ] {
        let t = estimate_exposure_time(&small, &film);

        for mode in [SimulationMode::Fast, SimulationMode::Accurate] {
            let config = SimulationConfig {
                simulation_mode: mode,
                exposure_time: t,
                enable_grain: false,
                output_mode: OutputMode::Positive,
                white_balance_mode: WhiteBalanceMode::Auto,
                ..Default::default()
            };
            let out = process_image(&small, &film, &config);
            let luma = avg_luma(&out);
            let (p10, p50, p90) = percentiles(&out);
            eprintln!(
                "  {:10} {:>8} | t={:.3} avg={:.0} p10={:.0} p50={:.0} p90={:.0}",
                film_name,
                format!("{:?}", mode),
                t,
                luma,
                p10,
                p50,
                p90
            );
        }
    }
}
