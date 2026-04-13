use filmr::presets::kodak::KODAK_PORTRA_400;
use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, SimulationMode,
    WhiteBalanceMode,
};
use image::{Rgb, RgbImage};

/// Create a test chart: top half = gray gradient, bottom half = color patches
fn make_test_chart(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        if y < h / 2 {
            // Gray gradient 0-255
            let v = (x as f32 / w as f32 * 255.0) as u8;
            Rgb([v, v, v])
        } else {
            // 6 color patches
            let section = x * 6 / w;
            match section {
                0 => Rgb([200, 80, 80]),  // warm red
                1 => Rgb([80, 180, 80]),  // green
                2 => Rgb([80, 80, 200]),  // blue
                3 => Rgb([200, 200, 80]), // yellow
                4 => Rgb([80, 200, 200]), // cyan
                _ => Rgb([200, 80, 200]), // magenta
            }
        }
    })
}

#[test]
fn diag_test_chart() {
    let img = make_test_chart(600, 400);

    for (film_name, film) in [
        ("Daylight", STANDARD_DAYLIGHT()),
        ("Portra400", KODAK_PORTRA_400()),
    ] {
        let t = estimate_exposure_time(&img, &film);

        eprintln!("\n=== {} (t={:.3}) ===", film_name, t);

        for mode in [SimulationMode::Fast, SimulationMode::Accurate] {
            let config = SimulationConfig {
                simulation_mode: mode,
                exposure_time: t,
                enable_grain: false,
                output_mode: OutputMode::Positive,
                white_balance_mode: WhiteBalanceMode::Auto,
                ..Default::default()
            };
            let out = process_image(&img, &film, &config);

            // Sample gray gradient at key points
            let y = out.height() / 4; // middle of gray gradient
            let black = out.get_pixel(0, y);
            let dark = out.get_pixel(out.width() / 4, y);
            let mid = out.get_pixel(out.width() / 2, y);
            let bright = out.get_pixel(out.width() * 3 / 4, y);
            let white = out.get_pixel(out.width() - 1, y);

            // Sample color patches
            let cy = out.height() * 3 / 4;
            let colors: Vec<_> = (0..6)
                .map(|i| {
                    let cx = (i * 2 + 1) * out.width() / 12;
                    *out.get_pixel(cx, cy)
                })
                .collect();

            eprintln!(
                "  {:>8} | black={:?} dark={:?} mid={:?} bright={:?} white={:?}",
                format!("{:?}", mode),
                black.0,
                dark.0,
                mid.0,
                bright.0,
                white.0
            );
            eprintln!(
                "           | R={:?} G={:?} B={:?} Y={:?} C={:?} M={:?}",
                colors[0].0, colors[1].0, colors[2].0, colors[3].0, colors[4].0, colors[5].0
            );
        }
    }
}
