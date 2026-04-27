use crate::film::{FilmStock, FilmType};
use crate::physics;
use crate::processor::{OutputMode, SimulationConfig, WhiteBalanceMode};
use crate::utils;
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use tracing::{debug, info, instrument};
use wide::f32x4;

/// Context shared across all pipeline stages.
/// Contains read-only references to film data and configuration.
pub struct PipelineContext<'a> {
    pub film: &'a FilmStock,
    pub config: &'a SimulationConfig,
    pub depth_map: Option<&'a crate::depth::DepthMap>,
}

/// A stage in the image processing pipeline.
/// Modifies the image buffer in place.
pub trait PipelineStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext);
}

/// # Linearize Stage (Initializer)
///
/// Converts sRGB input image to Linear RGB f32 format.
/// Uses a Look-Up Table (LUT) for performance optimization.
#[instrument(skip(input))]
pub fn create_linear_image(input: &RgbImage) -> ImageBuffer<Rgb<f32>, Vec<f32>> {
    debug!("Converting input image to linear space");
    let width = input.width();
    let height = input.height();
    let mut linear_image: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);

    // Precompute sRGB to Linear LUT for 8-bit input
    // This provides a significant speedup (instruction level parallelism via LUT)
    let lut: Vec<f32> = (0..=255)
        .map(|i| physics::srgb_to_linear(i as f32 / 255.0))
        .collect();

    linear_image
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(i, pixel)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            let in_pixel = input.get_pixel(x, y);

            // Use LUT for fast conversion
            pixel[0] = lut[in_pixel[0] as usize];
            pixel[1] = lut[in_pixel[1] as usize];
            pixel[2] = lut[in_pixel[2] as usize];
        });
    linear_image
}

/// # Micro Motion Stage
///
/// Simulates hand-held camera shake via 3D rotation in linear light space.
/// Uses multi-frequency sinusoidal tremor (8-12Hz + harmonics) with dwell weighting.
/// Applied before DevelopStage so bright areas naturally produce stronger trails.
pub struct MicroMotionStage;

impl PipelineStage for MicroMotionStage {
    #[instrument(skip(self, image, _context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, _context: &PipelineContext) {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let amount = _context.config.motion_blur_amount;

        let amplitude = (width as f32 / 4000.0) * 15.0 * amount;
        if amplitude < 0.3 {
            return;
        }

        let seed = _context.config.motion_blur_seed;
        let traj = crate::shake::ShakeTrajectory::generate(amplitude, 64, seed);

        info!(
            "Applying micro motion ({} samples, amp {:.1}px)",
            traj.points.len(),
            amplitude
        );

        // For each trajectory point, compute a shifted version and accumulate
        // In linear light space: bright pixels contribute more (physically correct)
        let src: Vec<f32> = image.as_flat_samples().as_slice().to_vec();

        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let dst_x = (i % width) as f32;
            let dst_y = (i / width) as f32;
            let (mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32);

            for &(tx, ty, weight) in &traj.points {
                // Source pixel = dst - trajectory offset (nearest neighbor = sharp overlay)
                let sx = (dst_x - tx).round() as i32;
                let sy = (dst_y - ty).round() as i32;

                if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                    let si = (sy as usize * width + sx as usize) * 3;
                    r += weight * src[si];
                    g += weight * src[si + 1];
                    b += weight * src[si + 2];
                } else {
                    r += weight * pixel[0];
                    g += weight * pixel[1];
                    b += weight * pixel[2];
                }
            }

            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
        });
    }
}

/// # Object Motion Stage
///
/// Simulates micro-movement of scene objects (people walking, leaves swaying).
/// Uses depth map to modulate displacement: near objects move more (1/depth).
/// Applied in linear light space so bright areas produce stronger trails.
pub struct ObjectMotionStage;

