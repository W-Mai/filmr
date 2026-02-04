use filmr::film::FilmStock;
use filmr::physics::linear_to_srgb;
use filmr::pipeline::{
    create_output_image, GrainStage, HalationStage, PipelineContext, PipelineStage,
};
use filmr::presets::KODAK_PORTRA_400;
use filmr::processor::{OutputMode, SimulationConfig};
use filmr::spectral::{CameraSensitivities, Spectrum};
use image::{GenericImage, ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use rayon::prelude::*;
use rusttype::{Font, Scale};
use std::fs;
use std::path::PathBuf;

/// Helper to draw a label box
fn draw_label(img: &mut RgbImage, text: &str, x: i32, y: i32, font: &Font) {
    let scale = Scale { x: 20.0, y: 20.0 };
    let padding = 5;

    // Estimate text width (rough)
    let text_width = text.len() as i32 * 12;
    let text_height = 24;

    // Draw background
    draw_filled_rect_mut(
        img,
        Rect::at(x, y).of_size(
            (text_width + padding * 2) as u32,
            (text_height + padding * 2) as u32,
        ),
        Rgb([0, 0, 0]),
    );

    // Draw text
    draw_text_mut(
        img,
        Rgb([255, 255, 255]),
        x + padding,
        y + padding,
        scale,
        font,
        text,
    );
}

/// Stage for developing Film Plane Exposure to Density
/// (Equivalent to DevelopStage but skips the Input->Uplift->Exposure part)
struct SpectralDevelopStage;

impl PipelineStage for SpectralDevelopStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        // Assume 'image' contains Film Plane Exposure values

        image.par_chunks_mut(3).for_each(|pixel| {
            let r_exp = pixel[0];
            let g_exp = pixel[1];
            let b_exp = pixel[2];

            let epsilon = 1e-6;
            let log_e = [
                r_exp.max(epsilon).log10(),
                g_exp.max(epsilon).log10(),
                b_exp.max(epsilon).log10(),
            ];

            let densities = film.map_log_exposure(log_e);
            pixel[0] = densities[0];
            pixel[1] = densities[1];
            pixel[2] = densities[2];
        });
    }
}

/// Custom pipeline to process Spectrum directly to Film Density -> Output
fn process_spectral_image<F>(
    width: u32,
    height: u32,
    film: &FilmStock,
    config: &SimulationConfig,
    generator: F,
) -> (RgbImage, RgbImage)
where
    F: Fn(u32, u32) -> Spectrum + Sync + Send,
{
    let mut input_img = RgbImage::new(width, height);

    let film_sens = film.get_spectral_sensitivities();
    let cam_sens = CameraSensitivities::srgb();

    let pixel_count = (width * height) as usize;
    let mut results: Vec<(Rgb<u8>, Rgb<f32>)> = Vec::with_capacity(pixel_count);
    results.resize(pixel_count, (Rgb([0, 0, 0]), Rgb([0.0, 0.0, 0.0])));

    results.par_iter_mut().enumerate().for_each(|(i, res)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;

        let spectrum = generator(x, y);

        // 1. Input Visualization: Spectrum -> Camera -> sRGB
        let r_cam = cam_sens.r_curve.integrate_product(&spectrum);
        let g_cam = cam_sens.g_curve.integrate_product(&spectrum);
        let b_cam = cam_sens.b_curve.integrate_product(&spectrum);

        let r_srgb = linear_to_srgb(r_cam.clamp(0.0, 1.0));
        let g_srgb = linear_to_srgb(g_cam.clamp(0.0, 1.0));
        let b_srgb = linear_to_srgb(b_cam.clamp(0.0, 1.0));

        res.0 = Rgb([
            (r_srgb * 255.0) as u8,
            (g_srgb * 255.0) as u8,
            (b_srgb * 255.0) as u8,
        ]);

        // 2. Film Processing: Spectrum -> Film Sensitivity -> Exposure
        let exposure_vals = film_sens.expose(&spectrum);

        res.1 = Rgb([
            exposure_vals[0] * config.exposure_time,
            exposure_vals[1] * config.exposure_time,
            exposure_vals[2] * config.exposure_time,
        ]);
    });

    // Fill buffers
    let mut processing_buffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);
    for (i, (in_px, exp_px)) in results.into_iter().enumerate() {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        input_img.put_pixel(x, y, in_px);
        processing_buffer.put_pixel(x, y, exp_px);
    }

    let context = PipelineContext { film, config };

    // Standard Pipeline Stages (reused where possible)
    let stages: Vec<Box<dyn PipelineStage>> = vec![
        Box::new(HalationStage),        // Works on Linear Exposure
        Box::new(SpectralDevelopStage), // Custom stage: Exposure -> Density
        Box::new(GrainStage),           // Works on Density
    ];

    for stage in stages {
        stage.process(&mut processing_buffer, &context);
    }

    // Output (Density -> Display RGB) using the standard pipeline (handles Inversion/Gamma)
    let output_img = create_output_image(&processing_buffer, &context);

    (input_img, output_img)
}

