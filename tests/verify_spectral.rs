#[cfg(test)]
mod tests {
    use filmr::presets::STANDARD_DAYLIGHT;
    use filmr::processor::{process_image, OutputMode, SimulationConfig, WhiteBalanceMode};
    use filmr::spectral::{
        CameraSensitivities, FilmSensitivities, FilmSpectralParams, Spectrum, BINS, LAMBDA_START,
        LAMBDA_STEP,
    };
    use image::{Rgb, RgbImage};

    #[test]
    fn test_orthochromatic_response() {
        // Setup a pure Red image
        let width = 10;
        let height = 10;
        let mut input = RgbImage::new(width, height);
        for p in input.pixels_mut() {
            *p = Rgb([255, 0, 0]); // Pure Red
        }

        // Setup Film with Red Blindness (Orthochromatic simulation)
        let mut film = STANDARD_DAYLIGHT();
        // Set d_min to 0 to ensure zero exposure results in black
        film.r_curve.d_min = 0.0;
        film.g_curve.d_min = 0.0;
        film.b_curve.d_min = 0.0;

        // Red Blindness (Orthochromatic simulation)
        // Use narrow curves to minimize overlap with sRGB Red tail
        film.spectral_params = FilmSpectralParams {
            r_peak: 0.0,
            r_width: 0.0,
            g_peak: 500.0,
            g_width: 20.0, // Shifted to 500nm to avoid Red tail
            b_peak: 440.0,
            b_width: 20.0,
        };
        // Disable grain and halation for pure color check
        film.grain_model.alpha = 0.0;
        film.grain_model.sigma_read = 0.0;
        film.halation_strength = 0.0;

        let config = SimulationConfig {
            exposure_time: 1.0,
            enable_grain: false,
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
            ..Default::default()
        };

        let output = process_image(&input, &film, &config);

        // Analyze output
        // Input was Red. Film is Red-blind.
        // Exposure should be 0 on all layers (since input has only Red, and layers ignore Red).
        // Output should be Black (or very dark, D_min).

        for p in output.pixels() {
            // Should be black
            assert!(
                p[0] < 40 && p[1] < 40 && p[2] < 40,
                "Red blind film should render Red light as black, got {:?}",
                p
            );
        }
    }

    #[test]
    fn test_cross_sensitivity() {
        // Setup a pure Green image
        let width = 10;
        let height = 10;
        let mut input = RgbImage::new(width, height);
        for p in input.pixels_mut() {
            *p = Rgb([0, 255, 0]); // Pure Green
        }

        // Setup Film where Red Layer is sensitive to Green Light (Cross talk)
        let mut film = STANDARD_DAYLIGHT();
        // Set d_min to 0
        film.r_curve.d_min = 0.0;
        film.g_curve.d_min = 0.0;
        film.b_curve.d_min = 0.0;

        film.spectral_params = FilmSpectralParams {
            r_peak: 540.0,
            r_width: 20.0, // Red layer sees GREEN (Cross talk), Narrow
            g_peak: 540.0,
            g_width: 20.0, // Green layer sees GREEN, Narrow
            b_peak: 440.0,
            b_width: 20.0, // Blue layer sees BLUE, Narrow
        };
        // Identity color matrix to ensure we see the layer densities directly
        film.color_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        film.grain_model.alpha = 0.0;
        film.grain_model.sigma_read = 0.0;
        film.halation_strength = 0.0;

        let config = SimulationConfig {
            exposure_time: 1.0, // Standard exposure
            enable_grain: false,
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Off, // Must be Off to see true color response
            white_balance_strength: 1.0,
            ..Default::default()
        };

        let output = process_image(&input, &film, &config);

        // Input Green.
        // R_layer sees Green -> Exposes.
        // G_layer sees Green -> Exposes.
        // B_layer sees nothing.
        //
        // Output (Positive):
        // R_channel corresponds to R_layer exposure.
        // G_channel corresponds to G_layer exposure.
        // B_channel corresponds to B_layer exposure (Dark).
        //
        // Wait, Positive output means High Exposure -> Bright Pixel.
        // So R and G should be Bright. B should be Dark.
        // Result: Yellow (R+G).

        let center = output.get_pixel(5, 5);
        println!("Cross Sensitivity Output: {:?}", center);

        assert!(
            center[0] > 100,
            "Red channel should be bright due to cross sensitivity"
        );
        assert!(center[1] > 100, "Green channel should be bright");
        assert!(center[2] < 60, "Blue channel should be dark");
    }