impl PipelineStage for ObjectMotionStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let depth_map = match context.depth_map {
            Some(dm) => dm,
            None => return,
        };

        let amount = context.config.object_motion_amount;
        if amount <= 0.0 {
            return;
        }

        let width = image.width() as usize;
        let height = image.height() as usize;

        info!("Applying object motion (amount={:.2})", amount);

        // Step 1: Label connected regions by depth similarity
        // Two pixels are connected if their depth difference < threshold
        let depth_threshold = 0.08f32; // ~8% depth difference = same object
        let mut labels = vec![0u32; width * height];
        let mut parent: Vec<u32> = vec![0]; // index 0 unused
        let mut next_label = 1u32;

        let find = |parent: &mut Vec<u32>, mut x: u32| -> u32 {
            while parent[x as usize] != x {
                parent[x as usize] = parent[parent[x as usize] as usize];
                x = parent[x as usize];
            }
            x
        };

        // First pass
        let dm_w = depth_map.width as f32;
        let dm_h = depth_map.height as f32;
        let w_f = width as f32;
        let h_f = height as f32;
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let dx = (x as f32 / w_f * dm_w) as u32;
                let dy = (y as f32 / h_f * dm_h) as u32;
                let d = depth_map.get(dx, dy);

                let left = if x > 0 {
                    let ldx = ((x - 1) as f32 / w_f * dm_w) as u32;
                    let ld = depth_map.get(ldx, dy);
                    if (d - ld).abs() < depth_threshold {
                        Some(labels[idx - 1])
                    } else {
                        None
                    }
                } else {
                    None
                };
                let up = if y > 0 {
                    let udy = ((y - 1) as f32 / h_f * dm_h) as u32;
                    let ud = depth_map.get(dx, udy);
                    if (d - ud).abs() < depth_threshold {
                        Some(labels[idx - width])
                    } else {
                        None
                    }
                } else {
                    None
                };

                match (left, up) {
                    (Some(l), Some(u)) => {
                        let rl = find(&mut parent, l);
                        let ru = find(&mut parent, u);
                        labels[idx] = rl.min(ru);
                        if rl != ru {
                            parent[rl.max(ru) as usize] = rl.min(ru);
                        }
                    }
                    (Some(l), None) => labels[idx] = find(&mut parent, l),
                    (None, Some(u)) => labels[idx] = find(&mut parent, u),
                    (None, None) => {
                        labels[idx] = next_label;
                        if parent.len() <= next_label as usize {
                            parent.resize(next_label as usize + 1, 0);
                        }
                        parent[next_label as usize] = next_label;
                        next_label += 1;
                    }
                }
            }
        }

        // Second pass: flatten
        for label in labels.iter_mut() {
            *label = find(&mut parent, *label);
        }

        // Step 2: Assign random motion direction per region
        let max_label = labels.iter().cloned().max().unwrap_or(0) as usize;
        let seed = context.config.motion_blur_seed;
        let mut rng = {
            use rand::SeedableRng;
            rand::rngs::StdRng::seed_from_u64(seed.wrapping_add(12345))
        };
        use rand::Rng;
        let mut region_dx = vec![0.0f32; max_label + 1];
        let mut region_dy = vec![0.0f32; max_label + 1];
        for i in 0..=max_label {
            let angle: f32 = rng.gen::<f32>() * std::f32::consts::TAU;
            region_dx[i] = angle.cos();
            region_dy[i] = angle.sin();
        }

        // Step 3: Apply depth-modulated multi-sample blur per region
        let max_disp = (width as f32 / 4000.0) * 4.0 * amount;
        let src: Vec<f32> = image.as_flat_samples().as_slice().to_vec();

        let dm_w = depth_map.width;
        let dm_h = depth_map.height;

        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let px = i % width;
            let py = i / width;

            let dmx = (px as f32 / width as f32 * dm_w as f32) as u32;
            let dmy = (py as f32 / height as f32 * dm_h as f32) as u32;
            let depth = depth_map.get(dmx, dmy);
            let depth_factor = 1.0 / (depth * 4.0 + 0.25);

            let label = labels[i] as usize;
            let dx = if label < region_dx.len() {
                region_dx[label]
            } else {
                0.0
            };
            let dy = if label < region_dy.len() {
                region_dy[label]
            } else {
                0.0
            };

            let disp = max_disp * depth_factor;

            let n_samples = 8;
            let (mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32);
            let mut w_sum = 0.0f32;
            for s in 0..n_samples {
                let t = s as f32 / (n_samples - 1).max(1) as f32 - 0.5;
                let sx = (px as f32 + dx * disp * t).round() as i32;
                let sy = (py as f32 + dy * disp * t).round() as i32;
                if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                    let si = (sy as usize * width + sx as usize) * 3;
                    r += src[si];
                    g += src[si + 1];
                    b += src[si + 2];
                    w_sum += 1.0;
                }
            }
            if w_sum > 0.0 {
                pixel[0] = r / w_sum;
                pixel[1] = g / w_sum;
                pixel[2] = b / w_sum;
            }
        });
    }
}

/// # Vignetting Stage
///
/// Simulates lens light falloff using cos⁴(θ) model.
/// Applied in exposure space (before develop) so it affects H-D curve naturally.
pub struct VignettingStage;

impl PipelineStage for VignettingStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let strength = context.film.vignette_strength;
        if strength <= 0.0 {
            return;
        }
        info!("Applying vignetting (strength: {:.2})", strength);
        let w = image.width() as f32;
        let h = image.height() as f32;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let r_max = (cx * cx + cy * cy).sqrt();
        // Equivalent focal length: assume 50mm on 36mm frame → f/diagonal ratio
        let f_equiv = r_max * 1.2; // ~50mm equivalent

        let img_w = image.width();
        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i as u32 % img_w) as f32 + 0.5;
            let y = (i as u32 / img_w) as f32 + 0.5;
            let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            let theta = (r / f_equiv).atan();
            let falloff = theta.cos().powi(4);
            // Blend between no vignetting (1.0) and full cos⁴
            let factor = 1.0 - strength * (1.0 - falloff);
            pixel[0] *= factor;
            pixel[1] *= factor;
            pixel[2] *= factor;
        });
    }
}

/// # Halation Stage
///
/// Simulates light reflecting off the film base back into the emulsion.
/// Creates a reddish-orange glow around highlights.
pub struct HalationStage;

impl PipelineStage for HalationStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        if film.halation_strength <= 0.0 {
            debug!("Halation disabled (strength <= 0)");
            return;
        }
        info!("Applying Halation effect");

        let width = image.width();
        let threshold = film.halation_threshold;
        let mut halation_map = image.clone();

        // Apply threshold
        halation_map.par_chunks_mut(3).for_each(|p| {
            let lum = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
            if lum < threshold {
                p[0] = 0.0;
                p[1] = 0.0;
                p[2] = 0.0;
            } else {
                p[0] = (p[0] - threshold).max(0.0);
                p[1] = (p[1] - threshold).max(0.0);
                p[2] = (p[2] - threshold).max(0.0);
            }
        });

        let blur_sigma = width as f32 * film.halation_sigma;
        utils::apply_gaussian_blur(&mut halation_map, blur_sigma);

        let tint = film.halation_tint;
        let strength = film.halation_strength;

        let factor_r = tint[0] * strength;
        let factor_g = tint[1] * strength;
        let factor_b = tint[2] * strength;

        // SIMD constants for RGBRGB... pattern
        let v0 = f32x4::from([factor_r, factor_g, factor_b, factor_r]);
        let v1 = f32x4::from([factor_g, factor_b, factor_r, factor_g]);
        let v2 = f32x4::from([factor_b, factor_r, factor_g, factor_b]);

        // Process 4 pixels (12 floats) at a time to align with SIMD lanes
        image
            .par_chunks_mut(12)
            .zip(halation_map.par_chunks(12))
            .for_each(|(dest, src)| {
                if dest.len() == 12 {
                    // SIMD Path
                    let d0_arr: [f32; 4] = dest[0..4].try_into().unwrap();
                    let s0_arr: [f32; 4] = src[0..4].try_into().unwrap();
                    let d0 = f32x4::from(d0_arr);
                    let s0 = f32x4::from(s0_arr);
                    let r0 = d0 + s0 * v0;
                    dest[0..4].copy_from_slice(&<[f32; 4]>::from(r0));

                    let d1_arr: [f32; 4] = dest[4..8].try_into().unwrap();
                    let s1_arr: [f32; 4] = src[4..8].try_into().unwrap();
                    let d1 = f32x4::from(d1_arr);
                    let s1 = f32x4::from(s1_arr);
                    let r1 = d1 + s1 * v1;
                    dest[4..8].copy_from_slice(&<[f32; 4]>::from(r1));

                    let d2_arr: [f32; 4] = dest[8..12].try_into().unwrap();
                    let s2_arr: [f32; 4] = src[8..12].try_into().unwrap();
                    let d2 = f32x4::from(d2_arr);
                    let s2 = f32x4::from(s2_arr);
                    let r2 = d2 + s2 * v2;
                    dest[8..12].copy_from_slice(&<[f32; 4]>::from(r2));
                } else {
                    // Scalar Fallback
                    for (d, s) in dest.chunks_mut(3).zip(src.chunks(3)) {
                        d[0] += s[0] * factor_r;
                        d[1] += s[1] * factor_g;
                        d[2] += s[2] * factor_b;
                    }
                }
            });
    }
}

