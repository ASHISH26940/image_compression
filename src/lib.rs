//! JPEG and PNG encoder library

pub mod color;
pub mod entropy;
pub mod format;
pub mod quantization;
pub mod transform;
pub mod encoder;
pub mod png;

#[cfg(target_arch = "wasm32")]
pub mod wasm;  // WASM bindings

pub use encoder::JpegEncoder;
pub use png::PngEncoder;

pub const VERSION: &str = "0.1.0";
