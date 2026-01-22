#[cfg(test)]
mod tests {
    use filmr::presets::STANDARD_DAYLIGHT;
    use filmr::processor::{process_image, OutputMode, SimulationConfig};
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
        let mut film = STANDARD_DAYLIGHT;
        // Set d_min to 0 to ensure zero exposure results in black
        film.r_curve.d_min = 0.0;
        film.g_curve.d_min = 0.0;
        film.b_curve.d_min = 0.0;

        film.spectral_sensitivity = [
            [0.0, 0.0, 0.0], // Red layer sees NOTHING
            [0.0, 1.0, 0.0], // Green layer sees Green
            [0.0, 0.0, 1.0], // Blue layer sees Blue
        ];
        // Disable grain and halation for pure color check
        film.grain_model.alpha = 0.0;
        film.grain_model.sigma_read = 0.0;
        film.halation_strength = 0.0;

        let config = SimulationConfig {
            exposure_time: 1.0,
            enable_grain: false,
            output_mode: OutputMode::Positive,
        };

        let output = process_image(&input, &film, &config);

        // Analyze output
        // Input was Red. Film is Red-blind.
        // Exposure should be 0 on all layers (since input has only Red, and layers ignore Red).
        // Output should be Black (or very dark, D_min).

        for p in output.pixels() {
            // Should be black
            assert!(
                p[0] < 10 && p[1] < 10 && p[2] < 10,
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
        let mut film = STANDARD_DAYLIGHT;
        // Set d_min to 0
        film.r_curve.d_min = 0.0;
        film.g_curve.d_min = 0.0;
        film.b_curve.d_min = 0.0;

        film.spectral_sensitivity = [
            [0.0, 1.0, 0.0], // Red layer sees GREEN
            [0.0, 1.0, 0.0], // Green layer sees GREEN
            [0.0, 0.0, 1.0], // Blue layer sees BLUE
        ];
        // Identity color matrix to ensure we see the layer densities directly
        film.color_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        film.grain_model.alpha = 0.0;
        film.grain_model.sigma_read = 0.0;
        film.halation_strength = 0.0;

        let config = SimulationConfig {
            exposure_time: 1.0, // Standard exposure
            enable_grain: false,
            output_mode: OutputMode::Positive,
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
        assert!(center[2] < 50, "Blue channel should be dark");
    }
}
