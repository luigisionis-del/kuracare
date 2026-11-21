//! Meal photos: shrunk, re-encoded as JPEG (which also drops EXIF metadata
//! such as GPS position), then sealed in the vault like any other record.

use std::io::Cursor;

use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageDecoder, ImageReader};

pub const MAX_SIDE: u32 = 1280;

/// Taking photos is paused (21/11/2026); kept, with its test, for when it comes back.
#[allow(dead_code)]
pub fn shrink(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_decoder()
        .map_err(|e| e.to_string())?;
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    img.apply_orientation(orientation);
    if img.width().max(img.height()) > MAX_SIDE {
        img = img.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Triangle);
    }
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 80)
        .encode_image(&img.to_rgb8())
        .map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn data_url(jpeg: &[u8]) -> String {
    format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(jpeg)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_images_are_shrunk_to_jpeg() {
        let img = DynamicImage::new_rgb8(3000, 2000);
        let mut png = Vec::new();
        img.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let jpeg = shrink(&png).unwrap();
        let back = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((back.width(), back.height()), (1280, 853));
        assert!(data_url(&jpeg).starts_with("data:image/jpeg;base64,"));
    }
}
