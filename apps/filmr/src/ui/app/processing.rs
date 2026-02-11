//! Image processing methods for FilmrApp.

use super::workers::ProcessRequest;
use super::FilmrApp;
use crate::config::AppMode;
use egui::Context;
use filmr::{estimate_exposure_time, light_leak::LightLeakConfig, SimulationConfig};
use std::sync::Arc;

impl FilmrApp {
    /// Process the preview image and update the texture.
    pub fn process_and_update_texture(&mut self, _ctx: &Context) {
        // Use preview image for GUI display to maintain responsiveness
        // For preview, we use the pre-converted Arc<RgbImage>
        let source_image = self.preview_image.as_ref();

        if let Some(img) = source_image {
            // Construct params
            // Use preset as base and modify
            let base_film = if self.mode == AppMode::StockStudio {
                self.studio_stock.clone()
            } else {
                self.get_current_stock().as_ref().clone()
            };

            let mut film = base_film;
            if self.mode == AppMode::Develop {
                // Only apply UI overrides in Develop mode
                film.halation_strength = self.halation_strength;
                film.halation_threshold = self.halation_threshold;
                film.halation_sigma = self.halation_sigma;

                film.grain_model.alpha = self.grain_alpha;
                film.grain_model.sigma_read = self.grain_sigma;
                film.grain_model.roughness = self.grain_roughness;
                film.grain_model.blur_radius = self.grain_blur_radius;

                // Apply gamma boost to all channels
                film.r_curve.gamma *= self.gamma_boost;
                film.g_curve.gamma *= self.gamma_boost;
                film.b_curve.gamma *= self.gamma_boost;
            }

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true,
                use_gpu: true,
                output_mode: self.output_mode,
                white_balance_mode: if self.ux_mode == crate::config::UxMode::Simple {
                    filmr::WhiteBalanceMode::Off
                } else {
                    self.white_balance_mode
                },
                white_balance_strength: self.white_balance_strength,
                warmth: self.warmth,
                saturation: self.saturation,
                light_leak: self.light_leak_config.clone(),
            };

            // Send request to worker
            // Direct clone of Arc, O(1)
            let request = ProcessRequest {
                image: Arc::clone(img),
                film,
                config,
                is_preview: true,
            };

            let _ = self.tx_req.send(request);
            self.is_processing = true;
        }
    }

    /// Regenerate thumbnails for all film stocks.
    pub fn regenerate_thumbnails(&self) {
        if let Some(img) = &self.original_image {
            let thumb_base = img.thumbnail(128, 128).to_rgb8();
            let thumb_config = SimulationConfig {
                exposure_time: 1.0,
                enable_grain: false,
                use_gpu: false,
                output_mode: self.output_mode,
                white_balance_mode: if self.ux_mode == crate::config::UxMode::Simple {
                    filmr::WhiteBalanceMode::Off
                } else {
                    self.white_balance_mode
                },
                white_balance_strength: self.white_balance_strength,
                warmth: self.warmth,
                saturation: self.saturation,
                light_leak: LightLeakConfig::default(),
            };
            for stock in &self.stocks {
                let mut thumb_stock = stock.as_ref().clone();
                // Apply gamma boost to thumbnail
                thumb_stock.r_curve.gamma *= self.gamma_boost;
                thumb_stock.g_curve.gamma *= self.gamma_boost;
                thumb_stock.b_curve.gamma *= self.gamma_boost;
                let _ = self.tx_thumb.send((
                    stock.full_name(),
                    thumb_base.clone(),
                    thumb_config.clone(),
                    thumb_stock,
                ));
            }
        }
    }

    /// Develop the full resolution image.
    pub fn develop_image(&mut self, _ctx: &Context) {
        if let Some(img) = &self.original_image {
            self.status_msg = "Developing full resolution image...".to_owned();

            // This might still take a bit of time to clone/convert, but it's unavoidable for full-res develop
            // unless we also keep full-res as RgbImage (memory intensive).
            let rgb_img = Arc::new(img.to_rgb8());

            let base_film = if self.mode == AppMode::StockStudio {
                self.studio_stock.clone()
            } else {
                self.get_current_stock().as_ref().clone()
            };
            let mut film = base_film;

            if self.mode == AppMode::Develop {
                film.halation_strength = self.halation_strength;
                film.halation_threshold = self.halation_threshold;
                film.halation_sigma = self.halation_sigma;
                film.r_curve.gamma *= self.gamma_boost;
                film.g_curve.gamma *= self.gamma_boost;
                film.b_curve.gamma *= self.gamma_boost;
            }

            let config = SimulationConfig {
                exposure_time: self.exposure_time,
                enable_grain: true,
                use_gpu: true,
                output_mode: self.output_mode,
                white_balance_mode: if self.ux_mode == crate::config::UxMode::Simple {
                    filmr::WhiteBalanceMode::Off
                } else {
                    self.white_balance_mode
                },
                white_balance_strength: self.white_balance_strength,
                warmth: self.warmth,
                saturation: self.saturation,
                light_leak: self.light_leak_config.clone(),
            };

            let request = ProcessRequest {
                image: rgb_img,
                film,
                config,
                is_preview: false,
            };

            let _ = self.tx_req.send(request);
            self.is_processing = true;
        }
    }

    /// Load preset values from the current stock into UI sliders.
    pub fn load_preset_values(&mut self) {
        let preset = self.get_current_stock();

        self.halation_strength = preset.halation_strength;
        self.halation_threshold = preset.halation_threshold;
        self.halation_sigma = preset.halation_sigma;

        self.grain_alpha = preset.grain_model.alpha;
        self.grain_sigma = preset.grain_model.sigma_read;
        self.grain_roughness = preset.grain_model.roughness;
        self.grain_blur_radius = preset.grain_model.blur_radius;

        let base_exposure = preset.r_curve.exposure_offset / 0.18;
        self.exposure_time = if let Some(img) = &self.original_image {
            estimate_exposure_time(&img.to_rgb8(), &preset)
        } else {
            base_exposure
        };
    }
}
