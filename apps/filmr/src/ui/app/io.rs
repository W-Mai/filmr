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

        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .add_filter("PNG Image", &["png"])
                .add_filter("TIFF Image (16-bit)", &["tiff", "tif"])
                .save_file()
            else {
                return;
            };

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_lowercase();

            let result = match ext.as_str() {
                "png" => {
                    // PNG 8-bit with sRGB
                    let mut bytes = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut bytes);
                    img.write_to(&mut cursor, image::ImageFormat::Png)
                        .map(|_| bytes)
                }
                "tiff" | "tif" => {
                    // TIFF 16-bit: convert 8-bit RGB to 16-bit
                    let rgb8 = img.to_rgb8();
                    let (w, h) = (rgb8.width(), rgb8.height());
                    let pixels_16: Vec<u16> =
                        rgb8.as_raw().iter().map(|&v| (v as u16) * 257).collect();
                    let bytes_16: Vec<u8> =
                        pixels_16.iter().flat_map(|v| v.to_ne_bytes()).collect();
                    let mut bytes = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut bytes);
                    let encoder = image::codecs::tiff::TiffEncoder::new(&mut cursor);
                    encoder
                        .encode(&bytes_16, w, h, image::ExtendedColorType::Rgb16)
                        .map(|_| bytes)
                }
                _ => {
                    // JPEG (default)
                    let mut bytes = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut bytes);
                    img.write_to(&mut cursor, image::ImageFormat::Jpeg)
                        .map(|_| {
                            // Embed EXIF with sRGB tag
                            let mut metadata = self.build_exif_metadata();
                            metadata
                                .set_tag(little_exif::exif_tag::ExifTag::ColorSpace(vec![1u16]));
                            let _ = metadata.write_to_vec(
                                &mut bytes,
                                little_exif::filetype::FileExtension::JPEG,
                            );
                            bytes
                        })
                }
            };

            match result {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(&path, &bytes) {
                        self.status_msg = format!("Failed to save: {}", e);
                    } else {
                        self.status_msg = format!("Saved to {:?}", path);
                    }
                }
                Err(e) => {
                    self.status_msg = format!("Failed to encode: {}", e);
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            // WASM: always JPEG
            let mut bytes: Vec<u8> = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut bytes);
            if let Err(e) = img.write_to(&mut cursor, image::ImageFormat::Jpeg) {
                self.status_msg = format!("Failed to encode: {}", e);
                return;
            }
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
