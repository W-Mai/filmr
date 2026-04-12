use filmr::presets::other::STANDARD_DAYLIGHT;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, SimulationMode,
    WhiteBalanceMode,
};
use image::{Rgb, RgbImage};

#[test]
fn diag_fast_vs_accurate() {
    let film = STANDARD_DAYLIGHT();
    let inputs: Vec<(&str, u8, u8, u8)> = vec![
        ("White", 255, 255, 255),
        ("Gray50", 128, 128, 128),
        ("Gray18", 119, 119, 119),
        ("Red", 200, 0, 0),
        ("Green", 0, 200, 0),
        ("Blue", 0, 0, 200),
        ("Yellow", 200, 200, 0),
        ("Cyan", 0, 200, 200),
        ("Magenta", 200, 0, 200),
    ];

    eprintln!(
        "\n{:8} | {:>20} | {:>20} | {:>6}",
        "Input", "Fast", "Accurate", "Δluma"
    );
    eprintln!("{}", "-".repeat(75));

    for (name, r, g, b) in &inputs {
        let img = RgbImage::from_fn(50, 50, |_, _| Rgb([*r, *g, *b]));
        let t = estimate_exposure_time(&img, &film);

        let cfg_f = SimulationConfig {
            simulation_mode: SimulationMode::Fast,
            exposure_time: t,
            enable_grain: false,
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Off,
            ..Default::default()
        };
        let out_f = process_image(&img, &film, &cfg_f);
        let pf = out_f.get_pixel(25, 25);

        let mut cfg_a = cfg_f.clone();
        cfg_a.simulation_mode = SimulationMode::Accurate;
        cfg_a.exposure_time = 1.0;
        let out_a = process_image(&img, &film, &cfg_a);
        let pa = out_a.get_pixel(25, 25);

        let luma_f = 0.299 * pf[0] as f32 + 0.587 * pf[1] as f32 + 0.114 * pf[2] as f32;
        let luma_a = 0.299 * pa[0] as f32 + 0.587 * pa[1] as f32 + 0.114 * pa[2] as f32;

        eprintln!(
            "{:8} | ({:3},{:3},{:3}) t={:.3} | ({:3},{:3},{:3})         | {:+.0}",
            name,
            pf[0],
            pf[1],
            pf[2],
            t,
            pa[0],
            pa[1],
            pa[2],
            luma_a - luma_f
        );
    }
}
