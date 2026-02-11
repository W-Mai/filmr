//! File I/O methods for FilmrApp.

use super::FilmrApp;

impl FilmrApp {
    /// Write EXIF metadata to the saved file, preserving original EXIF and adding Filmr copyright.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_exif_to_file(&self, path: &std::path::Path) {
        use little_exif::exif_tag::ExifTag;
        use little_exif::metadata::Metadata;

        let mut metadata = if let Some(ref source_exif) = self.source_exif {
            source_exif.clone()
        } else {
            Metadata::new()
        };

        // Add Filmr processing info
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

        if let Err(e) = metadata.write_to_file(path) {
            tracing::warn!("Failed to write EXIF metadata: {}", e);
        }
    }

    /// Save the developed image to a file.
    pub fn save_image(&mut self) {
        let default_name = self
            .source_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| format!("{}_FILMR.jpg", s.to_string_lossy()))
            .unwrap_or_else(|| "filmr_output.jpg".to_string());

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(img) = &self.developed_image {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .add_filter("PNG Image", &["png"])
                .save_file()
            {
                if let Err(e) = img.save(&path) {
                    self.status_msg = format!("Failed to save image: {}", e);
                } else {
                    // Write EXIF metadata to saved file
                    self.write_exif_to_file(&path);
                    self.status_msg = format!("Saved to {:?}", path);
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(img) = &self.developed_image {
                let mut bytes: Vec<u8> = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut bytes);
                if let Err(e) = img.write_to(&mut cursor, image::ImageFormat::Jpeg) {
                    self.status_msg = format!("Failed to encode image: {}", e);
                    return;
                }

                let task = rfd::AsyncFileDialog::new()
                    .set_file_name(&default_name)
                    .save_file();

                let bytes = bytes.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(handle) = task.await {
                        if let Err(_e) = handle.write(&bytes).await {
                            // Log error?
                        }
                    }
                });
                self.status_msg = "Download started...".to_owned();
            }
        }
    }
}
