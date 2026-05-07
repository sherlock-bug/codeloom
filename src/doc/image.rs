// Image compression and base64 encoding
use image::GenericImageView;

pub struct CompressedImage {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub compressed: bool,
}

/// Smart compress: small images (<100KB, <=800px) kept as-is
pub fn smart_compress(raw_bytes: &[u8]) -> anyhow::Result<CompressedImage> {
    if raw_bytes.is_empty() {
        return Ok(CompressedImage { data: vec![], width: 0, height: 0, compressed: false });
    }
    let img = image::load_from_memory(raw_bytes)?;
    let (w, h) = img.dimensions();
    if raw_bytes.len() <= 100_000 && w <= 800 && h <= 800 {
        return Ok(CompressedImage { data: raw_bytes.to_vec(), width: w as i32, height: h as i32, compressed: false });
    }
    compress_webp(&img)
}

/// Compress to lossy WebP (quality 80%, max 1920px wide)
fn compress_webp(img: &image::DynamicImage) -> anyhow::Result<CompressedImage> {
    let img = if img.width() > 1920 {
        let ratio = 1920.0 / img.width() as f64;
        let nh = (img.height() as f64 * ratio) as u32;
        img.resize_exact(1920, nh, image::imageops::FilterType::Lanczos3)
    } else {
        img.clone()
    };

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    // Use webp crate for lossy encoding (libwebp-sys, C library)
    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), w, h);
    let webp_data = encoder.encode(80.0); // quality 80%

    Ok(CompressedImage {
        data: webp_data.to_vec(),
        width: w as i32,
        height: h as i32,
        compressed: true,
    })
}

/// Encode bytes as base64 string (for MCP JSON response)
pub fn to_base64(data: &[u8]) -> String {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(alphabet[(triple >> 18) as usize & 0x3F] as char);
        result.push(alphabet[(triple >> 12) as usize & 0x3F] as char);
        result.push(if chunk.len() > 1 { alphabet[(triple >> 6) as usize & 0x3F] as char } else { '=' });
        result.push(if chunk.len() > 2 { alphabet[triple as usize & 0x3F] as char } else { '=' });
    }
    result
}