// --- Chart Generators ---

struct ChartResult {
    title: String,
    input: RgbImage,
    output: RgbImage,
    metric: String,
}

fn run_chart_1(film: &FilmStock, config: &SimulationConfig) -> ChartResult {
    let width = 1024;
    let height = 256;
    let (input, output) = process_spectral_image(width, height, film, config, |x, _| {
        let t = x as f32 / width as f32;
        let intensity = 0.5 + t * 2.0;
        Spectrum::new_flat(intensity)
    });

    ChartResult {
        title: "1. Highlight Shoulder (+2EV to +5EV)".to_string(),
        input,
        output,
        metric: "Check for smooth roll-off vs hard clip".to_string(),
    }
}

fn run_chart_2(film: &FilmStock, config: &SimulationConfig) -> ChartResult {
    let width = 1024;
    let height = 256;
    let (input, output) = process_spectral_image(width, height, film, config, |x, _| {
        if x < width / 2 - 1 {
            Spectrum::new_gaussian_normalized(610.0, 20.0) * 50.0 // Red
        } else if x > width / 2 {
            Spectrum::new_gaussian_normalized(450.0, 20.0) * 50.0 // Blue
        } else {
            Spectrum::new_flat(0.0)
        }
    });

    ChartResult {
        title: "2. Interlayer Inhibition (Red vs Blue)".to_string(),
        input,
        output,
        metric: "Check boundary for dark/cyan vignette".to_string(),
    }
}

fn run_chart_3(film: &FilmStock, config: &SimulationConfig) -> ChartResult {
    let width = 1024; // Standardize width
    let height = 256;
    let (input, output) = process_spectral_image(width, height, film, config, |x, _| {
        if x < width / 2 {
            Spectrum::new_flat(0.9)
        } else {
            let mut s = Spectrum::new();
            // Amplitude 0.4 * Width 100 * 2.5 ~= 100.0
            s = s + Spectrum::new_gaussian_normalized(500.0, 100.0) * 100.0;
            // Amplitude 0.8 * Width 80 * 2.5 ~= 160.0
            s = s + Spectrum::new_gaussian_normalized(650.0, 80.0) * 160.0;
            s * 2.0
        }
    });

    ChartResult {
        title: "3. Metamerism (Metal vs Skin)".to_string(),
        input,
        output,
        metric: "Skin (Right) should shift warm vs Metal".to_string(),
    }
}

fn run_chart_4(film: &FilmStock, config: &SimulationConfig) -> ChartResult {
    let width = 1024;
    let height = 256;

    // Create a modified film stock for this test to simulate "neon purple" crosstalk
    // This represents a specific chemical stress scenario where Red layer is sensitive to Blue
    let mut stress_film = film.clone();
    stress_film.spectral_params.r_width = 80.0; // Broaden Red sensitivity to overlap Blue
    stress_film.color_matrix[1][2] = -0.40; // Increase Blue inhibition of Green (Magenta dye)

    let (input, output) = process_spectral_image(width, height, &stress_film, config, |_, _| {
        // Boost amplitude to simulate bright neon light source
        // Narrow band sources need high total energy to match broadband energy levels
        // Previous: Amp 25.0 * Width 8.5 * 2.5 ~= 530.0
        Spectrum::new_gaussian_normalized(480.0, 8.5) * 530.0
    });

    ChartResult {
        title: "4. Spectral Resolution (480nm Neon) [Stress Modified]".to_string(),
        input,
        output,
        metric: "Expect purple/deep cyan shift".to_string(),
    }
}

