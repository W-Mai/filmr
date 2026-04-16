//! Monocular depth estimation using Depth Anything V2 Small (ONNX).
//!
//! Requires the `depth` feature flag and a downloaded ONNX model.
//! Model: https://huggingface.co/onnx-community/depth-anything-v2-small

/// Depth map: normalized relative depth values [0.0, 1.0] at original image resolution.
/// 0.0 = nearest, 1.0 = farthest.
#[derive(Clone)]
pub struct DepthMap {
    pub data: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

impl DepthMap {
    /// Get depth at pixel (x, y). Returns 0.0 (near) to 1.0 (far).
    pub fn get(&self, x: u32, y: u32) -> f32 {
        if x < self.width && y < self.height {
            self.data[y as usize * self.width as usize + x as usize]
        } else {
            0.5
        }
    }
}

/// Default model directory: ~/.filmr/models/
#[cfg(feature = "depth")]
pub fn default_model_dir() -> std::path::PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".filmr")
        .join("models")
}

/// Default model path for Depth Anything V2 Small.
#[cfg(feature = "depth")]
pub fn default_model_path() -> std::path::PathBuf {
    default_model_dir().join("depth_anything_v2_vits.onnx")
}

/// Check if the depth model is available.
#[cfg(feature = "depth")]
pub fn is_model_available() -> bool {
    default_model_path().exists()
}

/// Run depth estimation on an RGB image.
#[cfg(feature = "depth")]
pub fn estimate(image: &image::RgbImage) -> Result<DepthMap, Box<dyn std::error::Error>> {
    estimate_with_model(image, &default_model_path().to_string_lossy())
}

/// Run depth estimation with a specific model path.
#[cfg(feature = "depth")]
pub fn estimate_with_model(
    image: &image::RgbImage,
    model_path: &str,
) -> Result<DepthMap, Box<dyn std::error::Error>> {
    use ort::session::Session;
    use ort::value::Tensor;

    let (orig_w, orig_h) = (image.width(), image.height());
    let input_size = 518u32;

    // Resize + pad (keep aspect ratio)
    let scale = input_size as f32 / orig_w.max(orig_h) as f32;
    let scaled_w = (orig_w as f32 * scale).round() as u32;
    let scaled_h = (orig_h as f32 * scale).round() as u32;
    let resized = image::imageops::resize(
        image,
        scaled_w,
        scaled_h,
        image::imageops::FilterType::Lanczos3,
    );
    let mut padded = image::RgbImage::new(input_size, input_size);
    let pad_x = (input_size - scaled_w) / 2;
    let pad_y = (input_size - scaled_h) / 2;
    image::imageops::overlay(&mut padded, &resized, pad_x as i64, pad_y as i64);

    // Normalize (ImageNet mean/std) → NCHW f32
    let mean = [0.485f32, 0.456, 0.406];
    let std_dev = [0.229f32, 0.224, 0.225];
    let n = (input_size * input_size) as usize;
    let mut data = vec![0.0f32; 3 * n];
    for y in 0..input_size {
        for x in 0..input_size {
            let p = padded.get_pixel(x, y);
            for c in 0..3 {
                data[c * n + y as usize * input_size as usize + x as usize] =
                    (p[c] as f32 / 255.0 - mean[c]) / std_dev[c];
            }
        }
    }

    let input_tensor = Tensor::from_array((
        vec![1i64, 3, input_size as i64, input_size as i64],
        data.into_boxed_slice(),
    ))?;

    let mut session = Session::builder()?.commit_from_file(model_path)?;
    let outputs = session.run(ort::inputs!["pixel_values" => input_tensor])?;

    let depth_array = outputs[0].try_extract_array::<f32>()?;
    let raw: Vec<f32> = depth_array.iter().cloned().collect();

    // Normalize to [0, 1]
    let d_min = raw.iter().cloned().fold(f32::MAX, f32::min);
    let d_max = raw.iter().cloned().fold(f32::MIN, f32::max);
    let range = (d_max - d_min).max(1e-6);
    let normalized: Vec<f32> = raw.iter().map(|v| (v - d_min) / range).collect();

    // Remove padding + resize to original
    let mut result = vec![0.0f32; (orig_w * orig_h) as usize];
    for y in 0..orig_h {
        for x in 0..orig_w {
            let sx = pad_x as f32 + x as f32 * (scaled_w - 1) as f32 / (orig_w - 1).max(1) as f32;
            let sy = pad_y as f32 + y as f32 * (scaled_h - 1) as f32 / (orig_h - 1).max(1) as f32;
            let ix = (sx as usize).min(input_size as usize - 2);
            let iy = (sy as usize).min(input_size as usize - 2);
            let fx = sx - ix as f32;
            let fy = sy - iy as f32;
            let sw = input_size as usize;
            let v = normalized[iy * sw + ix] * (1.0 - fx) * (1.0 - fy)
                + normalized[iy * sw + ix + 1] * fx * (1.0 - fy)
                + normalized[(iy + 1) * sw + ix] * (1.0 - fx) * fy
                + normalized[(iy + 1) * sw + ix + 1] * fx * fy;
            result[y as usize * orig_w as usize + x as usize] = v;
        }
    }

    Ok(DepthMap {
        data: result,
        width: orig_w,
        height: orig_h,
    })
}
