use crate::pipeline::{PipelineContext, PipelineStage};
use image::{ImageBuffer, Rgb};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightLeakConfig {
    pub enabled: bool,
    pub leaks: Vec<LightLeak>,
}

impl Default for LightLeakConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            leaks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightLeak {
    /// Center position (x, y) normalized to [0.0, 1.0]
    pub position: (f32, f32),
    /// Color of the light leak (Linear RGB)
    pub color: [f32; 3],
    /// Radius normalized to image minimum dimension
    pub radius: f32,
    /// Intensity multiplier
    pub intensity: f32,
    /// Type of shape
    pub shape: LightLeakShape,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LightLeakShape {
    Circle,
    Linear, // A line/streak
    // RandomBlob, // Could be added later with noise
}

pub struct LightLeakStage;

impl PipelineStage for LightLeakStage {
    fn process(&self, image: &mut ImageBuffer<Rgb<f32>, Vec<f32>>, context: &PipelineContext) {
        if !context.config.light_leak.enabled || context.config.light_leak.leaks.is_empty() {
            return;
        }

        let width = image.width() as f32;
        let height = image.height() as f32;
        let min_dim = width.min(height);

        for leak in &context.config.light_leak.leaks {
            let center_x = leak.position.0 * width;
            let center_y = leak.position.1 * height;
            let radius_px = leak.radius * min_dim;
            let radius_sq = radius_px * radius_px;

            // Bounding box optimization
            let min_x = (center_x - radius_px).max(0.0) as u32;
            let max_x = (center_x + radius_px).min(width) as u32;
            let min_y = (center_y - radius_px).max(0.0) as u32;
            let max_y = (center_y + radius_px).min(height) as u32;

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let dx = x as f32 - center_x;
                    let dy = y as f32 - center_y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < radius_sq {
                        let dist = dist_sq.sqrt();
                        let falloff = match leak.shape {
                            LightLeakShape::Circle => {
                                // Smoothstep-like falloff
                                let t = dist / radius_px;
                                (1.0 - t).powf(2.0)
                            }
                            LightLeakShape::Linear => {
                                // For linear, we interpret position as center, and radius as width/length?
                                // Simplified: Linear vertical streak for now
                                let t = dx.abs() / radius_px;
                                (1.0 - t).max(0.0).powf(2.0)
                            }
                        };
                        
                        let factor = falloff * leak.intensity;

                        let pixel = image.get_pixel_mut(x, y);
                        // Additive light
                        pixel[0] += leak.color[0] * factor;
                        pixel[1] += leak.color[1] * factor;
                        pixel[2] += leak.color[2] * factor;
                    }
                }
            }
        }
    }
}
