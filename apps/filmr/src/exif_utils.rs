//! EXIF orientation utilities for reading and applying image orientation transforms.

use image::DynamicImage;
use std::io::Seek;

/// Read EXIF orientation from a reader and return the orientation value (1-8).
/// Returns 1 (normal) if no orientation is found or on error.
pub fn read_exif_orientation<R: std::io::BufRead + Seek>(reader: &mut R) -> u32 {
    let exif_reader = exif::Reader::new();
    match exif_reader.read_from_container(reader) {
        Ok(exif) => {
            if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                field.value.get_uint(0).unwrap_or(1)
            } else {
                1
            }
        }
        Err(_) => 1,
    }
}

/// Apply EXIF orientation transform to a DynamicImage.
/// Orientation values follow EXIF spec:
/// - 1: Normal
/// - 2: Flip horizontal
/// - 3: Rotate 180
/// - 4: Flip vertical
/// - 5: Transpose (rotate 90 CW + flip horizontal)
/// - 6: Rotate 90 CW
/// - 7: Transverse (rotate 90 CCW + flip horizontal)
/// - 8: Rotate 90 CCW
pub fn apply_exif_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img, // 1 or unknown: no transform
    }
}
