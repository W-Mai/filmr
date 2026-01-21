use image::{RgbImage, Pixel};
use rayon::prelude::*;
use crate::physics;
use crate::film::FilmStock;
use crate::grain::GrainModel;

/// Configuration for the simulation run.
pub struct SimulationConfig {
    pub exposure_time: f32, // t in E = I * t
    pub enable_grain: bool,
    pub output_mode: OutputMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    Negative, // Transmission of the negative (Dark -> Bright, Bright -> Dark)
    Positive, // Scanned/Inverted Positive (Dark -> Dark, Bright -> Bright)
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            exposure_time: 1.0,
            enable_grain: true,
            output_mode: OutputMode::Positive, // Default to what users expect
        }
    }
}

/// Main processor function.
/// Takes an input image and film parameters, returns the simulated image.
pub fn process_image(
    input: &RgbImage,
    film: &FilmStock,
    grain: &GrainModel,
    config: &SimulationConfig,
) -> RgbImage {
    let width = input.width();
    let height = input.height();
    
    // Create a mutable buffer for output
    // We use a vector to allow parallel processing easily, then convert to ImageBuffer
    let mut pixels: Vec<u8> = vec![0; (width * height * 3) as usize];
    
    // Parallel iterator over pixels
    pixels.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        
        let pixel = input.get_pixel(x, y);
        let rgb = pixel.channels();
        
        // 1. Linearize Input (sRGB -> Linear Irradiance)
        let r_lin = physics::srgb_to_linear(rgb[0] as f32 / 255.0);
        let g_lin = physics::srgb_to_linear(rgb[1] as f32 / 255.0);
        let b_lin = physics::srgb_to_linear(rgb[2] as f32 / 255.0);
        
        // 2. Apply Exposure (E = I * t)
        let r_exp = physics::calculate_exposure(r_lin, config.exposure_time);
        let g_exp = physics::calculate_exposure(g_lin, config.exposure_time);
        let b_exp = physics::calculate_exposure(b_lin, config.exposure_time);
        
        // Avoid log(0)
        let epsilon = 1e-6;
        let log_e = [
            r_exp.max(epsilon).log10(),
            g_exp.max(epsilon).log10(),
            b_exp.max(epsilon).log10(),
        ];
        
        // 3. Film Response & Color Coupling (Log E -> Density)
        let densities = film.map_log_exposure(log_e);
        
        // 4. Add Grain (if enabled)
        let final_densities = if config.enable_grain {
            let mut rng = rand::thread_rng(); // Thread-local RNG
            [
                grain.add_grain(densities[0], &mut rng),
                grain.add_grain(densities[1], &mut rng),
                grain.add_grain(densities[2], &mut rng),
            ]
        } else {
            densities
        };
        
        // 5. Output Formatting
        let (r_out, g_out, b_out) = match config.output_mode {
            OutputMode::Negative => {
                // Density -> Transmission (Physical Negative)
                // D high -> T low (Dark)
                let t_r = physics::density_to_transmission(final_densities[0]);
                let t_g = physics::density_to_transmission(final_densities[1]);
                let t_b = physics::density_to_transmission(final_densities[2]);
                (
                    physics::linear_to_srgb(t_r.clamp(0.0, 1.0)),
                    physics::linear_to_srgb(t_g.clamp(0.0, 1.0)),
                    physics::linear_to_srgb(t_b.clamp(0.0, 1.0)),
                )
            },
            OutputMode::Positive => {
                // Scanned Positive
                // We assume Density maps linearly to Value (Scan).
                // D_min maps to White (1.0), D_max maps to Black (0.0).
                // Or simply: Value = 1.0 - (D - D_min) / Range.
                // Wait, if D is high (exposed), it should be Bright in Positive.
                // Scene Bright -> Neg High D -> Pos Bright.
                // So Value proportional to D.
                // But we need to normalize.
                // Let's use the film's D_min/D_max to normalize.
                // We'll just assume a standard range [0.0, 3.0] or use the input D.
                
                // Simple Scan: D / 3.0?
                // Let's normalize based on typical D_max ~3.0.
                let max_d = 3.0;
                let norm = |d: f32| (d / max_d).clamp(0.0, 1.0);
                
                (
                    physics::linear_to_srgb(norm(final_densities[0])),
                    physics::linear_to_srgb(norm(final_densities[1])),
                    physics::linear_to_srgb(norm(final_densities[2])),
                )
            }
        };
        
        chunk[0] = (r_out * 255.0).round() as u8;
        chunk[1] = (g_out * 255.0).round() as u8;
        chunk[2] = (b_out * 255.0).round() as u8;
    });
    
    RgbImage::from_raw(width, height, pixels).unwrap()
}
