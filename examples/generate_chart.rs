use image::{ImageBuffer, Rgb, RgbImage};
use std::f32::consts::PI;

/// Generate a comprehensive test chart
/// Includes:
/// 1. Grayscale Step Wedge (Bottom)
/// 2. Hue Wheel (Center)
/// 3. Saturation Gradient (Top)
/// 4. Primary Colors (Red, Green, Blue, Cyan, Magenta, Yellow)
pub fn generate_test_chart(width: u32, height: u32) -> RgbImage {
    let mut image: RgbImage = ImageBuffer::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let u = x as f32 / width as f32;
        let v = y as f32 / height as f32;

        // Background: Neutral Gray
        let r;
        let g;
        let b;

        if v > 0.8 {
            // Bottom 20%: Grayscale Wedge (21 steps)
            let steps = 21.0;
            let step = (u * steps).floor() / (steps - 1.0);
            r = step;
            g = step;
            b = step;
        } else if v < 0.2 {
            // Top 20%: Primary Colors & Saturation
            if u < 0.166 {
                // Red
                r = 1.0;
                g = 0.0;
                b = 0.0;
            } else if u < 0.333 {
                // Green
                r = 0.0;
                g = 1.0;
                b = 0.0;
            } else if u < 0.5 {
                // Blue
                r = 0.0;
                g = 0.0;
                b = 1.0;
            } else if u < 0.666 {
                // Yellow
                r = 1.0;
                g = 1.0;
                b = 0.0;
            } else if u < 0.833 {
                // Cyan
                r = 0.0;
                g = 1.0;
                b = 1.0;
            } else {
                // Magenta
                r = 1.0;
                g = 0.0;
                b = 1.0;
            }
        } else {
            // Center: Hue Wheel / Saturation
            // Center of wheel
            let cx = 0.5;
            let cy = 0.5;
            let dx = u - cx;
            let dy = (v - cy) * (height as f32 / width as f32); // Correct aspect ratio
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx); // -PI to PI

            if dist < 0.25 {
                // Hue Wheel
                let hue = (angle + PI) / (2.0 * PI); // 0.0 to 1.0
                let sat = dist / 0.25;
                let val = 1.0;
                let (rh, gh, bh) = hsv_to_rgb(hue, sat, val);
                r = rh;
                g = gh;
                b = bh;
            } else if dist < 0.28 {
                // Ring border - Neutral
                r = 0.5;
                g = 0.5;
                b = 0.5;
            } else {
                // Outside: Gradient
                // Left to right: Red -> White
                // Top to Bottom: Black -> White
                // Just keep neutral gray background
                r = 0.18;
                g = 0.18;
                b = 0.18; // 18% Gray background
            }
        }

        // sRGB Encoding
        *pixel = Rgb([
            (srgb_encode(r) * 255.0).round() as u8,
            (srgb_encode(g) * 255.0).round() as u8,
            (srgb_encode(b) * 255.0).round() as u8,
        ]);
    }

    image
}

#[allow(dead_code)]
fn main() {
    println!(
        "This file is a module used by other examples. Run 'chart_diagnosis' example instead."
    );
}

fn srgb_encode(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
