//! WASM bindings for browser usage

use wasm_bindgen::prelude::*;
use crate::encoder::JpegEncoder;
use crate::png::PngEncoder;

// Set up panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Encode RGB image to JPEG
/// Returns compressed JPEG bytes
#[wasm_bindgen]
pub fn encode_jpeg(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, JsValue> {
    // Validate input
    let expected_size = (width * height * 3) as usize;
    if rgb_data.len() != expected_size {
        return Err(JsValue::from_str(&format!(
            "Invalid RGB data size: expected {} bytes, got {}",
            expected_size,
            rgb_data.len()
        )));
    }

    // Create encoder
    let encoder = JpegEncoder::new(width as u16, height as u16, quality);
    
    // Encode
    encoder.encode(rgb_data)
        .map_err(|e| JsValue::from_str(&e))
}

/// Encode RGB image to PNG
/// Returns compressed PNG bytes
#[wasm_bindgen]
pub fn encode_png(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    compression_level: u32,
) -> Result<Vec<u8>, JsValue> {
    // Validate input
    let expected_size = (width * height * 3) as usize;
    if rgb_data.len() != expected_size {
        return Err(JsValue::from_str(&format!(
            "Invalid RGB data size: expected {} bytes, got {}",
            expected_size,
            rgb_data.len()
        )));
    }

    // Create encoder
    let encoder = PngEncoder::new(width, height)
        .with_compression(compression_level);
    
    // Encode
    encoder.encode_rgb(rgb_data)
        .map_err(|e| JsValue::from_str(&e))
}

/// Encode RGBA image to PNG (with alpha channel)
#[wasm_bindgen]
pub fn encode_png_rgba(
    rgba_data: &[u8],
    width: u32,
    height: u32,
    compression_level: u32,
) -> Result<Vec<u8>, JsValue> {
    let expected_size = (width * height * 4) as usize;
    if rgba_data.len() != expected_size {
        return Err(JsValue::from_str(&format!(
            "Invalid RGBA data size: expected {} bytes, got {}",
            expected_size,
            rgba_data.len()
        )));
    }

    let encoder = PngEncoder::new(width, height)
        .with_compression(compression_level);
    
    encoder.encode_rgba(rgba_data)
        .map_err(|e| JsValue::from_str(&e))
}

/// Get encoder version
#[wasm_bindgen]
pub fn version() -> String {
    crate::VERSION.to_string()
}

/// Log to browser console (for debugging)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Utility for logging from Rust
pub fn console_log(s: &str) {
    log(s);
}
