//! RAW image format support using rawler library.
//!
//! This module provides RAW file decoding and demosaicing functionality.
//! Supports CR2, CR3, NEF, ARW, RAF, ORF, RW2, PEF, DNG and many other formats.

use image::{DynamicImage, Rgb, RgbImage};

/// Supported RAW file extensions
pub const RAW_EXTENSIONS: &[&str] = &[
    "cr2", "cr3", "crw", // Canon
    "nef", "nrw", // Nikon
    "arw", "srf", "sr2", // Sony
    "raf", // Fujifilm
    "orf", // Olympus
    "rw2", // Panasonic/Leica
    "pef", // Pentax/Ricoh
    "dng", // Adobe DNG
    "3fr", // Hasselblad
    "kdc", "dcs", "dcr", // Kodak
    "mef", // Mamiya
    "mrw", // Minolta
    "iiq", "mos", // Phase One / Leaf
    "srw", // Samsung
    "erf", // Epson
    "ari", // ARRI
];

/// Check if a file extension is a supported RAW format
pub fn is_raw_extension(ext: &str) -> bool {
    RAW_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

/// Decode a RAW file from path and return a DynamicImage
pub fn decode_raw_file(path: &std::path::Path) -> Result<DynamicImage, String> {
    let raw_image =
        rawler::decode_file(path).map_err(|e| format!("Failed to decode RAW file: {}", e))?;

    let width = raw_image.width;
    let height = raw_image.height;

    // Get CFA pattern for demosaicing from camera definition
    let cfa = &raw_image.camera.cfa;

    // Extract raw pixel data
    let raw_data: Vec<u16> = match raw_image.data {
        rawler::RawImageData::Integer(data) => data,
        rawler::RawImageData::Float(data) => {
            // Convert f32 to u16 (assuming normalized 0-1 range)
            data.iter().map(|&v| (v * 65535.0) as u16).collect()
        }
    };

    // Get black and white levels for normalization
    let black_level = raw_image
        .blacklevel
        .levels
        .iter()
        .map(|v| v.as_f32())
        .sum::<f32>()
        / raw_image.blacklevel.levels.len() as f32;
    let white_level = raw_image.whitelevel.as_vec().iter().sum::<f32>()
        / raw_image.whitelevel.as_vec().len() as f32;

    // Get white balance multipliers
    let wb_coeffs = raw_image.wb_coeffs;

    // Demosaic using bilinear interpolation
    let rgb_image = demosaic_bilinear(
        &raw_data,
        width,
        height,
        cfa,
        black_level,
        white_level,
        &wb_coeffs,
    );

    Ok(DynamicImage::ImageRgb8(rgb_image))
}

/// Bilinear demosaicing algorithm
fn demosaic_bilinear(
    raw: &[u16],
    width: usize,
    height: usize,
    cfa: &rawler::CFA,
    black_level: f32,
    white_level: f32,
    wb_coeffs: &[f32; 4],
) -> RgbImage {
    let mut img = RgbImage::new(width as u32, height as u32);
    let range = white_level - black_level;

    // Normalize white balance coefficients (use green as reference)
    let wb_g = (wb_coeffs[1] + wb_coeffs[3]) / 2.0;
    let wb_r = if wb_g > 0.0 { wb_coeffs[0] / wb_g } else { 1.0 };
    let wb_b = if wb_g > 0.0 { wb_coeffs[2] / wb_g } else { 1.0 };

    for y in 0..height {
        for x in 0..width {
            let (r, g, b) = interpolate_pixel(
                raw,
                width,
                height,
                x,
                y,
                cfa,
                black_level,
                range,
                wb_r,
                wb_b,
            );

            // Apply gamma correction (simple sRGB approximation)
            let r_gamma = gamma_correct(r);
            let g_gamma = gamma_correct(g);
            let b_gamma = gamma_correct(b);

            img.put_pixel(
                x as u32,
                y as u32,
                Rgb([
                    (r_gamma * 255.0).clamp(0.0, 255.0) as u8,
                    (g_gamma * 255.0).clamp(0.0, 255.0) as u8,
                    (b_gamma * 255.0).clamp(0.0, 255.0) as u8,
                ]),
            );
        }
    }

    img
}

/// Interpolate a single pixel using bilinear interpolation
#[allow(clippy::too_many_arguments)]
fn interpolate_pixel(
    raw: &[u16],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    cfa: &rawler::CFA,
    black_level: f32,
    range: f32,
    wb_r: f32,
    wb_b: f32,
) -> (f32, f32, f32) {
    // CFA color indices: 0=Red, 1=Green, 2=Blue
    const CFA_RED: usize = 0;
    const CFA_GREEN: usize = 1;
    const CFA_BLUE: usize = 2;

    let color = cfa.color_at(x, y);

    // Get the raw value at this position, normalized
    let get_normalized = |px: usize, py: usize| -> f32 {
        if px < width && py < height {
            let val = raw[py * width + px] as f32;
            ((val - black_level) / range).clamp(0.0, 1.0)
        } else {
            0.0
        }
    };

    // Safe coordinate access with boundary clamping
    let clamp_x = |v: i32| -> usize { (v.max(0) as usize).min(width - 1) };
    let clamp_y = |v: i32| -> usize { (v.max(0) as usize).min(height - 1) };

    let xi = x as i32;
    let yi = y as i32;

    match color {
        CFA_RED => {
            let r = get_normalized(x, y) * wb_r;
            // Green: average of 4 neighbors
            let g = (get_normalized(clamp_x(xi - 1), y)
                + get_normalized(clamp_x(xi + 1), y)
                + get_normalized(x, clamp_y(yi - 1))
                + get_normalized(x, clamp_y(yi + 1)))
                / 4.0;
            // Blue: average of 4 diagonal neighbors
            let b = (get_normalized(clamp_x(xi - 1), clamp_y(yi - 1))
                + get_normalized(clamp_x(xi + 1), clamp_y(yi - 1))
                + get_normalized(clamp_x(xi - 1), clamp_y(yi + 1))
                + get_normalized(clamp_x(xi + 1), clamp_y(yi + 1)))
                / 4.0
                * wb_b;
            (r, g, b)
        }
        CFA_GREEN => {
            let g = get_normalized(x, y);
            // Determine if this is a green in red row or blue row
            let neighbor_color = cfa.color_at(clamp_x(xi - 1), y);
            if neighbor_color == CFA_RED {
                // Green in red row: R left/right, B top/bottom
                let r = (get_normalized(clamp_x(xi - 1), y) + get_normalized(clamp_x(xi + 1), y))
                    / 2.0
                    * wb_r;
                let b = (get_normalized(x, clamp_y(yi - 1)) + get_normalized(x, clamp_y(yi + 1)))
                    / 2.0
                    * wb_b;
                (r, g, b)
            } else {
                // Green in blue row: B left/right, R top/bottom
                let b = (get_normalized(clamp_x(xi - 1), y) + get_normalized(clamp_x(xi + 1), y))
                    / 2.0
                    * wb_b;
                let r = (get_normalized(x, clamp_y(yi - 1)) + get_normalized(x, clamp_y(yi + 1)))
                    / 2.0
                    * wb_r;
                (r, g, b)
            }
        }
        CFA_BLUE => {
            let b = get_normalized(x, y) * wb_b;
            // Green: average of 4 neighbors
            let g = (get_normalized(clamp_x(xi - 1), y)
                + get_normalized(clamp_x(xi + 1), y)
                + get_normalized(x, clamp_y(yi - 1))
                + get_normalized(x, clamp_y(yi + 1)))
                / 4.0;
            // Red: average of 4 diagonal neighbors
            let r = (get_normalized(clamp_x(xi - 1), clamp_y(yi - 1))
                + get_normalized(clamp_x(xi + 1), clamp_y(yi - 1))
                + get_normalized(clamp_x(xi - 1), clamp_y(yi + 1))
                + get_normalized(clamp_x(xi + 1), clamp_y(yi + 1)))
                / 4.0
                * wb_r;
            (r, g, b)
        }
        _ => {
            // For other patterns (rare), just return grayscale
            let v = get_normalized(x, y);
            (v, v, v)
        }
    }
}

/// Simple sRGB gamma correction
fn gamma_correct(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}
