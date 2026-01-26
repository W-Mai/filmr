use image::{ImageBuffer, Rgb};
use rayon::prelude::*;

/// Helper to apply Box Blur (Approximates Gaussian when repeated)
/// Uses a sliding window (Integral Image / Moving Average) approach for O(1) per pixel independent of radius.
pub fn apply_box_blur(image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, radius: u32) {
    let width = image.width();
    let height = image.height();
    let r = radius as i32;

    // We process each channel independently
    // Horizontal Pass
    let mut temp = image.clone(); // Need temp buffer for separable pass

    // Use Rayon for parallel processing of rows
    // Horizontal Pass using par_chunks_mut on raw buffer
    let raw_buffer = temp.as_mut();
    raw_buffer
        .par_chunks_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, row_slice)| {
            let _y = y as u32;
            // Sliding window sum
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            // Initialize window
            // Left padding (pixel at x=0)
            let first_r = row_slice[0];
            let first_g = row_slice[1];
            let first_b = row_slice[2];

            for _ in 0..=r {
                sum_r += first_r;
                sum_g += first_g;
                sum_b += first_b;
            }
            // Right side of window
            for x in 1..=r {
                let idx = (x.min((width - 1).try_into().unwrap()) as usize) * 3;
                sum_r += row_slice[idx];
                sum_g += row_slice[idx + 1];
                sum_b += row_slice[idx + 2];
            }

            for x in 0..width {
                let count = (2 * r + 1) as f32;
                let current_idx = (x as usize) * 3;

                // Write to `temp` (row_slice)
                row_slice[current_idx] = sum_r / count;
                row_slice[current_idx + 1] = sum_g / count;
                row_slice[current_idx + 2] = sum_b / count;

                // Slide window: subtract left-out, add incoming-in
                let left_x = (x as i32 - r).clamp(0, (width - 1) as i32) as u32;
                let right_x = (x as i32 + r + 1).clamp(0, (width - 1) as i32) as u32;

                // Read from source `image`
                let p_out = image.get_pixel(left_x, _y).0;
                let p_in = image.get_pixel(right_x, _y).0;

                sum_r += p_in[0] - p_out[0];
                sum_g += p_in[1] - p_out[1];
                sum_b += p_in[2] - p_out[2];
            }
        });

    // Vertical Pass: Transpose -> Horizontal Blur -> Transpose
    // 1. Transpose temp -> image (swapped dimensions)
    let mut transposed: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(height, width);
    // Use raw buffer parallel iteration for transpose
    // temp is width x height, transposed is height x width
    let transposed_raw = transposed.as_mut();

    // Parallelize by rows of the OUTPUT (transposed) image
    transposed_raw
        .par_chunks_mut((height * 3) as usize)
        .enumerate()
        .for_each(|(y_t, row_t)| {
            for x_t in 0..height {
                let src_pixel = temp.get_pixel(y_t as u32, x_t).0;

                let idx = (x_t as usize) * 3;
                row_t[idx] = src_pixel[0];
                row_t[idx + 1] = src_pixel[1];
                row_t[idx + 2] = src_pixel[2];
            }
        });

    // 2. Horizontal Blur on Transposed
    // This is essentially the vertical blur of the original image
    let mut transposed_blurred = transposed.clone();

    let raw_blurred_transposed = transposed_blurred.as_mut();
    // Iterating over rows of transposed image
    raw_blurred_transposed
        .par_chunks_mut((height * 3) as usize)
        .enumerate()
        .for_each(|(_y, row_slice)| {
            // y is row index in transposed image
            // width_t is width of transposed image (= original height)
            let width_t = height;

            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            // Initialize window
            let first_r = row_slice[0];
            let first_g = row_slice[1];
            let first_b = row_slice[2];

            for _ in 0..=r {
                sum_r += first_r;
                sum_g += first_g;
                sum_b += first_b;
            }
            for x in 1..=r {
                let idx = (x.min((width_t - 1).try_into().unwrap()) as usize) * 3;
                sum_r += row_slice[idx];
                sum_g += row_slice[idx + 1];
                sum_b += row_slice[idx + 2];
            }

            for x in 0..width_t {
                let count = (2 * r + 1) as f32;
                let current_idx = (x as usize) * 3;

                row_slice[current_idx] = sum_r / count;
                row_slice[current_idx + 1] = sum_g / count;
                row_slice[current_idx + 2] = sum_b / count;

                let left_idx = (x as i32 - r).clamp(0, (width_t - 1) as i32) as u32;
                let right_idx = (x as i32 + r + 1).clamp(0, (width_t - 1) as i32) as u32;

                let p_out = transposed.get_pixel(left_idx, _y as u32).0;
                let p_in = transposed.get_pixel(right_idx, _y as u32).0;

                sum_r += p_in[0] - p_out[0];
                sum_g += p_in[1] - p_out[1];
                sum_b += p_in[2] - p_out[2];
            }
        });

    // 3. Transpose back: transposed_blurred -> image
    let raw_buffer = image.as_mut();
    // image is width x height
    raw_buffer
        .par_chunks_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, row)| {
            // y is row index in image (0..height)
            for x in 0..width {
                // Target(x, y) = Source(y, x)
                // Source is transposed_blurred
                let src_pixel = transposed_blurred.get_pixel(y as u32, x).0;

                let idx = (x as usize) * 3;
                row[idx] = src_pixel[0];
                row[idx + 1] = src_pixel[1];
                row[idx + 2] = src_pixel[2];
            }
        });
}

/// Helper to apply Gaussian blur (Approx) using 3 Box Blurs
pub fn apply_gaussian_blur(image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, sigma: f32) {
    // 3 passes of Box Blur approximates Gaussian Blur very well (Central Limit Theorem)
    // w = sqrt(12 * sigma^2 / n + 1)
    // radius = (w - 1) / 2
    let n = 3.0;
    let w = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let radius = ((w - 1.0) / 2.0).floor() as u32;
    let radius = radius.max(1);

    for _ in 0..3 {
        apply_box_blur(image, radius);
    }
}
