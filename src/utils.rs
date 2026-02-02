use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use wide::f32x4;

/// Helper to apply Gaussian blur (Approx) using 3 Box Blurs
/// Optimized to minimize allocations and use SIMD
pub fn apply_gaussian_blur(image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, sigma: f32) {
    if sigma <= 0.0 {
        return;
    }

    let width = image.width();
    let height = image.height();

    // w = sqrt(12 * sigma^2 / n + 1)
    // radius = (w - 1) / 2
    let n = 3.0;
    let w = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let radius = ((w - 1.0) / 2.0).floor() as u32;
    let radius = radius.max(1);

    // Single auxiliary buffer allocation
    let mut backbuffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

    for _ in 0..3 {
        // Horizontal: Image -> Backbuffer
        horizontal_blur_pass(image, &mut backbuffer, radius);
        // Vertical: Backbuffer -> Image
        vertical_blur_pass(&backbuffer, image, radius);
    }
}

fn horizontal_blur_pass(
    src: &ImageBuffer<Rgb<f32>, Vec<f32>>,
    dst: &mut ImageBuffer<Rgb<f32>, Vec<f32>>,
    radius: u32,
) {
    let width = src.width();
    let r = radius as i32;
    let weight = 1.0 / (2.0 * radius as f32 + 1.0);
    let weight_vec = f32x4::splat(weight);

    // Iterate over rows in parallel
    // We use chunks of the raw buffer to write to dst safely
    dst.par_chunks_mut((width * 3) as usize)
        .enumerate()
        .for_each(|(y, dst_row)| {
            let y = y as u32;
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            // Initial window [-r, r] centered at 0
            // Left side [-r, -1] -> clamped to 0
            let p0 = src.get_pixel(0, y).0;
            sum_r += p0[0] * (r as f32);
            sum_g += p0[1] * (r as f32);
            sum_b += p0[2] * (r as f32);

            // Right side [0, r]
            for x in 0..=r {
                let px = src.get_pixel(x.min((width - 1) as i32) as u32, y).0;
                sum_r += px[0];
                sum_g += px[1];
                sum_b += px[2];
            }

            let mut sum_vec = f32x4::from([sum_r, sum_g, sum_b, 0.0]);

            for x in 0..width {
                let avg = sum_vec * weight_vec;
                let avg_arr: [f32; 4] = avg.into();

                let pixel_idx = (x as usize) * 3;
                dst_row[pixel_idx] = avg_arr[0];
                dst_row[pixel_idx + 1] = avg_arr[1];
                dst_row[pixel_idx + 2] = avg_arr[2];

                // Update sliding window
                let out_x = (x as i32 - r).max(0) as u32;
                let in_x = (x as i32 + r + 1).min((width - 1) as i32) as u32;

                let p_out = src.get_pixel(out_x, y).0;
                let p_in = src.get_pixel(in_x, y).0;

                let v_out = f32x4::from([p_out[0], p_out[1], p_out[2], 0.0]);
                let v_in = f32x4::from([p_in[0], p_in[1], p_in[2], 0.0]);

                sum_vec += v_in - v_out;
            }
        });
}

fn vertical_blur_pass(
    src: &ImageBuffer<Rgb<f32>, Vec<f32>>,
    dst: &mut ImageBuffer<Rgb<f32>, Vec<f32>>,
    radius: u32,
) {
    let width = src.width();
    let height = src.height();
    let r = radius as i32;
    let weight = 1.0 / (2.0 * radius as f32 + 1.0);
    let weight_vec = f32x4::splat(weight);

    // Send raw pointer address to threads as usize (safe to share)
    let dst_ptr_addr = dst.as_mut_ptr() as usize;

    // Parallelize by columns
    (0..width).into_par_iter().for_each(|x| {
        // Reconstruct pointer locally
        let dst_ptr = dst_ptr_addr as *mut f32;
        // Safety: We only write to indices corresponding to column x.
        // Different threads handle different x, so indices are disjoint.
        let dst_slice =
            unsafe { std::slice::from_raw_parts_mut(dst_ptr, (width * height * 3) as usize) };

        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;

        let p0 = src.get_pixel(x, 0).0;
        sum_r += p0[0] * (r as f32);
        sum_g += p0[1] * (r as f32);
        sum_b += p0[2] * (r as f32);

        for y in 0..=r {
            let py = src.get_pixel(x, y.min((height - 1) as i32) as u32).0;
            sum_r += py[0];
            sum_g += py[1];
            sum_b += py[2];
        }

        let mut sum_vec = f32x4::from([sum_r, sum_g, sum_b, 0.0]);

        for y in 0..height {
            let avg = sum_vec * weight_vec;
            let avg_arr: [f32; 4] = avg.into();

            let idx = ((y as usize) * (width as usize) + (x as usize)) * 3;
            dst_slice[idx] = avg_arr[0];
            dst_slice[idx + 1] = avg_arr[1];
            dst_slice[idx + 2] = avg_arr[2];

            let out_y = (y as i32 - r).max(0) as u32;
            let in_y = (y as i32 + r + 1).min((height - 1) as i32) as u32;

            let p_out = src.get_pixel(x, out_y).0;
            let p_in = src.get_pixel(x, in_y).0;

            let v_out = f32x4::from([p_out[0], p_out[1], p_out[2], 0.0]);
            let v_in = f32x4::from([p_in[0], p_in[1], p_in[2], 0.0]);

            sum_vec += v_in - v_out;
        }
    });
}