fn run_chart_5(film: &FilmStock, config: &SimulationConfig) -> ChartResult {
    let width = 1024;
    let height = 256;
    let (input, output) = process_spectral_image(width, height, film, config, |x, _| {
        if x < width / 2 {
            Spectrum::new_blackbody(3200.0)
        } else {
            Spectrum::new_blackbody(8000.0)
        }
    });

    ChartResult {
        title: "5. Mixed WB (3200K vs 8000K)".to_string(),
        input,
        output,
        metric: "Check natural WB retention".to_string(),
    }
}

fn run_chart_6(film: &FilmStock, config: &SimulationConfig) -> ChartResult {
    let width = 1024;
    let height = 512; // Taller
    let (input, output) = process_spectral_image(width, height, film, config, |x, y| {
        let dx = x as f32 - width as f32 / 2.0;
        let dy = y as f32 - height as f32 / 2.0;
        let angle = dy.atan2(dx);
        let _dist = (dx * dx + dy * dy).sqrt();

        let num_spokes = 120.0;
        let val = 0.5 * (1.0 + (angle * num_spokes).cos());

        let density_target = 0.2 + (x as f32 / width as f32) * 2.3;
        let gamma = 0.65;
        let intensity_scale = 10.0f32.powf(density_target / gamma) * 0.01;

        let final_intensity = val * intensity_scale;
        Spectrum::new_flat(final_intensity)
    });

    ChartResult {
        title: "6. Grain Density (Starburst)".to_string(),
        input,
        output,
        metric: "High density (Right) should be grainier".to_string(),
    }
}

fn main() {
    let film = KODAK_PORTRA_400();
    let config = SimulationConfig {
        exposure_time: 1.0,
        enable_grain: true,
        output_mode: OutputMode::Positive,
        white_balance_mode: filmr::processor::WhiteBalanceMode::Auto,
        white_balance_strength: 1.0,
        ..Default::default()
    };

    let font_data = include_bytes!("../apps/filmr/static/ark-pixel-12px-monospaced-zh_cn.otf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    println!("Generating CHEMICAL STRESS CHARTS (Combined)...");

    let results = vec![
        run_chart_1(&film, &config),
        run_chart_2(&film, &config),
        run_chart_3(&film, &config),
        run_chart_4(&film, &config),
        run_chart_5(&film, &config),
        run_chart_6(&film, &config),
    ];

    // Calculate total height
    // Each chart: input_h + output_h + header_space
    let header_h = 40;
    let spacing = 20;
    let mut total_height = 0;
    let max_width = 1024;

    for r in &results {
        total_height += header_h + r.input.height() + r.output.height() + spacing;
    }

    let mut canvas = RgbImage::new(max_width, total_height);
    // Fill white background
    for p in canvas.pixels_mut() {
        *p = Rgb([240, 240, 240]);
    }

    let mut current_y = 0;

    for r in results {
        // Draw Header
        let title_text = format!("{} | {}", r.title, r.metric);
        draw_label(&mut canvas, &title_text, 10, current_y as i32 + 5, &font);

        // Print color stats for analysis
        if r.title.contains("480nm Neon") {
            let center_pixel = r
                .output
                .get_pixel(r.output.width() / 2, r.output.height() / 2);
            let input_pixel = r.input.get_pixel(r.input.width() / 2, r.input.height() / 2);
            println!("Chart 4 Analysis (Center Pixel):");
            println!(
                "  Input (sRGB Sim): R={}, G={}, B={}",
                input_pixel[0], input_pixel[1], input_pixel[2]
            );
            println!(
                "  Output (Film Sim): R={}, G={}, B={}",
                center_pixel[0], center_pixel[1], center_pixel[2]
            );
        }

        current_y += header_h;

        // Draw Input
        canvas.copy_from(&r.input, 0, current_y).unwrap();
        draw_label(
            &mut canvas,
            "INPUT (sRGB Simulation)",
            10,
            current_y as i32 + 10,
            &font,
        );
        current_y += r.input.height();

        // Draw Output
        canvas.copy_from(&r.output, 0, current_y).unwrap();
        draw_label(
            &mut canvas,
            "OUTPUT (Film Simulation)",
            10,
            current_y as i32 + 10,
            &font,
        );
        current_y += r.output.height();

        current_y += spacing;
    }

    let path = PathBuf::from("charts/chemical_stress_test_report.png");
    if !path.parent().unwrap().exists() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    canvas.save(&path).expect("Failed to save report");

    println!("Report saved to: {:?}", path);
}