    #[test]
    fn test_rgb_uplift_spectra() {
        let camera = CameraSensitivities::srgb();

        // Helper to find peak wavelength
        let find_peak = |spectrum: &filmr::spectral::Spectrum| -> f32 {
            let mut max_val = -1.0;
            let mut peak_idx = 0;
            for i in 0..BINS {
                if spectrum.power[i] > max_val {
                    max_val = spectrum.power[i];
                    peak_idx = i;
                }
            }
            (LAMBDA_START + peak_idx * LAMBDA_STEP) as f32
        };

        // Test Red Uplift
        let red_spectrum = camera.uplift(1.0, 0.0, 0.0);
        let red_peak = find_peak(&red_spectrum);
        println!("Red Peak: {} nm", red_peak);
        assert!(
            (red_peak - 610.0).abs() < 10.0,
            "Red peak should be around 610nm"
        );

        // Test Green Uplift
        let green_spectrum = camera.uplift(0.0, 1.0, 0.0);
        let green_peak = find_peak(&green_spectrum);
        println!("Green Peak: {} nm", green_peak);
        assert!(
            (green_peak - 540.0).abs() < 10.0,
            "Green peak should be around 540nm"
        );

        // Test Blue Uplift
        let blue_spectrum = camera.uplift(0.0, 0.0, 1.0);
        let blue_peak = find_peak(&blue_spectrum);
        println!("Blue Peak: {} nm", blue_peak);
        assert!(
            (blue_peak - 465.0).abs() < 10.0,
            "Blue peak should be around 465nm"
        );
    }

    #[test]
    fn test_white_spectrum_energy() {
        let camera = CameraSensitivities::srgb();
        let white_spectrum = camera.uplift(1.0, 1.0, 1.0);

        // White spectrum should have energy across the board (sum of 3 Gaussians)
        // Let's check a few points
        let get_val = |nm: usize| -> f32 {
            let idx = (nm - LAMBDA_START) / LAMBDA_STEP;
            if idx < BINS {
                white_spectrum.power[idx]
            } else {
                0.0
            }
        };

        let v_blue = get_val(450);
        let v_green = get_val(540);
        let v_red = get_val(610);

        // With normalized gaussians (Area=1), peaks are much lower (~0.013 for sigma=30)
        // We check for presence of energy, not arbitrary amplitude 0.5
        assert!(v_blue > 0.005, "White light should have Blue component");
        assert!(v_green > 0.005, "White light should have Green component");
        assert!(v_red > 0.005, "White light should have Red component");
    }

    #[test]
    fn test_gray_spectrum_probe() {
        let gray_spectrum = Spectrum::new_flat(1.0);
        let film_params = FilmSpectralParams::new_panchromatic();
        let mut film_sens = FilmSensitivities::from_params(film_params);

        // Calibrate to the flat spectrum so we expect balanced response
        film_sens.calibrate_to_white_point(&gray_spectrum);

        let hb = film_sens.b_sensitivity.integrate_product(&gray_spectrum) * film_sens.b_factor;
        let hg = film_sens.g_sensitivity.integrate_product(&gray_spectrum) * film_sens.g_factor;
        let hr = film_sens.r_sensitivity.integrate_product(&gray_spectrum) * film_sens.r_factor;
        println!("Gray H {},{},{}", hb, hg, hr);
        let max_h = hb.max(hg).max(hr);
        let min_h = hb.min(hg).min(hr);
        assert!((max_h / min_h) < 1.05);
    }

    #[test]
    fn test_white_balance() {
        let gray_spectrum = Spectrum::new_flat(1.0);
        let film_params = FilmSpectralParams::new_panchromatic();
        let mut film_sens = FilmSensitivities::from_params(film_params);

        // Calibrate to the flat spectrum so we expect balanced response
        film_sens.calibrate_to_white_point(&gray_spectrum);

        let exposure = film_sens.expose(&gray_spectrum);
        let r_response = exposure[0];
        let g_response = exposure[1];
        let b_response = exposure[2];

        println!("White Balance Response:");
        println!("R: {:.4}", r_response);
        println!("G: {:.4}", g_response);
        println!("B: {:.4}", b_response);

        let max_resp = r_response.max(g_response).max(b_response);
        let min_resp = r_response.min(g_response).min(b_response);

        let ratio = min_resp / max_resp;
        println!("Min/Max Ratio: {:.2}", ratio);

        assert!(ratio > 0.95);
    }
}