/// # Depth of Field Stage
///
/// Simulates lens bokeh using depth map. Mipmap-based variable radius blur:
/// precompute multiple Gaussian blur levels, then per-pixel interpolate
/// between levels based on Circle of Confusion (CoC).
pub struct DepthOfFieldStage;

impl PipelineStage for DepthOfFieldStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let amount = context.config.dof_amount;
        if amount <= 0.0 {
            return;
        }
        let depth_map = match context.depth_map {
            Some(dm) => dm,
            None => return,
        };

        let focus = context.config.dof_focus;
        let width = image.width() as usize;
        let height = image.height() as usize;
        let max_radius = (width as f32 / 200.0) * amount;

        info!(
            "Applying depth of field (amount={:.2}, focus={:.2}, max_r={:.1}px)",
            amount, focus, max_radius
        );

        // Build mipmap: 6 levels of increasing Gaussian blur
        let n_levels = 6usize;
        let src: Vec<f32> = image.as_flat_samples().as_slice().to_vec();
        let mut levels: Vec<Vec<f32>> = vec![src.clone()];

        let mut prev = src;
        for level in 1..n_levels {
            let sigma = (1 << level) as f32 * 0.5;
            let kernel_r = (sigma * 2.5).ceil() as usize;
            let kernel: Vec<f32> = (0..=kernel_r)
                .map(|i| (-0.5 * (i as f32 / sigma).powi(2)).exp())
                .collect();
            let k_sum: f32 = kernel[0] + 2.0 * kernel[1..].iter().sum::<f32>();

            // Horizontal pass
            let mut temp = vec![0.0f32; width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    let (mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32);
                    for (ki, &w) in kernel.iter().enumerate() {
                        for &dx in &[-(ki as i32), ki as i32] {
                            if ki == 0 && dx != 0 {
                                continue;
                            }
                            let sx = (x as i32 + dx).clamp(0, width as i32 - 1) as usize;
                            let si = (y * width + sx) * 3;
                            r += prev[si] * w;
                            g += prev[si + 1] * w;
                            b += prev[si + 2] * w;
                        }
                    }
                    let di = (y * width + x) * 3;
                    temp[di] = r / k_sum;
                    temp[di + 1] = g / k_sum;
                    temp[di + 2] = b / k_sum;
                }
            }

            // Vertical pass
            let mut blurred = vec![0.0f32; width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    let (mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32);
                    for (ki, &w) in kernel.iter().enumerate() {
                        for &dy in &[-(ki as i32), ki as i32] {
                            if ki == 0 && dy != 0 {
                                continue;
                            }
                            let sy = (y as i32 + dy).clamp(0, height as i32 - 1) as usize;
                            let si = (sy * width + x) * 3;
                            r += temp[si] * w;
                            g += temp[si + 1] * w;
                            b += temp[si + 2] * w;
                        }
                    }
                    let di = (y * width + x) * 3;
                    blurred[di] = r / k_sum;
                    blurred[di + 1] = g / k_sum;
                    blurred[di + 2] = b / k_sum;
                }
            }

            levels.push(blurred.clone());
            prev = blurred;
        }

        // Per-pixel: compute CoC, interpolate between mipmap levels
        let dm_w = depth_map.width;
        let dm_h = depth_map.height;
        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let px = i % width;
            let py = i / width;
            let dx = (px as f32 / width as f32 * dm_w as f32) as u32;
            let dy = (py as f32 / height as f32 * dm_h as f32) as u32;
            let depth = depth_map.get(dx, dy);
            let coc = max_radius * (depth - focus).abs();

            // Map CoC to mipmap level (log2 scale)
            let level_f = if coc > 1.0 { coc.log2() } else { 0.0 };
            let level_lo = (level_f as usize).min(n_levels - 2);
            let level_hi = level_lo + 1;
            let frac = (level_f - level_lo as f32).clamp(0.0, 1.0);

            let si = i * 3;
            pixel[0] = levels[level_lo][si] * (1.0 - frac) + levels[level_hi][si] * frac;
            pixel[1] = levels[level_lo][si + 1] * (1.0 - frac) + levels[level_hi][si + 1] * frac;
            pixel[2] = levels[level_lo][si + 2] * (1.0 - frac) + levels[level_hi][si + 2] * frac;
        });

        // Petzval swirly bokeh: tangential stretch on out-of-focus areas
        let swirl = context.config.dof_swirl;
        if swirl > 0.0 {
            let cx = width as f32 / 2.0;
            let cy = height as f32 / 2.0;
            let src: Vec<f32> = image.as_flat_samples().as_slice().to_vec();
            let n_samples = 16;

            image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
                let px = (i % width) as f32;
                let py = (i / width) as f32;
                let ddx = (px / width as f32 * dm_w as f32) as u32;
                let ddy = (py / height as f32 * dm_h as f32) as u32;
                let depth = depth_map.get(ddx, ddy);
                let defocus = (depth - focus).abs();

                if defocus < 0.05 {
                    return; // In focus, skip
                }

                // Tangent direction (perpendicular to radial)
                let dx = px - cx;
                let dy = py - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let tx = -dy / dist;
                let ty = dx / dist;

                // Stretch amount: proportional to defocus × distance from center × swirl
                let stretch = swirl * defocus * (dist / cx.max(cy)) * max_radius * 2.0;

                let (mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32);
                for s in 0..n_samples {
                    let t = s as f32 / (n_samples - 1) as f32 - 0.5;
                    let sx = (px + tx * stretch * t).round() as i32;
                    let sy = (py + ty * stretch * t).round() as i32;
                    if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                        let si = (sy as usize * width + sx as usize) * 3;
                        r += src[si];
                        g += src[si + 1];
                        b += src[si + 2];
                    } else {
                        let si = i * 3;
                        r += src[si];
                        g += src[si + 1];
                        b += src[si + 2];
                    }
                }
                let inv = 1.0 / n_samples as f32;
                pixel[0] = r * inv;
                pixel[1] = g * inv;
                pixel[2] = b * inv;
            });
        }
    }
}

