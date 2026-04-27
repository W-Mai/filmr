//! Monocular depth estimation using Depth Anything V2 Small (RTen).
//!
//! Requires the `depth` feature flag and a downloaded .rten model.
//! Model converted from: https://huggingface.co/onnx-community/depth-anything-v2-small

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

/// Default model path for Depth Anything V2 Small (.rten format).
#[cfg(feature = "depth")]
pub fn default_model_path() -> std::path::PathBuf {
    default_model_dir().join("depth_anything_v2_vits.rten")
}

/// Check if the depth model is available.
#[cfg(feature = "depth")]
pub fn is_model_available() -> bool {
    default_model_path().exists()
}

pub const MODEL_URL: &str =
    "https://github.com/W-Mai/filmr/releases/download/models-v1/depth_anything_v2_vits.rten";
pub const MODEL_SIZE: u64 = 99_060_839; // ~95MB

/// Download the depth model with progress callback.
/// `on_progress(downloaded_bytes, total_bytes)` called periodically.
#[cfg(feature = "depth")]
pub fn download_model(on_progress: impl Fn(u64, u64)) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    let dir = default_model_dir();
    std::fs::create_dir_all(&dir)?;

    let resp = ureq::get(MODEL_URL).call()?;
    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(MODEL_SIZE);

    let mut reader = resp.into_body().into_reader();
    let tmp = dir.join("depth_anything_v2_vits.rten.tmp");
    let mut file = std::fs::File::create(&tmp)?;

    let mut downloaded = 0u64;
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n])?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }

    std::fs::rename(&tmp, default_model_path())?;
    Ok(())
}

/// Delete the depth model.
#[cfg(feature = "depth")]
pub fn delete_model() -> std::io::Result<()> {
    let path = default_model_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
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
    use rten::Model;
    use rten_tensor::prelude::*;
    use rten_tensor::NdTensor;

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

    // Load model and run inference
    let model = Model::load_file(model_path)?;
    let input: NdTensor<f32, 4> =
        NdTensor::from_data([1, 3, input_size as usize, input_size as usize], data);
    let input_id = model.node_id("pixel_values")?;
    let output_id = model.output_ids()[0];

    let mut results = model.run(vec![(input_id, input.as_dyn().into())], &[output_id], None)?;

    let output = results
        .remove(0)
        .into_tensor::<f32>()
        .ok_or("output not float")?;
    let raw: Vec<f32> = output.iter().copied().collect();

    // Normalize to [0, 1]
    let d_min = raw.iter().cloned().fold(f32::MAX, f32::min);
    let d_max = raw.iter().cloned().fold(f32::MIN, f32::max);
    let range = (d_max - d_min).max(1e-6);
    let normalized: Vec<f32> = raw.iter().map(|v| (v - d_min) / range).collect();

    // Remove padding + resize to original (bilinear interpolation)
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
