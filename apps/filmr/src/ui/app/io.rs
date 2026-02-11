//! File I/O methods for FilmrApp.

use super::FilmrApp;

impl FilmrApp {
    /// Build EXIF metadata with Filmr processing info.
    pub fn build_exif_metadata(&self) -> little_exif::metadata::Metadata {
        use little_exif::exif_tag::ExifTag;

        let mut metadata = self.source_exif.clone().unwrap_or_default();

        let stock_name = self.get_current_stock().name.clone();
        metadata.set_tag(ExifTag::Software(
            "Filmr - Physics-based Film Simulation".to_string(),
        ));
        metadata.set_tag(ExifTag::ImageDescription(format!(
            "Processed with Filmr using {} film stock",
            stock_name
        )));
        metadata.set_tag(ExifTag::Copyright(
            "Processed by Filmr (https://github.com/W-Mai/filmr)".to_string(),
        ));

        metadata
    }

    /// Save the developed image to a file.
    pub fn save_image(&mut self) {
        let default_name = self
            .source_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| format!("{}_FILMR.jpg", s.to_string_lossy()))
            .unwrap_or_else(|| "filmr_output.jpg".to_string());

        let Some(img) = &self.developed_image else {
            return;
        };

        // Encode image to bytes
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut bytes);
        if let Err(e) = img.write_to(&mut cursor, image::ImageFormat::Jpeg) {
            self.status_msg = format!("Failed to encode image: {}", e);
            return;
        }

        // Write EXIF metadata to bytes
        let metadata = self.build_exif_metadata();
        let file_ext = if default_name.ends_with(".png") {
            little_exif::filetype::FileExtension::PNG {
                as_zTXt_chunk: false,
            }
        } else {
            little_exif::filetype::FileExtension::JPEG
        };
        if let Err(e) = metadata.write_to_vec(&mut bytes, file_ext) {
            tracing::warn!("Failed to write EXIF metadata: {}", e);
        }

        // Write bytes to file/download
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .add_filter("PNG Image", &["png"])
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, &bytes) {
                    self.status_msg = format!("Failed to save image: {}", e);
                } else {
                    self.status_msg = format!("Saved to {:?}", path);
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let task = rfd::AsyncFileDialog::new()
                .set_file_name(&default_name)
                .save_file();

            wasm_bindgen_futures::spawn_local(async move {
                if let Some(handle) = task.await {
                    let _ = handle.write(&bytes).await;
                }
            });
            self.status_msg = "Download started...".to_owned();
        }
    }
}