/// # Rotational Blur Stage
///
/// Simulates camera rotation around the optical axis.
/// Center stays sharp, edges blur along tangent direction.
/// Blur amount increases with distance from center.
pub struct RotationalBlurStage;

impl PipelineStage for RotationalBlurStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let amount = context.config.rotational_blur_amount;
        if amount <= 0.0 {
            return;
        }

        let width = image.width() as usize;
        let height = image.height() as usize;
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        // Max rotation angle in radians (amount=1.0 → ~0.5°)
        let max_angle = amount * 0.03;
        let n_samples = 32;

        info!("Applying rotational blur (amount={:.2})", amount);

        let src: Vec<f32> = image.as_flat_samples().as_slice().to_vec();

        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let px = (i % width) as f32;
            let py = (i / width) as f32;
            let dx = px - cx;
            let dy = py - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            // Blur proportional to distance from center
            let angle_range = max_angle * (dist / cx.max(cy));

            let (mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32);
            for s in 0..n_samples {
                let t = s as f32 / (n_samples - 1).max(1) as f32 - 0.5;
                let a = angle_range * t;
                let cos_a = a.cos();
                let sin_a = a.sin();
                let sx = (cx + dx * cos_a - dy * sin_a).round() as i32;
                let sy = (cy + dx * sin_a + dy * cos_a).round() as i32;
                if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                    let si = (sy as usize * width + sx as usize) * 3;
                    r += src[si];
                    g += src[si + 1];
                    b += src[si + 2];
                } else {
                    let si = i * 3;
                    r += src[si];
                    g += src[si + 1];
                    b += src[si + 2];
                }
            }
            let inv = 1.0 / n_samples as f32;
            pixel[0] = r * inv;
            pixel[1] = g * inv;
            pixel[2] = b * inv;
        });
    }
}

/// # MTF (Modulation Transfer Function) Stage
///
/// Simulates optical softness based on the film's resolving power (lp/mm).
/// Applied before grain to simulate the physical blurring of the image.
pub struct MtfStage;

impl PipelineStage for MtfStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        let film = context.film;
        let width = image.width();
        let height = image.height();

        let pixels_per_mm = width as f32 / 36.0;
        let mtf_sigma = (0.5 / film.resolution_lp_mm) * pixels_per_mm;

        if mtf_sigma <= 0.5 {
            debug!("MTF blur skipped (sigma too small: {:.2})", mtf_sigma);
            return;
        }

        info!("Applying radial MTF blur (edge sigma: {:.2})", mtf_sigma);

        // Blur a copy at full MTF sigma
        let mut blurred = image.clone();
        utils::apply_gaussian_blur(&mut blurred, mtf_sigma);

        // Blend: center = sharp (original), edges = blurred
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let r_max = (cx * cx + cy * cy).sqrt();

        image
            .par_chunks_mut(3)
            .zip(blurred.par_chunks(3))
            .enumerate()
            .for_each(|(i, (orig, blur))| {
                let x = (i as u32 % width) as f32 + 0.5 - cx;
                let y = (i as u32 / width) as f32 + 0.5 - cy;
                let r = (x * x + y * y).sqrt() / r_max;
                // Quadratic falloff: center=0, corner=1
                let t = (r * r).clamp(0.0, 1.0);
                orig[0] = orig[0] * (1.0 - t) + blur[0] * t;
                orig[1] = orig[1] * (1.0 - t) + blur[1] * t;
                orig[2] = orig[2] * (1.0 - t) + blur[2] * t;
            });
    }
}

/// # Chromatic Aberration Stage
///
/// Simulates lateral chromatic aberration: R channel slightly magnified,
/// B channel slightly demagnified relative to G. Produces RGB fringing at edges.
pub struct ChromaticAberrationStage;

impl PipelineStage for ChromaticAberrationStage {
    #[instrument(skip(self, image, _context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, _context: &PipelineContext) {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;

        // Scale factors: R slightly larger, B slightly smaller
        let r_scale = 1.0015f32; // R magnified
        let b_scale = 0.9985f32; // B demagnified

        info!(
            "Applying chromatic aberration (R×{}, B×{})",
            r_scale, b_scale
        );

        let src: Vec<f32> = image.as_flat_samples().as_slice().to_vec();

        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let x = (i % width) as f32 + 0.5;
            let y = (i / width) as f32 + 0.5;
            let dx = x - cx;
            let dy = y - cy;

