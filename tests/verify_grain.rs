#[cfg(test)]
mod tests {
    use filmr::presets::KODAK_TRI_X_400;
    use filmr::processor::{process_image, OutputMode, SimulationConfig, WhiteBalanceMode};
    use image::{Rgb, RgbImage};

    #[test]
    fn test_monochrome_grain_consistency() {
        // Setup a flat gray image
        let width = 10;
        let height = 10;
        let mut input = RgbImage::new(width, height);
        for p in input.pixels_mut() {
            *p = Rgb([128, 128, 128]);
        }

        // Setup Monochrome Film (modified Tri-X)
        let mut film = KODAK_TRI_X_400();
        film.grain_model.monochrome = true;
        film.grain_model.alpha = 0.5; // High noise to be sure
        film.grain_model.sigma_read = 0.0;

        let config = SimulationConfig {
            exposure_time: 1.0,
            enable_grain: true,
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
            ..Default::default()
        };

        let output = process_image(&input, &film, &config);

        // Check if R, G, B channels have identical noise deviations (relative to each other)
        // Since the input is gray and film color matrix for Tri-X yields gray,
        // and grain is monochrome, the output pixels should be gray (R=G=B).

        for p in output.pixels() {
            let r = p[0];
            let g = p[1];
            let b = p[2];

            // Allow small rounding differences, but they should be very close
            assert!(
                (r as i16 - g as i16).abs() <= 1,
                "Pixel {:?} is not gray in monochrome mode",
                p
            );
            assert!(
                (r as i16 - b as i16).abs() <= 1,
                "Pixel {:?} is not gray in monochrome mode",
                p
            );
        }
    }

    #[test]
    fn test_color_grain_independence() {
        // Setup a flat gray image
        let width = 10;
        let height = 10;
        let mut input = RgbImage::new(width, height);
        for p in input.pixels_mut() {
            *p = Rgb([128, 128, 128]);
        }

        // Setup Color Film (modified Tri-X but with monochrome=false)
        let mut film = KODAK_TRI_X_400();
        film.grain_model.monochrome = false;
        film.grain_model.alpha = 0.5; // High noise
        film.grain_model.sigma_read = 0.0;
        // Reset color matrix to identity to avoid color shifts from the matrix itself confusing things?
        // Actually, if we use identity matrix, output should be gray *plus* independent noise.
        film.color_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        let config = SimulationConfig {
            exposure_time: 1.0,
            enable_grain: true,
            output_mode: OutputMode::Positive,
            white_balance_mode: WhiteBalanceMode::Auto,
            white_balance_strength: 1.0,
            ..Default::default()
        };

        let output = process_image(&input, &film, &config);

        // Check if R, G, B channels are NOT all identical
        let mut identical_count = 0;
        for p in output.pixels() {
            let r = p[0];
            let g = p[1];
            let b = p[2];

            if r == g && g == b {
                identical_count += 1;
            }
        }

        // With independent noise, it's highly unlikely all pixels are perfectly gray
        assert!(
            identical_count < (width * height) / 2,
            "Too many pixels are gray in color noise mode: {}/{}",
            identical_count,
            width * height
        );
    }
}
