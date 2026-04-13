/// Full RGB histogram comparison between Fast and Accurate.
/// Uses a synthetic scene that covers: skin tones, sky gradient, shadows,
/// highlights, saturated colors, and neutral gray — no external file needed.
use filmr::presets::kodak::KODAK_PORTRA_400;
use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, SimulationMode,
    WhiteBalanceMode,
};
use image::{Rgb, RgbImage};

/// Generate a synthetic "scene" image that exercises the full tonal range.
/// Layout (4 rows):
///   Row 0: gray gradient 0-255
///   Row 1: skin tones gradient + sky blue gradient
///   Row 2: 8 saturated color patches (R,G,B,C,M,Y, warm, cool)
///   Row 3: shadow-to-highlight with mixed colors
fn make_scene(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        let fx = x as f32 / w as f32;
        let fy = y as f32 / h as f32;
        let row = (fy * 4.0) as u32;

        match row {
            0 => {
                // Gray gradient
                let v = (fx * 255.0) as u8;
                Rgb([v, v, v])
            }
            1 => {
                if fx < 0.5 {
                    // Skin tones: warm beige to brown
                    let t = fx * 2.0;
                    let r = (220.0 - t * 100.0) as u8;
                    let g = (180.0 - t * 80.0) as u8;
                    let b = (150.0 - t * 70.0) as u8;
                    Rgb([r, g, b])
                } else {
                    // Sky gradient: light blue to deep blue
                    let t = (fx - 0.5) * 2.0;
                    let r = (180.0 - t * 140.0) as u8;
                    let g = (210.0 - t * 120.0) as u8;
                    let b = (240.0 - t * 40.0) as u8;
                    Rgb([r, g, b])
                }
            }
            2 => {
                // 8 color patches
                let patch = (fx * 8.0) as u32;
                match patch {
                    0 => Rgb([220, 50, 50]),   // Red
                    1 => Rgb([50, 200, 50]),   // Green
                    2 => Rgb([50, 50, 220]),   // Blue
                    3 => Rgb([50, 200, 200]),  // Cyan
                    4 => Rgb([200, 50, 200]),  // Magenta
                    5 => Rgb([220, 220, 50]),  // Yellow
                    6 => Rgb([200, 150, 100]), // Warm (orange)
                    _ => Rgb([100, 150, 200]), // Cool (steel blue)
                }
            }
            _ => {
                // Shadow-to-highlight with color variation
                let base = (fx * 240.0) as u8;
                let r = base.saturating_add((20.0 * (fx * std::f32::consts::TAU).sin()) as u8);
                let g = base;
                let b = base.saturating_add((15.0 * (fx * 3.0 * std::f32::consts::PI).cos()) as u8);
                Rgb([r, g, b])
            }
        }
    })
}

fn simple_config(mode: SimulationMode, t: f32) -> SimulationConfig {
    SimulationConfig {
        simulation_mode: mode,
        exposure_time: t,
        enable_grain: false,
        use_gpu: false,
        output_mode: OutputMode::Positive,
        white_balance_mode: WhiteBalanceMode::Off,
        warmth: 0.0,
        saturation: 1.0,
        ..Default::default()
    }
}

fn percentiles_from_hist(hist: &[u32; 256], total: u32) -> [u8; 7] {
    let targets = [1, 5, 25, 50, 75, 95, 99];
    let mut result = [0u8; 7];
    let mut cumsum = 0u32;
    let mut ti = 0;
    for (val, &count) in hist.iter().enumerate() {
        cumsum += count;
        while ti < 7 && cumsum * 100 >= targets[ti] as u32 * total {
            result[ti] = val as u8;
            ti += 1;
        }
    }
    result
}

fn full_histogram(img: &RgbImage) -> ([u32; 256], [u32; 256], [u32; 256]) {
    let mut r = [0u32; 256];
    let mut g = [0u32; 256];
    let mut b = [0u32; 256];
    for p in img.pixels() {
        r[p[0] as usize] += 1;
        g[p[1] as usize] += 1;
        b[p[2] as usize] += 1;
    }
    (r, g, b)
}

fn compare(img: &RgbImage, film_name: &str, film: &filmr::FilmStock) {
    let t = estimate_exposure_time(img, film);
    let total = img.width() * img.height();

    let out_f = process_image(img, film, &simple_config(SimulationMode::Fast, t));
    let out_a = process_image(img, film, &simple_config(SimulationMode::Accurate, t));

    let (rf, gf, bf) = full_histogram(&out_f);
    let (ra, ga, ba) = full_histogram(&out_a);

    eprintln!("\n=== {} (t={:.3}) ===", film_name, t);

    let mut max_diff = 0i16;
    for (ch, hf, ha) in [("R", &rf, &ra), ("G", &gf, &ga), ("B", &bf, &ba)] {
        let pf = percentiles_from_hist(hf, total);
        let pa = percentiles_from_hist(ha, total);
        eprintln!(
            "  {} Fast:     p1={:3} p5={:3} p25={:3} p50={:3} p75={:3} p95={:3} p99={:3}",
            ch, pf[0], pf[1], pf[2], pf[3], pf[4], pf[5], pf[6]
        );
        eprintln!(
            "  {} Accurate: p1={:3} p5={:3} p25={:3} p50={:3} p75={:3} p95={:3} p99={:3}",
            ch, pa[0], pa[1], pa[2], pa[3], pa[4], pa[5], pa[6]
        );
        for (&f, &a) in pf.iter().zip(pa.iter()) {
            max_diff = max_diff.max((a as i16 - f as i16).abs());
        }
    }
    eprintln!("  Max percentile diff: {}", max_diff);
    assert!(
        max_diff <= 15,
        "{}: Fast/Accurate histogram diff too large: {}",
        film_name,
        max_diff
    );
}

#[test]
fn histogram_synthetic_scene() {
    let img = make_scene(800, 600);
    compare(&img, "Daylight", &STANDARD_DAYLIGHT());
    compare(&img, "Portra400", &KODAK_PORTRA_400());
}