            // R channel: sample from slightly inward (magnified → inward lookup)
            let rx = cx + dx / r_scale;
            let ry = cy + dy / r_scale;
            pixel[0] = sample_channel(&src, width, height, rx, ry, 0);

            // G channel: unchanged
            // pixel[1] stays as is

            // B channel: sample from slightly outward (demagnified → outward lookup)
            let bx = cx + dx / b_scale;
            let by = cy + dy / b_scale;
            pixel[2] = sample_channel(&src, width, height, bx, by, 2);
        });
    }
}

/// Bilinear sample of a single channel from flat RGB buffer.
fn sample_channel(src: &[f32], w: usize, h: usize, x: f32, y: f32, ch: usize) -> f32 {
    let ix = (x - 0.5).floor() as i32;
    let iy = (y - 0.5).floor() as i32;
    if ix < 0 || ix >= w as i32 - 1 || iy < 0 || iy >= h as i32 - 1 {
        // Edge: clamp
        let sx = (x as i32).clamp(0, w as i32 - 1) as usize;
        let sy = (y as i32).clamp(0, h as i32 - 1) as usize;
        return src[(sy * w + sx) * 3 + ch];
    }
    let fx = x - 0.5 - ix as f32;
    let fy = y - 0.5 - iy as f32;
    let i00 = (iy as usize * w + ix as usize) * 3 + ch;
    let i10 = i00 + 3;
    let i01 = i00 + w * 3;
    let i11 = i01 + 3;
    src[i00] * (1.0 - fx) * (1.0 - fy)
        + src[i10] * fx * (1.0 - fy)
        + src[i01] * (1.0 - fx) * fy
        + src[i11] * fx * fy
}

/// # Develop Stage
///
/// The core physical simulation:
/// - Spectral Sensitivity (RGB -> Spectrum -> Exposure)
/// - Reciprocity Failure (Exposure Adjustment)
/// - White Balance (Exposure Gain)
/// - H-D Curves (Exposure -> Density)
pub struct DevelopStage;

const SPECTRAL_NORM: f32 = 1.0;

impl PipelineStage for DevelopStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        info!("Developing film (Spectral -> Exposure -> Density)");
        let film = context.film;
        let config = context.config;
        let width = image.width();
        let height = image.height();

        // Precompute Spectral Matrix (3x3)
        // Maps Linear RGB -> Film Layer Exposure directly
        let spectral_matrix = film.compute_spectral_matrix();

        let apply_matrix = |r: f32, g: f32, b: f32| -> [f32; 3] {
            [
                r * spectral_matrix[0][0] + g * spectral_matrix[0][1] + b * spectral_matrix[0][2],
                r * spectral_matrix[1][0] + g * spectral_matrix[1][1] + b * spectral_matrix[1][2],
                r * spectral_matrix[2][0] + g * spectral_matrix[2][1] + b * spectral_matrix[2][2],
            ]
        };

        let reciprocity_factor = if config.exposure_time > 1.0 {
            1.0 + film.reciprocity.beta * config.exposure_time.log10().powi(2)
        } else {
            1.0
        };
        let t_eff = config.exposure_time / reciprocity_factor;

        // White Balance Calculation
        let wb_gains = match config.white_balance_mode {
            WhiteBalanceMode::Auto => {
                let step = (width * height / 1000).max(1);
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut count = 0.0;

                for i in (0..(width * height)).step_by(step as usize) {
                    let x = i % width;
                    let y = i / width;
                    let p = image.get_pixel(x, y).0;

                    let exposure_vals = apply_matrix(p[0], p[1], p[2]);

                    sum_r += exposure_vals[0];
                    sum_g += exposure_vals[1];
                    sum_b += exposure_vals[2];
                    count += 1.0;
                }

                if count > 0.0 {
                    let avg_r = sum_r / count;
                    let avg_g = sum_g / count;
                    let avg_b = sum_b / count;
                    let lum = (avg_r + avg_g + avg_b) / 3.0;
                    let eps = 1e-9;

                    let gain_r = lum / avg_r.max(eps);
                    let gain_g = lum / avg_g.max(eps);
                    let gain_b = lum / avg_b.max(eps);

                    let s = config.white_balance_strength.clamp(0.0, 1.0);
                    let base_gains = [
                        1.0 + (gain_r - 1.0) * s,
                        1.0 + (gain_g - 1.0) * s,
                        1.0 + (gain_b - 1.0) * s,
                    ];

                    // Apply Warmth (Shift R/B)
                    let warmth = config.warmth.clamp(-1.0, 1.0);
                    [
                        base_gains[0] * (1.0 + warmth * 0.1),
                        base_gains[1],
                        base_gains[2] * (1.0 - warmth * 0.1),
                    ]
                } else {
                    [1.0, 1.0, 1.0]
                }
            }
            _ => {
                // Manual/Off mode still supports Warmth
                let warmth = config.warmth.clamp(-1.0, 1.0);
                [1.0 + warmth * 0.1, 1.0, 1.0 - warmth * 0.1]
            }
        };

        // Transform in place: Linear -> Density
        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let _x = (i as u32) % width;
            let _y = (i as u32) / width;
            // Current pixel is Linear RGB
            let lin_pixel = [pixel[0], pixel[1], pixel[2]];

            let exposure_vals = apply_matrix(lin_pixel[0], lin_pixel[1], lin_pixel[2]);

            let r_balanced = exposure_vals[0] * wb_gains[0];
            let g_balanced = exposure_vals[1] * wb_gains[1];
            let b_balanced = exposure_vals[2] * wb_gains[2];

            let r_in = (r_balanced * SPECTRAL_NORM).max(0.0);
            let g_in = (g_balanced * SPECTRAL_NORM).max(0.0);
            let b_in = (b_balanced * SPECTRAL_NORM).max(0.0);

            let r_exp = physics::calculate_exposure(r_in, t_eff);
            let g_exp = physics::calculate_exposure(g_in, t_eff);
            let b_exp = physics::calculate_exposure(b_in, t_eff);

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

/// # Auto Levels Stage
///
/// Stretches the image's actual density range to fill the full output range,
/// like a film scanner's auto-levels. Uses 1st/99th percentile to avoid
/// outlier influence. Operates on density values before output conversion.
pub struct AutoLevelsStage;

impl PipelineStage for AutoLevelsStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        if !context.config.auto_levels {
            return;
        }

        let n = image.width() as usize * image.height() as usize;
        if n == 0 {
            return;
        }

        info!("Applying auto levels (black/white point stretch)");

        // Collect per-channel values (subsample for performance on large images)
        let step = (n / 50_000).max(1);
        let mut vals: [Vec<f32>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for (i, pixel) in image.chunks(3).enumerate() {
            if i % step == 0 {
                vals[0].push(pixel[0]);
                vals[1].push(pixel[1]);
                vals[2].push(pixel[2]);
            }
        }

        // Find 1st and 99th percentile per channel
        let mut lo = [0.0f32; 3];
        let mut hi = [0.0f32; 3];
        for c in 0..3 {
            vals[c].sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let len = vals[c].len();
            lo[c] = vals[c][(len as f32 * 0.01) as usize];
            hi[c] = vals[c][((len as f32 * 0.99) as usize).min(len - 1)];
        }

        // Stretch each channel: (val - lo) / (hi - lo)
        let range = [
            (hi[0] - lo[0]).max(0.01),
            (hi[1] - lo[1]).max(0.01),
            (hi[2] - lo[2]).max(0.01),
        ];

        image.par_chunks_mut(3).for_each(|pixel| {
            pixel[0] = ((pixel[0] - lo[0]) / range[0]).clamp(0.0, 1.0);
            pixel[1] = ((pixel[1] - lo[1]) / range[1]).clamp(0.0, 1.0);
            pixel[2] = ((pixel[2] - lo[2]) / range[2]).clamp(0.0, 1.0);
        });
    }
}

/// # Grain Stage
///
/// Adds film grain noise based on density.
/// Supports both monochrome and color grain models.
pub struct GrainStage;

impl PipelineStage for GrainStage {
    #[instrument(skip(self, image, context))]
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        if !context.config.enable_grain {
            debug!("Grain disabled");
            return;
        }
        info!("Applying Grain noise");
        let film = context.film;
        let width = image.width();
        let height = image.height();
        let gm = &film.grain_model;

        // Physical grain size in pixels
        let pixels_per_mm = width as f32 / 36.0;
        let grain_sigma = (gm.blur_radius * 0.05 * pixels_per_mm).max(0.8);

        let mono = gm.monochrome;
        let n_textures = if mono { 1 } else { 4 }; // mono: 1 shared; color: shared + R/G/B

        // Generate and blur noise textures
        let gen_and_blur = |sigma: f32| -> Vec<f32> {
            let mut tex = vec![0.0f32; (width * height) as usize];
            tex.par_chunks_mut(1).for_each(|p| {
                let mut rng = rand::thread_rng();
                p[0] = rand_distr::Distribution::sample(
                    &rand_distr::Normal::new(0.0f32, 1.0f32).unwrap(),
                    &mut rng,
                );
            });
            if sigma >= 0.5 {
                let mut img: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);
                img.chunks_mut(3).enumerate().for_each(|(i, pixel)| {
                    pixel[0] = tex[i];
                    pixel[1] = tex[i];
                    pixel[2] = tex[i];
                });
                utils::apply_gaussian_blur(&mut img, sigma);
                img.chunks(3).enumerate().for_each(|(i, pixel)| {
                    tex[i] = pixel[0];
                });
            }
            tex
        };

        let textures: Vec<Vec<f32>> = (0..n_textures).map(|_| gen_and_blur(grain_sigma)).collect();

        // Grain strength: Selwyn law σ_D = alpha × √D
        // In sRGB output space, this needs significant amplification.
        // Real Portra 400 shows σ ≈ 8-20 in sRGB 8-bit; our density-space
        // grain needs to produce similar output variation.
        let corr = gm.color_correlation;
        let alpha = gm.alpha;
        // Scale factor: alpha is tiny (0.000125 for Portra), but real grain σ_D ≈ 0.02-0.05.
        // boost converts alpha to physical σ_D scale.
        let boost = 2000.0f32;

        image.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
            let shared = textures[0][i];
            let (nr, ng, nb) = if mono {
                (shared, shared, shared)
            } else {
                (
                    corr * shared + (1.0 - corr) * textures[1][i],
                    corr * shared + (1.0 - corr) * textures[2][i],
                    corr * shared + (1.0 - corr) * textures[3][i],
                )
            };

            // Selwyn: σ_D ∝ √D. Additive grain in density space.
            let str_r = alpha * boost * pixel[0].max(0.01).sqrt();
            let str_g = alpha * boost * pixel[1].max(0.01).sqrt();
            let str_b = alpha * boost * pixel[2].max(0.01).sqrt();

            pixel[0] = (pixel[0] + str_r * nr).max(0.0);
            pixel[1] = (pixel[1] + str_g * ng).max(0.0);
            pixel[2] = (pixel[2] + str_b * nb).max(0.0);
        });
    }
}

/// # Output Stage (Final Conversion)
///
/// Converts Density to final output color space.
/// - Negative Mode: Simulates transmission light through the negative.
/// - Positive Mode: Simulates scan/inversion for display.
/// - Applies Color Matrix (Crosstalk) and CMY -> RGB conversion.
#[instrument(skip(image, context))]
pub fn create_output_image(
    image: &ImageBuffer<Rgb<f32>, Vec<f32>>,
    context: &PipelineContext,
) -> RgbImage {
    info!("Converting to final output image");
    let width = image.width();
    let height = image.height();
    let film = context.film;
    let config = context.config;

    let map_densities = |densities: [f32; 3]| -> (f32, f32, f32) {
        let net_r = (densities[0] - film.r_curve.d_min).max(0.0);
        let net_g = (densities[1] - film.g_curve.d_min).max(0.0);
        let net_b = (densities[2] - film.b_curve.d_min).max(0.0);
        match config.output_mode {
            OutputMode::Negative => {
                let t_r = physics::apply_dye_self_absorption(
                    net_r,
                    physics::density_to_transmission(net_r),
                );
                let t_g = physics::apply_dye_self_absorption(
                    net_g,
                    physics::density_to_transmission(net_g),
                );
                let t_b = physics::apply_dye_self_absorption(
                    net_b,
                    physics::density_to_transmission(net_b),
                );
                (
                    t_r.clamp(0.0, 1.0),
                    t_g.clamp(0.0, 1.0),
                    t_b.clamp(0.0, 1.0),
                )
            }
            OutputMode::Positive => {
                // Density linear mapping + tone gamma (scan model)
                let range_r = (film.r_curve.d_max - film.r_curve.d_min).max(0.01);
                let range_g = (film.g_curve.d_max - film.g_curve.d_min).max(0.01);
                let range_b = (film.b_curve.d_max - film.b_curve.d_min).max(0.01);
                let tone_gamma = match film.film_type {
                    FilmType::ColorSlide => 1.5,
                    _ => 2.47, // ln(0.18)/ln(0.5): maps density midpoint to 18% gray
                };
                (
                    (net_r / range_r).clamp(0.0, 1.0).powf(tone_gamma),
                    (net_g / range_g).clamp(0.0, 1.0).powf(tone_gamma),
                    (net_b / range_b).clamp(0.0, 1.0).powf(tone_gamma),
                )
            }
        }
    };

    // Extract dye spectra from layer_stack (if available) for spectral output path
    let dye_spectra = film.layer_stack.as_ref().and_then(|stack| {
        use crate::film_layer::{EmulsionChannel, LayerKind};
        let mut yellow = None;
        let mut magenta = None;
        let mut cyan = None;
        for layer in &stack.layers {
            if let LayerKind::Emulsion { channel } = layer.kind {
                if let Some(ref dye) = layer.dye_spectrum {
                    match channel {
                        EmulsionChannel::Blue => yellow = Some(*dye),
                        EmulsionChannel::Green => magenta = Some(*dye),
                        EmulsionChannel::Red => cyan = Some(*dye),
                    }
                }
            }
        }
        match (yellow, magenta, cyan) {
            (Some(y), Some(m), Some(c)) => Some((y, m, c)),
            _ => None,
        }
    });

    // Precompute D65 × CIE XYZ for spectral output (if dye spectra available)
    let spectral_output = dye_spectra.map(|(y_dye, m_dye, c_dye)| {
        use crate::cie_data::{CIE_X, CIE_Y, CIE_Z, D65_SPD, XYZ_TO_SRGB};
        use crate::spectral::{BINS, LAMBDA_STEP};
        // Precompute D65 × CMF
        let mut d65_x = [0.0f32; BINS];
        let mut d65_y = [0.0f32; BINS];
        let mut d65_z = [0.0f32; BINS];
        for i in 0..BINS {
            d65_x[i] = D65_SPD[i] * CIE_X[i] * LAMBDA_STEP as f32;
            d65_y[i] = D65_SPD[i] * CIE_Y[i] * LAMBDA_STEP as f32;
            d65_z[i] = D65_SPD[i] * CIE_Z[i] * LAMBDA_STEP as f32;
        }
        // White point normalization: Y of D65 should = 1.0
        let y_sum: f32 = d65_y.iter().sum();
        let y_norm = if y_sum > 0.0 { 1.0 / y_sum } else { 1.0 };
        (
            y_dye,
            m_dye,
            c_dye,
            d65_x,
            d65_y,
            d65_z,
            y_norm,
            XYZ_TO_SRGB,
        )
    });

    // First pass: compute linear RGB for all pixels
    let mut linear_buf: Vec<f32> = vec![0.0; (width * height * 3) as usize];

    linear_buf
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(i, out)| {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            let d = image.get_pixel(x, y).0;

            let (mut r_lin, mut g_lin, mut b_lin) = if let Some(ref sp) = spectral_output {
                let (y_dye, m_dye, c_dye, d65_x, d65_y, d65_z, y_norm, xyz_to_srgb) = sp;
                let net = [
                    (d[0] - film.r_curve.d_min).max(0.0),
                    (d[1] - film.g_curve.d_min).max(0.0),
                    (d[2] - film.b_curve.d_min).max(0.0),
                ];
                let mut xyz = [0.0f32; 3];
                for i in 0..crate::spectral::BINS {
                    let od = net[0] * c_dye[i] + net[1] * m_dye[i] + net[2] * y_dye[i];
                    let t = 10.0f32.powf(-od);
                    xyz[0] += t * d65_x[i];
                    xyz[1] += t * d65_y[i];
                    xyz[2] += t * d65_z[i];
                }
                xyz[0] *= y_norm;
                xyz[1] *= y_norm;
                xyz[2] *= y_norm;
                let mut r = xyz_to_srgb[0][0] * xyz[0]
                    + xyz_to_srgb[0][1] * xyz[1]
                    + xyz_to_srgb[0][2] * xyz[2];
                let mut g = xyz_to_srgb[1][0] * xyz[0]
                    + xyz_to_srgb[1][1] * xyz[1]
                    + xyz_to_srgb[1][2] * xyz[2];
                let mut b = xyz_to_srgb[2][0] * xyz[0]
                    + xyz_to_srgb[2][1] * xyz[1]
                    + xyz_to_srgb[2][2] * xyz[2];
                if film.film_type == FilmType::ColorNegative
                    || film.film_type == FilmType::BwNegative
                {
                    r = 1.0 - r;
                    g = 1.0 - g;
                    b = 1.0 - b;
                }
                (r, g, b)
            } else {
                map_densities(d)
            };

            if config.saturation != 1.0 {
                let lum = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
                r_lin = lum + (r_lin - lum) * config.saturation;
                g_lin = lum + (g_lin - lum) * config.saturation;
                b_lin = lum + (b_lin - lum) * config.saturation;
            }

            let v_str = film.vignette_strength;
            if v_str > 0.0 {
                let px = i as u32 % width;
                let py = i as u32 / width;
                let dx = (px as f32 + 0.5) / width as f32 - 0.5;
                let dy = (py as f32 + 0.5) / height as f32 - 0.5;
                let r2 = dx * dx + dy * dy;
                let f2 = 0.2;
                let cos4 = 1.0 / (1.0 + r2 / f2).powi(2);
                let factor = 1.0 - v_str * (1.0 - cos4);
                r_lin *= factor;
                g_lin *= factor;
                b_lin *= factor;
            }

            out[0] = r_lin;
            out[1] = g_lin;
            out[2] = b_lin;
        });

    // Auto Levels in linear f32 space (no banding)
    if config.auto_levels {
        let n = linear_buf.len() / 3;
        let step = (n / 50_000).max(1);
        let mut lums: Vec<f32> = Vec::with_capacity(n / step + 1);
        for i in (0..n).step_by(step) {
            let idx = i * 3;
            lums.push(
                0.2126 * linear_buf[idx]
                    + 0.7152 * linear_buf[idx + 1]
                    + 0.0722 * linear_buf[idx + 2],
            );
        }
        lums.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p_lo = lums[(lums.len() as f32 * 0.01) as usize];
        let p_hi = lums[((lums.len() as f32 * 0.99) as usize).min(lums.len() - 1)];
        let strength = 0.6;
        let lo = p_lo * (1.0 - strength);
        let hi = p_hi + (1.0 - p_hi) * strength;
        let range = (hi - lo).max(1e-6);
        linear_buf.par_chunks_mut(3).for_each(|px| {
            px[0] = ((px[0] - lo) / range).clamp(0.0, 1.0);
            px[1] = ((px[1] - lo) / range).clamp(0.0, 1.0);
            px[2] = ((px[2] - lo) / range).clamp(0.0, 1.0);
        });
    }

    // Grain in linear output space (after tone mapping, before sRGB)
    if config.enable_grain {
        let gm = &film.grain_model;
        let pixels_per_mm = width as f32 / 36.0;
        let grain_sigma = (gm.blur_radius * 0.05 * pixels_per_mm).max(0.8);
        let mono = gm.monochrome;
        let n_tex = if mono { 1usize } else { 4 };

        // Generate blurred noise textures
        let gen_blur = |sigma: f32| -> Vec<f32> {
            let mut tex = vec![0.0f32; (width * height) as usize];
            tex.par_chunks_mut(1).for_each(|p| {
                let mut rng = rand::thread_rng();
                p[0] = rand_distr::Distribution::sample(
                    &rand_distr::Normal::new(0.0f32, 1.0f32).unwrap(),
                    &mut rng,
                );
            });
            if sigma >= 0.5 {
                let mut img: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(width, height);
                img.chunks_mut(3).enumerate().for_each(|(i, px)| {
                    px[0] = tex[i];
                    px[1] = tex[i];
                    px[2] = tex[i];
                });
                utils::apply_gaussian_blur(&mut img, sigma);
                img.chunks(3).enumerate().for_each(|(i, px)| {
                    tex[i] = px[0];
                });
            }
            tex
        };
        let textures: Vec<Vec<f32>> = (0..n_tex).map(|_| gen_blur(grain_sigma)).collect();

        let corr = gm.color_correlation;
        // Grain strength in linear output space.
        // Real Portra 400 σ ≈ 8-20 in sRGB 8-bit → σ ≈ 0.03-0.08 in linear.
        // Scale by alpha (preset-specific) and pixel brightness (Selwyn: brighter = less grain).
        let base_strength = gm.alpha * 1500.0;

        linear_buf
            .par_chunks_mut(3)
            .enumerate()
            .for_each(|(i, px)| {
                let shared = textures[0][i];
                let (nr, ng, nb) = if mono {
                    (shared, shared, shared)
                } else {
                    (
                        corr * shared + (1.0 - corr) * textures[1][i],
                        corr * shared + (1.0 - corr) * textures[2][i],
                        corr * shared + (1.0 - corr) * textures[3][i],
                    )
                };
                // Selwyn in output space: grain stronger in shadows (low linear value)
                // σ ∝ sqrt(1 - brightness) — shadows get more grain
                let lum = (0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]).clamp(0.01, 1.0);
                // Selwyn law: σ_D ∝ √D. In output space, high density = low brightness.
                // Grain stronger in shadows, weaker in highlights.
                // But cap absolute noise to avoid bright speckles in pure black.
                let selwyn = (1.0 - lum).sqrt();
                let strength = base_strength * selwyn * lum.max(0.05);
                px[0] = (px[0] + strength * nr).clamp(0.0, 1.0);
                px[1] = (px[1] + strength * ng).clamp(0.0, 1.0);
                px[2] = (px[2] + strength * nb).clamp(0.0, 1.0);
            });
    }

    // Final pass: linear → sRGB u8
    let mut pixels: Vec<u8> = vec![0; (width * height * 3) as usize];
    pixels.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let idx = i * 3;
        chunk[0] = (physics::linear_to_srgb(linear_buf[idx].clamp(0.0, 1.0)) * 255.0).round() as u8;
        chunk[1] =
            (physics::linear_to_srgb(linear_buf[idx + 1].clamp(0.0, 1.0)) * 255.0).round() as u8;
        chunk[2] =
            (physics::linear_to_srgb(linear_buf[idx + 2].clamp(0.0, 1.0)) * 255.0).round() as u8;
    });

    RgbImage::from_raw(width, height, pixels).unwrap()
}
